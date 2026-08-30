use crate::config::OllamaConfig;
use crate::llm_service;
use serde_json::{json, Value};

const SYSTEM_PROMPT: &str = r#"You are WingBot, a League of Legends rune coach.
Input is compact JSON rune setup data. Output exactly a JSON array of 2-3 insight objects.
Each object must have: "type" (positive, negative, warning, neutral), "title" (short, under 6 words), "message" (one actionable sentence).
Focus on keystone fit, tree synergy, missing slots, and matchup-relevant rune pivots — not generic rune descriptions.
Output ONLY valid JSON — no markdown, no commentary."#;

pub fn build_llm_context(analysis: &Value, team_trees: &Value) -> Value {
    json!({
        "task": "rune_insights",
        "champion": analysis.get("champion_name").or_else(|| analysis.get("champion")),
        "role": analysis.get("role"),
        "primary_tree": analysis.get("primary_tree"),
        "secondary_tree": analysis.get("secondary_tree"),
        "keystone": analysis.get("keystone"),
        "total_runes": analysis.get("total_runes"),
        "quality_score": analysis.get("quality_score"),
        "issues": analysis.get("issues"),
        "team_primary_trees": team_trees,
    })
}

pub async fn generate_ai_insights(
    config: &OllamaConfig,
    analysis: &Value,
    team_trees: &Value,
) -> Result<Value, String> {
    if !config.insights_rune {
        return Err("Rune AI insights are disabled in Settings".to_string());
    }

    let rate_limit = llm_service::check_rune_rate_limit(config)?;

    let context = build_llm_context(analysis, team_trees);
    let user_content = serde_json::to_string(&context)
        .map_err(|e| format!("Failed to encode rune context: {}", e))?;

    let raw = llm_service::query_with_system(config, Some(SYSTEM_PROMPT), &user_content).await?;
    let insights = llm_service::collect_insights(&raw, 3)?;
    rate_limit.commit();
    Ok(insights)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_rune_context() {
        let analysis = json!({
            "primary_tree": "Domination",
            "secondary_tree": "Sorcery",
            "keystone": "Electrocute",
            "total_runes": 6
        });
        let ctx = build_llm_context(&analysis, &json!(["Domination", "Precision"]));
        assert_eq!(ctx["task"], "rune_insights");
    }
}
