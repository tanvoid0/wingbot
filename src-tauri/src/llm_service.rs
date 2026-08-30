use crate::config::OllamaConfig;
use serde_json::{json, Value};
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

static LAST_QUERY: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
static LAST_MATCH_HISTORY_QUERY: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
static LAST_SCOUT_QUERY: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
static LAST_RUNE_QUERY: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
static LAST_LOBBY_QUERY: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

fn last_query_mutex() -> &'static Mutex<Option<Instant>> {
    LAST_QUERY.get_or_init(|| Mutex::new(None))
}

fn last_match_history_query_mutex() -> &'static Mutex<Option<Instant>> {
    LAST_MATCH_HISTORY_QUERY.get_or_init(|| Mutex::new(None))
}

fn last_scout_query_mutex() -> &'static Mutex<Option<Instant>> {
    LAST_SCOUT_QUERY.get_or_init(|| Mutex::new(None))
}

fn last_rune_query_mutex() -> &'static Mutex<Option<Instant>> {
    LAST_RUNE_QUERY.get_or_init(|| Mutex::new(None))
}

fn last_lobby_query_mutex() -> &'static Mutex<Option<Instant>> {
    LAST_LOBBY_QUERY.get_or_init(|| Mutex::new(None))
}

/// A held slot in a rate-limited window.
///
/// Dropping without calling [`Self::commit`] restores the previous timestamp,
/// so a request that fails does not burn the caller's cooldown. Without this,
/// one failure hides the real error behind "Please wait Ns" for a full
/// interval, and `?` on any intermediate step is enough to trigger it.
#[must_use = "the reservation rolls back unless committed"]
pub struct RateLimitGuard {
    mutex: &'static Mutex<Option<Instant>>,
    previous: Option<Instant>,
    committed: bool,
}

impl RateLimitGuard {
    /// Keeps the reservation — call once the request has actually succeeded.
    pub fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for RateLimitGuard {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        if let Ok(mut guard) = self.mutex.lock() {
            *guard = self.previous;
        }
    }
}

fn check_rate_limit_on(
    mutex: &'static Mutex<Option<Instant>>,
    interval_secs: u64,
    label: &str,
) -> Result<RateLimitGuard, String> {
    let interval = Duration::from_secs(interval_secs.max(5));
    let mut guard = mutex
        .lock()
        .map_err(|e| format!("Rate limit lock error: {}", e))?;

    if let Some(last) = *guard {
        let elapsed = last.elapsed();
        if elapsed < interval {
            let wait = (interval - elapsed).as_secs();
            return Err(format!(
                "Please wait {}s before requesting new {}",
                wait.max(1),
                label
            ));
        }
    }

    let previous = *guard;
    *guard = Some(Instant::now());

    Ok(RateLimitGuard {
        mutex,
        previous,
        committed: false,
    })
}

pub fn check_rate_limit(config: &OllamaConfig) -> Result<RateLimitGuard, String> {
    check_rate_limit_on(
        last_query_mutex(),
        config.poll_interval_sec,
        "AI recommendations",
    )
}

pub fn check_match_history_rate_limit(config: &OllamaConfig) -> Result<RateLimitGuard, String> {
    // Match history changes less often — allow a longer cooldown than live game advice.
    let interval = config.poll_interval_sec.max(60);
    check_rate_limit_on(
        last_match_history_query_mutex(),
        interval,
        "match history insights",
    )
}

pub fn check_scout_rate_limit(config: &OllamaConfig) -> Result<RateLimitGuard, String> {
    let interval = config.poll_interval_sec.max(45);
    check_rate_limit_on(
        last_scout_query_mutex(),
        interval,
        "scout insights",
    )
}

pub fn check_rune_rate_limit(config: &OllamaConfig) -> Result<RateLimitGuard, String> {
    let interval = config.poll_interval_sec.max(45);
    check_rate_limit_on(last_rune_query_mutex(), interval, "rune insights")
}

pub fn check_lobby_rate_limit(config: &OllamaConfig) -> Result<RateLimitGuard, String> {
    let interval = config.poll_interval_sec.max(45);
    check_rate_limit_on(last_lobby_query_mutex(), interval, "lobby insights")
}

pub fn normalize_insight_entry(entry: &Value) -> Option<Value> {
    let title = entry.get("title")?.as_str()?.trim();
    let message = entry.get("message")?.as_str()?.trim();
    if title.is_empty() || message.is_empty() {
        return None;
    }

    let insight_type = entry
        .get("type")
        .and_then(|v| v.as_str())
        .map(|t| match t {
            "positive" | "negative" | "warning" | "neutral" => t,
            _ => "neutral",
        })
        .unwrap_or("neutral");

    Some(json!({
        "type": insight_type,
        "title": title,
        "message": message,
        "source": "ai",
    }))
}

pub fn collect_insights(raw: &str, max: usize) -> Result<Value, String> {
    let parsed = parse_json_array(raw)?;
    let insights: Vec<Value> = parsed
        .as_array()
        .ok_or("Model insight response was not a JSON array")?
        .iter()
        .filter_map(normalize_insight_entry)
        .take(max)
        .collect();

    if insights.is_empty() {
        return Err("Model returned no usable insights".to_string());
    }

    Ok(Value::Array(insights))
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
    query_with_system(config, None, user_content).await
}

pub async fn query_with_system(
    config: &OllamaConfig,
    system_content: Option<&str>,
    user_content: &str,
) -> Result<String, String> {
    let base = normalize_base_url(&config.base_url);
    let url = format!("{}/v1/chat/completions", base);

    let mut messages = Vec::new();
    if let Some(system) = system_content.filter(|s| !s.trim().is_empty()) {
        messages.push(json!({ "role": "system", "content": system }));
    }
    messages.push(json!({ "role": "user", "content": user_content }));

    let body = json!({
        "model": config.model_name,
        "stream": false,
        "messages": messages,
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

/// Extract a JSON array from model output (handles optional markdown fences).
pub fn parse_json_array(text: &str) -> Result<Value, String> {
    let trimmed = text.trim();
    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        if value.is_array() {
            return Ok(value);
        }
    }

    let start = trimmed.find('[').ok_or("Model did not return a JSON array")?;
    let end = trimmed
        .rfind(']')
        .ok_or("Model did not return a JSON array")?;
    let slice = &trimmed[start..=end];
    serde_json::from_str(slice).map_err(|e| format!("Failed to parse insight JSON: {}", e))
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

    static TEST_SLOT: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

    fn test_slot() -> &'static Mutex<Option<Instant>> {
        TEST_SLOT.get_or_init(|| Mutex::new(None))
    }

    /// A failed request must not burn the cooldown — otherwise the real error
    /// is hidden behind "Please wait Ns" for a full interval.
    #[test]
    fn rolled_back_reservation_frees_the_slot() {
        // Dropped without commit, as happens when `?` bails on a later step.
        drop(check_rate_limit_on(test_slot(), 60, "test").expect("first reservation"));

        let second = check_rate_limit_on(test_slot(), 60, "test")
            .expect("a rolled-back reservation must leave the slot free");

        // Committing marks the request as having actually happened.
        second.commit();

        let third = check_rate_limit_on(test_slot(), 60, "test");
        let err = third.err().expect("committed reservation must hold the slot");
        assert!(err.contains("Please wait"), "unexpected message: {err}");
    }
}
