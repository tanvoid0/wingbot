use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const SCOUT_TTL: Duration = Duration::from_secs(600);

struct ScoutCacheEntry {
    profile: Value,
    fetched_at: Instant,
}

struct ScoutCacheInner {
    profiles: HashMap<String, ScoutCacheEntry>,
}

struct ScoutCache {
    inner: Mutex<ScoutCacheInner>,
}

impl ScoutCacheInner {
    fn new() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }

    fn get(&self, puuid: &str) -> Option<Value> {
        let entry = self.profiles.get(puuid)?;
        if entry.fetched_at.elapsed() > SCOUT_TTL {
            return None;
        }
        Some(entry.profile.clone())
    }

    fn store(&mut self, puuid: &str, profile: Value) {
        self.profiles.insert(
            puuid.to_string(),
            ScoutCacheEntry {
                profile,
                fetched_at: Instant::now(),
            },
        );
    }

    fn invalidate(&mut self, puuid: &str) {
        self.profiles.remove(puuid);
    }
}

static SCOUT_CACHE: OnceLock<ScoutCache> = OnceLock::new();

fn cache() -> &'static ScoutCache {
    SCOUT_CACHE.get_or_init(|| ScoutCache {
        inner: Mutex::new(ScoutCacheInner::new()),
    })
}

pub async fn get_cached_profile(puuid: &str) -> Option<Value> {
    let inner = cache().inner.lock().await;
    inner.get(puuid)
}

pub async fn store_profile(puuid: &str, profile: Value) {
    let mut inner = cache().inner.lock().await;
    inner.store(puuid, profile);
}

pub async fn invalidate_profile(puuid: &str) {
    let mut inner = cache().inner.lock().await;
    inner.invalidate(puuid);
}

/// Compact enemy intel for AI prompts (refreshed when scouting completes).
static SESSION_ENEMY_INTEL: OnceLock<Mutex<Option<Value>>> = OnceLock::new();

fn session_intel() -> &'static Mutex<Option<Value>> {
    SESSION_ENEMY_INTEL.get_or_init(|| Mutex::new(None))
}

pub async fn set_session_enemy_intel(intel: Value) {
    let mut guard = session_intel().lock().await;
    *guard = Some(intel);
}

pub async fn get_session_enemy_intel() -> Option<Value> {
    let guard = session_intel().lock().await;
    guard.clone()
}

pub async fn clear_session_enemy_intel() {
    let mut guard = session_intel().lock().await;
    *guard = None;
}
