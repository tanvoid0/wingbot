use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use std::time::Duration;

use crate::config::{resolve_riot_config, sanitize_riot_api_key, RiotApiConfig};
use crate::data_processing::MatchHistoryAnalyzer;
use crate::match_history_service;
use crate::riot_api_cache;

const RIOT_API_BASE: &str = "https://{routing}.api.riotgames.com";

pub struct RiotApiClient {
    client: Client,
    config: RiotApiConfig,
}

impl RiotApiClient {
    pub fn new(config: RiotApiConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            config: resolve_riot_config(&config),
            client,
        }
    }

    pub fn is_configured(&self) -> bool {
        self.api_key().is_some()
    }

    fn api_key(&self) -> Option<String> {
        self.config
            .api_key
            .as_ref()
            .map(|k| sanitize_riot_api_key(k))
            .filter(|k| !k.is_empty())
    }

    fn routing_region(&self) -> &str {
        if !self.config.routing_region.is_empty() {
            return &self.config.routing_region;
        }
        platform_to_routing(&self.config.platform)
    }

    fn base_url(&self) -> String {
        RIOT_API_BASE.replace("{routing}", self.routing_region())
    }

    async fn get(&self, path: &str) -> Result<Value, String> {
        let api_key = self
            .api_key()
            .ok_or("Riot API key not configured — add it in Settings")?;

        if !api_key.starts_with("RGAPI-") {
            return Err(
                "Riot API key looks invalid (expected RGAPI- prefix). Regenerate at developer.riotgames.com"
                    .to_string(),
            );
        }

        let url = format!("{}{}", self.base_url(), path);

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Riot-Token",
            HeaderValue::from_str(&api_key)
                .map_err(|_| "Riot API key contains invalid characters".to_string())?,
        );

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("Riot API request failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let hint = riot_error_hint(status.as_u16(), &body);
            return Err(format!("Riot API HTTP {}: {}{}", status, body, hint));
        }

        response
            .json::<Value>()
            .await
            .map_err(|e| format!("Failed to parse Riot API response: {}", e))
    }

    /// Step 1 from https://www.itero.gg/articles/riot-api — list recent match IDs.
    pub async fn get_match_ids(&self, puuid: &str, start: u32, count: u32) -> Result<Vec<String>, String> {
        let path = format!(
            "/lol/match/v5/matches/by-puuid/{}/ids?start={}&count={}",
            puuid, start, count
        );
        let ids = self.get(&path).await?;
        let list = ids
            .as_array()
            .ok_or("Unexpected match ID list format")?
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        Ok(list)
    }

    /// Step 2 — fetch full match detail for a single match ID.
    pub async fn get_match(&self, match_id: &str) -> Result<Value, String> {
        self.get(&format!("/lol/match/v5/matches/{}", match_id))
            .await
    }

    /// Fetch match IDs then load each match document.
    /// Uses cache: skips detail fetches for known match IDs unless `force` is true.
    pub async fn fetch_recent_matches(
        &self,
        puuid: &str,
        count: u32,
        force: bool,
    ) -> Result<(Vec<String>, Vec<Value>), String> {
        let count = match_history_service::clamp_match_count(Some(count));

        if !force {
            if let Some(cached_ids) = riot_api_cache::get_cached_match_ids(puuid, false).await {
                if cached_ids.len() >= count as usize {
                    let ids: Vec<String> = cached_ids.iter().take(count as usize).cloned().collect();
                    if let Some(matches) = riot_api_cache::resolve_matches_from_cache(&ids).await {
                        eprintln!(
                            "[RiotCache] Using cached matches for {} ({} games, 0 API calls)",
                            puuid,
                            matches.len()
                        );
                        return Ok((ids, matches));
                    }
                } else if !cached_ids.is_empty() {
                    return self
                        .extend_cached_matches(puuid, cached_ids, count)
                        .await;
                }
            }
        }

        let ids = self.get_match_ids(puuid, 0, count).await?;

        if !force {
            if let Some(cached_ids) = riot_api_cache::get_cached_match_ids(puuid, false).await {
                if cached_ids.len() >= ids.len() && cached_ids[..ids.len()] == ids[..] {
                    if let Some(matches) = riot_api_cache::resolve_matches_from_cache(&ids).await {
                        eprintln!(
                            "[RiotCache] Match IDs unchanged for {} ({} games, 1 API call)",
                            puuid,
                            matches.len()
                        );
                        return Ok((ids, matches));
                    }
                }
            }
        }

        let matches = self.fetch_match_details(&ids).await?;

        eprintln!(
            "[RiotCache] Fetched match details for {} ({} total)",
            puuid,
            matches.len()
        );

        Ok((ids, matches))
    }

    async fn extend_cached_matches(
        &self,
        puuid: &str,
        mut cached_ids: Vec<String>,
        count: u32,
    ) -> Result<(Vec<String>, Vec<Value>), String> {
        let start = cached_ids.len() as u32;
        let additional = count.saturating_sub(start);
        let new_ids = self.get_match_ids(puuid, start, additional).await?;
        let new_count = new_ids.len();
        cached_ids.extend(new_ids);

        let matches = self.fetch_match_details(&cached_ids).await?;

        eprintln!(
            "[RiotCache] Extended cached matches for {} ({} games, {} new IDs)",
            puuid,
            matches.len(),
            new_count
        );

        Ok((cached_ids, matches))
    }

    async fn fetch_match_details(&self, ids: &[String]) -> Result<Vec<Value>, String> {
        let mut matches = Vec::with_capacity(ids.len());
        let mut api_calls = 0u32;

        for id in ids {
            if let Some(cached) = riot_api_cache::get_cached_match(id).await {
                matches.push(cached);
                continue;
            }

            match self.get_match(id).await {
                Ok(m) => {
                    api_calls += 1;
                    riot_api_cache::store_match(id, m.clone()).await;
                    matches.push(m);
                }
                Err(e) => eprintln!("[RiotAPI] Skipping match {}: {}", id, e),
            }
        }

        eprintln!(
            "[RiotCache] Fetched {} new match details ({} total)",
            api_calls,
            matches.len()
        );

        Ok(matches)
    }

    /// Fetch and analyze recent matches for a player (cache-aware).
    pub async fn fetch_match_analysis(
        &self,
        puuid: &str,
        count: u32,
        force: bool,
    ) -> Result<Value, String> {
        let count = match_history_service::clamp_match_count(Some(count));

        if !force {
            if let Some(cached) = riot_api_cache::get_cached_analysis(puuid, false).await {
                let cached_requested = cached
                    .get("requested_match_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                let cached_total = cached
                    .get("total_matches")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;

                if cached_requested >= count && cached_total >= count {
                    eprintln!(
                        "[RiotCache] Returning cached analysis for {} ({} games, 0 API calls)",
                        puuid,
                        count
                    );
                    return Ok(cached);
                }
            }
        }

        let (ids, matches) = self.fetch_recent_matches(puuid, count, force).await?;
        let analysis = MatchHistoryAnalyzer::from_riot_matches(&matches, puuid)
            .map(|a| a.to_json())
            .ok_or_else(|| "No analyzable matches returned from Riot API".to_string())?;

        let fetched = analysis
            .get("total_matches")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let enriched = match_history_service::enrich_analysis_for_response(analysis, count, fetched);

        riot_api_cache::store_player_analysis(puuid, ids, enriched.clone()).await;
        Ok(enriched)
    }

    /// Quick connectivity check using match ID lookup.
    pub async fn test_connection(&self, puuid: &str) -> Result<Value, String> {
        let ids = self.get_match_ids(puuid, 0, 1).await?;
        Ok(serde_json::json!({
            "ok": true,
            "routing": self.routing_region(),
            "platform": self.config.platform,
            "sample_match_count": ids.len(),
        }))
    }
}

pub fn is_riot_key_expired_error(status: u16, body: &str) -> bool {
    status == 401
        || status == 403
        || body.contains("Exception decrypting")
        || body.to_lowercase().contains("expired")
}

fn riot_error_hint(status: u16, body: &str) -> String {
    if status == 401 || body.contains("Exception decrypting") {
        return " — Your dev API key is invalid or expired. Regenerate at https://developer.riotgames.com/ (dev keys last 24h).".to_string();
    }
    if status == 403 {
        return " — API key forbidden. Check that your app is registered and the key is a Development key.".to_string();
    }
    if status == 404 {
        return " — Player or match not found. Check platform/routing region matches your account.".to_string();
    }
    String::new()
}

pub fn platform_to_routing(platform: &str) -> &'static str {
    match platform.to_lowercase().as_str() {
        "na1" | "br1" | "la1" | "la2" => "americas",
        "euw1" | "eun1" | "tr1" | "ru" => "europe",
        "kr" | "jp1" => "asia",
        "oc1" | "ph2" | "sg2" | "th2" | "tw2" | "vn2" => "sea",
        _ => "americas",
    }
}

pub fn routing_from_summoner(summoner: &Value, config: &RiotApiConfig) -> String {
    let platform = summoner
        .get("platformId")
        .or_else(|| summoner.get("platform"))
        .and_then(|v| v.as_str())
        .unwrap_or(&config.platform);
    platform_to_routing(platform).to_string()
}
