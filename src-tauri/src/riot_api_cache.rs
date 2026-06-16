use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use tokio::sync::Mutex;

/// In-memory cache for Riot API match data.
///
/// Match documents are immutable once fetched and are kept indefinitely.
/// Per-player match ID lists and analysis are refreshed only when invalidated
/// (game finished, manual refresh) or on first fetch.
struct RiotApiCacheInner {
    /// match_id → full match document (never expires)
    matches: HashMap<String, Value>,
    /// puuid → cached match ID list + derived analysis
    players: HashMap<String, PlayerEntry>,
    /// Players whose ID list / analysis must be re-checked on next fetch
    stale_players: HashSet<String>,
}

#[derive(Clone)]
struct PlayerEntry {
    match_ids: Vec<String>,
    analysis: Value,
}

struct RiotApiCache {
    inner: Mutex<RiotApiCacheInner>,
}

impl RiotApiCacheInner {
    fn new() -> Self {
        Self {
            matches: HashMap::new(),
            players: HashMap::new(),
            stale_players: HashSet::new(),
        }
    }

    fn cached_analysis(&self, puuid: &str) -> Option<Value> {
        if self.stale_players.contains(puuid) {
            return None;
        }
        self.players.get(puuid).map(|e| e.analysis.clone())
    }

    fn cached_match_ids(&self, puuid: &str) -> Option<Vec<String>> {
        if self.stale_players.contains(puuid) {
            return None;
        }
        self.players.get(puuid).map(|e| e.match_ids.clone())
    }

    fn get_match(&self, match_id: &str) -> Option<Value> {
        self.matches.get(match_id).cloned()
    }

    fn store_match(&mut self, match_id: &str, data: Value) {
        self.matches.insert(match_id.to_string(), data);
    }

    fn store_player(&mut self, puuid: &str, match_ids: Vec<String>, analysis: Value) {
        self.stale_players.remove(puuid);
        self.players.insert(
            puuid.to_string(),
            PlayerEntry {
                match_ids,
                analysis,
            },
        );
    }

    fn invalidate_player(&mut self, puuid: &str) {
        self.stale_players.insert(puuid.to_string());
        self.players.remove(puuid);
    }

    fn resolve_matches(&self, ids: &[String]) -> Vec<Value> {
        ids.iter()
            .filter_map(|id| self.matches.get(id).cloned())
            .collect()
    }
}

static RIOT_API_CACHE: OnceLock<RiotApiCache> = OnceLock::new();

fn cache() -> &'static RiotApiCache {
    RIOT_API_CACHE.get_or_init(|| RiotApiCache {
        inner: Mutex::new(RiotApiCacheInner::new()),
    })
}

/// Mark a player's cached match list and analysis as stale (e.g. after a game ends).
pub async fn invalidate_player(puuid: &str) {
    let mut inner = cache().inner.lock().await;
    inner.invalidate_player(puuid);
    eprintln!("[RiotCache] Invalidated cache for player {}", puuid);
}

/// Return cached analysis when fresh; `None` if stale, missing, or `force` is true.
pub async fn get_cached_analysis(puuid: &str, force: bool) -> Option<Value> {
    if force {
        return None;
    }
    let inner = cache().inner.lock().await;
    inner.cached_analysis(puuid)
}

/// Return cached match IDs when fresh.
pub async fn get_cached_match_ids(puuid: &str, force: bool) -> Option<Vec<String>> {
    if force {
        return None;
    }
    let inner = cache().inner.lock().await;
    inner.cached_match_ids(puuid)
}

/// Look up a single match document from cache.
pub async fn get_cached_match(match_id: &str) -> Option<Value> {
    let inner = cache().inner.lock().await;
    inner.get_match(match_id)
}

/// Store a match document and update player-level cache after a fetch cycle.
pub async fn store_match(match_id: &str, data: Value) {
    let mut inner = cache().inner.lock().await;
    inner.store_match(match_id, data);
}

pub async fn store_player_analysis(puuid: &str, match_ids: Vec<String>, analysis: Value) {
    let mut inner = cache().inner.lock().await;
    inner.store_player(puuid, match_ids, analysis);
}

/// Build match list purely from cache (no API). Returns `None` if any ID is missing.
pub async fn resolve_matches_from_cache(ids: &[String]) -> Option<Vec<Value>> {
    let inner = cache().inner.lock().await;
    let matches = inner.resolve_matches(ids);
    if matches.len() == ids.len() {
        Some(matches)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    async fn fresh_cache() {
        let mut inner = cache().inner.lock().await;
        *inner = RiotApiCacheInner::new();
    }

    #[tokio::test]
    async fn cache_hit_and_miss() {
        fresh_cache().await;

        let puuid = "cache-hit-miss-puuid";
        let analysis = json!({ "total_matches": 5, "source": "riot_api" });
        let ids = vec!["NA1_1".to_string(), "NA1_2".to_string()];

        assert!(get_cached_analysis(puuid, false).await.is_none());

        store_player_analysis(puuid, ids.clone(), analysis.clone()).await;
        assert_eq!(
            get_cached_analysis(puuid, false).await,
            Some(analysis.clone())
        );
        assert!(get_cached_analysis(puuid, true).await.is_none());
        assert_eq!(get_cached_match_ids(puuid, false).await, Some(ids));
    }

    #[tokio::test]
    async fn invalidation_clears_player_cache() {
        fresh_cache().await;

        let puuid = "invalidation-puuid";
        store_player_analysis(
            puuid,
            vec!["NA1_1".to_string()],
            json!({ "total_matches": 1 }),
        )
        .await;

        invalidate_player(puuid).await;
        assert!(get_cached_analysis(puuid, false).await.is_none());

        store_match("NA1_1", json!({ "metadata": { "matchId": "NA1_1" } })).await;
        assert!(get_cached_match("NA1_1").await.is_some());
    }

    #[tokio::test]
    async fn resolve_matches_requires_all_ids() {
        fresh_cache().await;

        store_match("NA1_1", json!({ "id": 1 })).await;
        let partial = resolve_matches_from_cache(&["NA1_1".to_string(), "NA1_2".to_string()])
            .await;
        assert!(partial.is_none());

        store_match("NA1_2", json!({ "id": 2 })).await;
        let full = resolve_matches_from_cache(&["NA1_1".to_string(), "NA1_2".to_string()])
            .await
            .unwrap();
        assert_eq!(full.len(), 2);
    }
}
