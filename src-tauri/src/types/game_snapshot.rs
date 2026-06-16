use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Canonical game state snapshot shared between backend and frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub lcu_connected: bool,
    pub game_phase: String,
    pub live_client_available: bool,
    pub is_spectator_mode: bool,
    pub current_summoner: Option<Value>,
    pub champion_select: Option<Value>,
    pub live_game: Option<Value>,
    pub live_stats: Option<Value>,
    pub player_scores: Option<Value>,
    pub timestamp: u64,
}

impl GameSnapshot {
    pub fn disconnected() -> Self {
        Self {
            lcu_connected: false,
            game_phase: "None".to_string(),
            live_client_available: false,
            is_spectator_mode: false,
            current_summoner: None,
            champion_select: None,
            live_game: None,
            live_stats: None,
            player_scores: None,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}
