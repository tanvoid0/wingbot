use crate::config::OllamaConfig;
use crate::llm_service;
use serde_json::{json, Value};

const SYSTEM_PROMPT: &str = r#"You are WingBot, a League of Legends performance analyst.
Input is compact JSON match history stats. Output exactly a JSON array of 2-3 insight objects.
Each object must have: "type" (one of: positive, negative, warning, neutral), "title" (short, under 6 words), "message" (one actionable sentence).
Focus on patterns: champion fit, role performance, farming vs fighting balance, vision, trends across recent games.
Do not restate obvious stats (win rate, KDA) unless tied to a specific actionable tip.
Output ONLY valid JSON — no markdown, no commentary."#;

/// Build a compact payload for the LLM from a full analysis response.
pub fn build_llm_context(analysis: &Value) -> Value {
    let top_champions = top_champion_buckets(
        analysis.get("champion_performance").unwrap_or(&Value::Null),
        5,
    );
    let role_stats = top_champion_buckets(
        analysis.get("role_stats").unwrap_or(&Value::Null),
        5,
    );
    let recent_results = recent_win_loss(analysis.get("matches").unwrap_or(&Value::Null));

    json!({
        "task": "match_history_insights",
        "total_matches": analysis.get("total_matches"),
        "wins": analysis.get("wins"),
        "losses": analysis.get("losses"),
        "win_rate": analysis.get("win_rate"),
        "avg_kills": analysis.get("avg_kills"),
        "avg_deaths": analysis.get("avg_deaths"),
        "avg_assists": analysis.get("avg_assists"),
        "avg_cs": analysis.get("avg_cs"),
        "avg_damage": analysis.get("avg_damage"),
        "avg_vision": analysis.get("avg_vision"),
        "avg_impact": analysis.get("avg_impact"),
        "top_champions": top_champions,
        "role_stats": role_stats,
        "recent_results": recent_results,
    })
}

fn top_champion_buckets(value: &Value, limit: usize) -> Value {
    let Some(obj) = value.as_object() else {
        return json!([]);
    };

    let mut rows: Vec<Value> = obj
        .values()
        .filter_map(|entry| {
            let games = entry.get("games")?.as_u64()?;
            if games == 0 {
                return None;
            }
            Some(json!({
                "name": entry.get("name").or_else(|| entry.get("champion_name")),
                "games": games,
                "win_rate": entry.get("win_rate"),
                "avg_impact": entry.get("avg_impact"),
            }))
        })
        .collect();

    rows.sort_by(|a, b| {
        b.get("games")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            .cmp(&a.get("games").and_then(|v| v.as_u64()).unwrap_or(0))
    });
    rows.truncate(limit);
    Value::Array(rows)
}

fn recent_win_loss(matches: &Value) -> Value {
    let Some(arr) = matches.as_array() else {
        return json!([]);
    };

    let results: Vec<&str> = arr
        .iter()
        .take(8)
        .filter_map(|m| {
            m.get("stats")
                .and_then(|s| s.get("win"))
                .and_then(|w| w.as_bool())
                .map(|win| if win { "W" } else { "L" })
        })
        .collect();

    json!(results)
}

pub async fn generate_ai_insights(
    config: &OllamaConfig,
    analysis: &Value,
) -> Result<Value, String> {
    if !config.insights_match_history {
        return Err("Match history AI insights are disabled in Settings".to_string());
    }

    let rate_limit = llm_service::check_match_history_rate_limit(config)?;

    let context = build_llm_context(analysis);
    let user_content = serde_json::to_string(&context)
        .map_err(|e| format!("Failed to encode match history context: {}", e))?;

    let raw = llm_service::query_with_system(config, Some(SYSTEM_PROMPT), &user_content).await?;
    let insights = llm_service::collect_insights(&raw, 3)?;
    rate_limit.commit();
    Ok(insights)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_compact_context() {
        let analysis = json!({
            "total_matches": 10,
            "wins": 6,
            "losses": 4,
            "win_rate": 60.0,
            "avg_kills": 5.0,
            "avg_deaths": 3.0,
            "avg_assists": 7.0,
            "champion_performance": {
                "103": { "name": "Ahri", "games": 5, "wins": 4, "win_rate": 80.0, "avg_impact": 72.0 }
            },
            "role_stats": {
                "MIDDLE": { "name": "MIDDLE", "games": 8, "wins": 5, "win_rate": 62.5, "avg_impact": 68.0 }
            },
            "matches": [
                { "stats": { "win": true } },
                { "stats": { "win": false } }
            ]
        });

        let ctx = build_llm_context(&analysis);
        assert_eq!(ctx["task"], "match_history_insights");
        assert_eq!(ctx["top_champions"].as_array().unwrap().len(), 1);
        assert_eq!(ctx["recent_results"], json!(["W", "L"]));
    }

    #[test]
    fn normalizes_insight_entries() {
        let entry = json!({
            "type": "positive",
            "title": "Ahri Anchor",
            "message": "Your Ahri games show strong impact — keep it as a comfort pick."
        });
        let out = llm_service::normalize_insight_entry(&entry).unwrap();
        assert_eq!(out["source"], "ai");
        assert_eq!(out["type"], "positive");
    }
}
