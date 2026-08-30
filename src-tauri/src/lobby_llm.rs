use crate::config::OllamaConfig;
use crate::llm_service;
use serde_json::{json, Value};

const SYSTEM_PROMPT: &str = r#"You are WingBot, a League of Legends premade lobby coach.
Input is compact JSON lobby state (queue, readiness, roles). Output exactly a JSON array of 2-3 insight objects.
Each object must have: "type" (positive, negative, warning, neutral), "title" (short, under 6 words), "message" (one actionable sentence).
Focus on role coverage, readiness, flex picks, and queue-specific prep — not repeating raw member counts.
Output ONLY valid JSON — no markdown, no commentary."#;

pub fn build_llm_context(analysis: &Value) -> Value {
    let team = analysis.get("team_analysis").unwrap_or(&Value::Null);
    let members = analysis
        .get("members")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|m| {
                    json!({
                        "riot_id": m.get("riot_id"),
                        "position": m.get("position"),
                        "ready": m.get("ready"),
                        "is_leader": m.get("is_leader"),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    json!({
        "task": "lobby_insights",
        "queue_id": analysis.get("queue_id"),
        "total_members": team.get("total_members"),
        "ready_members": team.get("ready_members"),
        "ready_percentage": team.get("ready_percentage"),
        "champion_diversity": team.get("champion_diversity"),
        "members": members,
    })
}

pub async fn generate_ai_insights(config: &OllamaConfig, analysis: &Value) -> Result<Value, String> {
    if !config.insights_lobby {
        return Err("Lobby AI insights are disabled in Settings".to_string());
    }

    let rate_limit = llm_service::check_lobby_rate_limit(config)?;

    let context = build_llm_context(analysis);
    let user_content = serde_json::to_string(&context)
        .map_err(|e| format!("Failed to encode lobby context: {}", e))?;

    let raw = llm_service::query_with_system(config, Some(SYSTEM_PROMPT), &user_content).await?;
    let insights = llm_service::collect_insights(&raw, 3)?;
    rate_limit.commit();
    Ok(insights)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_lobby_context() {
        let analysis = json!({
            "queue_id": 420,
            "team_analysis": {
                "total_members": 5,
                "ready_members": 4,
                "ready_percentage": 80.0,
                "champion_diversity": 5
            },
            "members": [{ "position": "MIDDLE", "ready": true }]
        });
        let ctx = build_llm_context(&analysis);
        assert_eq!(ctx["task"], "lobby_insights");
    }
}
