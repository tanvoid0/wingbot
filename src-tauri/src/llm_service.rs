use crate::config::OllamaConfig;
use serde_json::{json, Value};
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

static LAST_QUERY: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

fn last_query_mutex() -> &'static Mutex<Option<Instant>> {
    LAST_QUERY.get_or_init(|| Mutex::new(None))
}

pub fn check_rate_limit(config: &OllamaConfig) -> Result<(), String> {
    let interval = Duration::from_secs(config.poll_interval_sec.max(5));
    let mut guard = last_query_mutex()
        .lock()
        .map_err(|e| format!("Rate limit lock error: {}", e))?;

    if let Some(last) = *guard {
        let elapsed = last.elapsed();
        if elapsed < interval {
            let wait = (interval - elapsed).as_secs();
            return Err(format!(
                "Please wait {}s before requesting new AI recommendations",
                wait.max(1)
            ));
        }
    }

    *guard = Some(Instant::now());
    Ok(())
}

fn normalize_base_url(base_url: &str) -> String {
    let trimmed = base_url.trim().trim_end_matches('/');
    // Avoid Windows IPv6 localhost hangs when Ollama listens on 127.0.0.1 only.
    if trimmed.contains("://localhost") {
        trimmed.replace("://localhost", "://127.0.0.1")
    } else {
        trimmed.to_string()
    }
}

fn model_matches(listed_name: &str, wanted: &str) -> bool {
    let listed = listed_name.trim();
    let wanted = wanted.trim();
    if listed.is_empty() || wanted.is_empty() {
        return false;
    }
    if listed == wanted {
        return true;
    }
    let listed_base = listed.split(':').next().unwrap_or(listed);
    let wanted_base = wanted.split(':').next().unwrap_or(wanted);
    listed_base == wanted_base
        || listed.starts_with(&format!("{wanted_base}:"))
        || listed.starts_with(&format!("{wanted}:"))
}

fn http_client(timeout_secs: u64) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
}

pub async fn query(config: &OllamaConfig, user_content: &str) -> Result<String, String> {
    let base = normalize_base_url(&config.base_url);
    let url = format!("{}/v1/chat/completions", base);

    let body = json!({
        "model": config.model_name,
        "stream": false,
        "messages": [
            { "role": "user", "content": user_content }
        ]
    });

    let client = http_client(120)?;

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() {
                "Ollama is not running — start Ollama and ensure the model is loaded".to_string()
            } else if e.is_timeout() {
                "Ollama request timed out".to_string()
            } else {
                format!("Ollama request failed: {}", e)
            }
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Ollama returned HTTP {}: {}", status, text));
    }

    let data: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

    data.get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Ollama returned an empty response".to_string())
}

pub async fn test_connection(config: &OllamaConfig) -> Result<Value, String> {
    let base = normalize_base_url(&config.base_url);
    let url = format!("{}/api/tags", base);

    let client = http_client(10)?;

    let response = client.get(&url).send().await.map_err(|e| {
        if e.is_connect() {
            "Cannot connect to Ollama — start Ollama (ollama serve or desktop app)".to_string()
        } else if e.is_timeout() {
            format!(
                "Ollama connection timed out at {} — try http://127.0.0.1:11434 instead of localhost",
                base
            )
        } else {
            format!("Connection test failed: {}", e)
        }
    })?;

    if !response.status().is_success() {
        return Err(format!(
            "Ollama health check failed: HTTP {}",
            response.status()
        ));
    }

    let data: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Ollama tags: {}", e))?;

    let model_found = data
        .get("models")
        .and_then(|m| m.as_array())
        .map(|models| {
            models.iter().any(|m| {
                m.get("name")
                    .and_then(|n| n.as_str())
                    .map(|name| model_matches(name, &config.model_name))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    if !model_found {
        return Err(format!(
            "Ollama is running but model '{}' was not found — run: ollama list",
            config.model_name
        ));
    }

    Ok(json!({
        "ok": true,
        "model": config.model_name,
        "base_url": base,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_matches_wingbot_lol_latest() {
        assert!(model_matches("wingbot-lol:latest", "wingbot-lol"));
        assert!(model_matches("wingbot-lol", "wingbot-lol:latest"));
    }

    #[test]
    fn normalize_localhost_to_ipv4() {
        assert_eq!(
            normalize_base_url("http://localhost:11434/"),
            "http://127.0.0.1:11434"
        );
    }
}
