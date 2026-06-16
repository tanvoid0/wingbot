use serde_json::{json, Value};

use crate::config::{resolve_riot_config, RiotApiConfig};
use crate::data_processing::MatchHistoryAnalyzer;
use crate::lcu_api::LCUAPI;
use crate::riot_api::{routing_from_summoner, RiotApiClient};

pub const DEFAULT_MATCH_COUNT: u32 = 20;
pub const MATCH_COUNT_STEP: u32 = 20;
pub const MAX_MATCH_COUNT: u32 = 100;
pub const LCU_MATCH_PAGE_SIZE: u32 = 20;

pub fn clamp_match_count(count: Option<u32>) -> u32 {
    count
        .unwrap_or(DEFAULT_MATCH_COUNT)
        .clamp(1, MAX_MATCH_COUNT)
}

pub struct MatchHistoryService;

impl MatchHistoryService {
    /// Fetch and analyze match history for the logged-in summoner.
    /// Prefers LCU (no API key); falls back to Riot API when configured.
    pub async fn fetch_current_player_analysis(
        lcu_api: &LCUAPI,
        match_count: u32,
        force_riot_refresh: bool,
    ) -> Result<Value, String> {
        let match_count = clamp_match_count(Some(match_count));
        let riot_config = current_riot_config();

        let summoner = lcu_api.get_current_summoner().await.ok();
        let puuid = summoner
            .as_ref()
            .and_then(|s| s.get("puuid"))
            .and_then(|v| v.as_str())
            .map(str::to_string);

        // Try LCU first
        if lcu_api.has_connection() {
            if let Some(ref id) = puuid {
                match lcu_api.get_match_history(id, match_count).await {
                    Ok(raw) => {
                        if let Some(analysis) = MatchHistoryAnalyzer::from_lcu(&raw, Some(id)) {
                            let mut result = enrich_analysis(
                                analysis.to_json(),
                                match_count,
                                analysis.total_matches,
                            );
                            if let Some(ranked) = lcu_api.get_ranked_stats().await.ok() {
                                result["ranked"] = ranked;
                            }
                            return Ok(result);
                        }
                        eprintln!("[MatchHistory] LCU returned data but no games to analyze");
                    }
                    Err(e) => eprintln!("[MatchHistory] LCU match history failed: {}", e),
                }
            }
        }

        // Riot API fallback
        if let Some(id) = puuid {
            return Self::fetch_riot_analysis(
                &id,
                &riot_config,
                summoner.as_ref(),
                match_count,
                force_riot_refresh,
            )
            .await;
        }

        Err("No summoner PUUID available — log into the League client".to_string())
    }

    /// Fetch and analyze match history for any player by PUUID.
    pub async fn fetch_player_analysis(
        lcu_api: &LCUAPI,
        puuid: &str,
        match_count: u32,
        force_riot_refresh: bool,
    ) -> Result<Value, String> {
        let match_count = clamp_match_count(Some(match_count));
        let riot_config = current_riot_config();

        if lcu_api.has_connection() {
            if let Ok(raw) = lcu_api.get_match_history(puuid, match_count).await {
                if let Some(analysis) = MatchHistoryAnalyzer::from_lcu(&raw, Some(puuid)) {
                    return Ok(enrich_analysis(
                        analysis.to_json(),
                        match_count,
                        analysis.total_matches,
                    ));
                }
            }
        }

        Self::fetch_riot_analysis(
            puuid,
            &riot_config,
            None,
            match_count,
            force_riot_refresh,
        )
        .await
    }

    async fn fetch_riot_analysis(
        puuid: &str,
        base_config: &RiotApiConfig,
        summoner: Option<&Value>,
        match_count: u32,
        force: bool,
    ) -> Result<Value, String> {
        let mut config = base_config.clone();
        if let Some(s) = summoner {
            config.routing_region = routing_from_summoner(s, base_config);
        }

        let client = RiotApiClient::new(config);
        if !client.is_configured() {
            return Err(
                "Riot API key not configured — paste your key in Settings"
                    .to_string(),
            );
        }

        client
            .fetch_match_analysis(puuid, match_count, force)
            .await
            .map(|value| {
                persist_riot_verification(true, None);
                value
            })
            .map_err(|e| {
                persist_riot_verification(false, Some(e.clone()));
                e
            })
    }
}

fn persist_riot_verification(success: bool, error: Option<String>) {
    if let Ok(mut mgr) = crate::get_config_manager().lock() {
        let _ = mgr.record_riot_api_verification(success, error);
    }
}

fn enrich_analysis(result: Value, requested: u32, fetched: u32) -> Value {
    enrich_analysis_for_response(result, requested, fetched)
}

pub fn enrich_analysis_for_response(mut result: Value, requested: u32, fetched: u32) -> Value {
    let has_more = fetched >= requested && requested < MAX_MATCH_COUNT;
    if let Some(obj) = result.as_object_mut() {
        obj.insert("requested_match_count".into(), json!(requested));
        obj.insert("has_more_matches".into(), json!(has_more));
    }
    result
}

fn current_riot_config() -> RiotApiConfig {
    crate::get_config_manager()
        .lock()
        .map(|m| resolve_riot_config(&m.get_config().riot_api))
        .unwrap_or_else(|_| resolve_riot_config(&RiotApiConfig::default()))
}

/// Invalidate Riot API cache when a game finishes so the next fetch picks up new matches.
pub async fn maybe_invalidate_riot_cache_on_game_end(
    lcu_api: &LCUAPI,
    previous_phase: &str,
    current_phase: &str,
) {
    let was_in_game = previous_phase == "InProgress" || previous_phase == "GameStart";
    let post_game = matches!(
        current_phase,
        "EndOfGame" | "WaitingForStats" | "PreEndOfGame"
    );

    if !was_in_game || !post_game {
        return;
    }

    if let Ok(summoner) = lcu_api.get_current_summoner().await {
        if let Some(puuid) = summoner.get("puuid").and_then(|v| v.as_str()) {
            crate::riot_api_cache::invalidate_player(puuid).await;
        }
    }
}
