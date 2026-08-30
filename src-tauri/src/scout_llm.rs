use crate::config::OllamaConfig;
use crate::llm_service;
use serde_json::{json, Value};

const SYSTEM_PROMPT: &str = r#"You are WingBot, a League of Legends draft analyst.
Input is compact JSON with enemy scout profiles (rank, mastery, one-trick flags).
Output exactly a JSON array of 2-3 insight objects for the user's team.
Each object must have: "type" (positive, negative, warning, neutral), "title" (short, under 6 words), "message" (one actionable sentence).
Focus on threats to respect, lanes to target, and draft patterns (OTP, high elo, off-role picks).
Do not repeat raw rank numbers unless tied to a specific plan.
Output ONLY valid JSON — no markdown, no commentary."#;

pub fn build_llm_context(payload: &Value) -> Value {
    let enemies = payload
        .get("enemies")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let compact_enemies: Vec<Value> = enemies
        .iter()
        .filter_map(|enemy| {
            let rank = enemy
                .get("primary_rank")
                .or_else(|| enemy.get("solo_rank"))
                .or_else(|| enemy.get("flex_rank"));
            Some(json!({
                "riot_id": enemy.get("riot_id"),
                "position": enemy.get("position"),
                "champion_id": enemy.get("champion_id"),
                "champion_name": enemy.get("champion_name"),
                "is_one_trick": enemy.get("is_one_trick"),
                "champion_in_top_masteries": enemy.get("champion_in_top_masteries"),
                "rank_tier": rank.and_then(|r| r.get("tier")),
                "rank_summary": rank.and_then(|r| r.get("summary")),
                "rank_win_rate": rank.and_then(|r| r.get("win_rate")),
                "champion_mastery_level": enemy
                    .get("champion_mastery")
                    .and_then(|m| m.get("level")),
                "champion_form": enemy.get("champion_form").and_then(|f| f.get("summary")),
            }))
        })
        .collect();

    json!({
        "task": "enemy_scout_insights",
        "enemies": compact_enemies,
    })
}

pub async fn generate_ai_insights(
    config: &OllamaConfig,
    payload: &Value,
) -> Result<Value, String> {
    if !config.insights_scout {
        return Err("Scout AI insights are disabled in Settings".to_string());
    }

    let rate_limit = llm_service::check_scout_rate_limit(config)?;

    let context = build_llm_context(payload);
    let user_content = serde_json::to_string(&context)
        .map_err(|e| format!("Failed to encode scout context: {}", e))?;

    let raw = llm_service::query_with_system(config, Some(SYSTEM_PROMPT), &user_content).await?;
    let insights = llm_service::collect_insights(&raw, 3)?;
    rate_limit.commit();
    Ok(insights)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_scout_context() {
        let payload = json!({
            "enemies": [{
                "riot_id": "Player#NA1",
                "position": "JUNGLE",
                "champion_id": 104,
                "champion_name": "Graves",
                "is_one_trick": true,
                "primary_rank": { "tier": "DIAMOND", "summary": "Diamond II", "win_rate": 54.0 },
                "champion_mastery": { "level": 7, "points": 450000 }
            }]
        });

        let ctx = build_llm_context(&payload);
        assert_eq!(ctx["task"], "enemy_scout_insights");
        assert_eq!(ctx["enemies"].as_array().unwrap().len(), 1);
    }
}
