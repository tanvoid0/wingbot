use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;
use crate::data_processing::GameDataShaper;
use crate::types::GameSnapshot;
use crate::config::LeagueClientConfig;
use regex::Regex;

// LCU modules will be added later

// Constants for LCU API endpoints
pub const LCU_BASE_URL: &str = "https://127.0.0.1";
/// The Live Client Data API is HTTPS-only, behind Riot's self-signed
/// certificate. Plain HTTP on this port does not respond.
pub const LIVE_CLIENT_BASE_URL: &str = "https://127.0.0.1:2999";

// LCU API Endpoints
pub const LCU_CHAMP_SELECT_ENDPOINT: &str = "/lol-champ-select/v1/session";
pub const LCU_GAMEFLOW_SESSION_ENDPOINT: &str = "/lol-gameflow/v1/session";
pub const LCU_GAMEFLOW_PHASE_ENDPOINT: &str = "/lol-gameflow/v1/gameflow-phase";
pub const LCU_SUMMONER_ENDPOINT: &str = "/lol-summoner/v1/current-summoner";
pub const LCU_CHAMPIONS_ENDPOINT: &str = "/lol-champions/v1/champions";
pub const LCU_CHAMPION_BY_ID_ENDPOINT: &str = "/lol-champions/v1/champions";
pub const LCU_ITEMS_ENDPOINT: &str = "/lol-items/v1/items";
pub const LCU_RUNES_ENDPOINT: &str = "/lol-perks/v1/perks";
pub const LCU_SUMMONER_SPELLS_ENDPOINT: &str = "/lol-summoner/v1/summoner-spells";
pub const LCU_MATCH_HISTORY_ENDPOINT: &str = "/lol-match-history/v1/products/lol";
pub const LCU_RANKED_STATS_ENDPOINT: &str = "/lol-ranked/v1/current-ranked-stats";
pub const LCU_LOBBY_ENDPOINT: &str = "/lol-lobby/v2/lobby";
pub const LCU_READY_CHECK_ENDPOINT: &str = "/lol-lobby-team-builder/v1/ready-check";
pub const LCU_BUILDS_ENDPOINT: &str = "/system/v1/builds";
pub const LCU_PERK_PAGES_ENDPOINT: &str = "/lol-perks/v1/pages";
pub const LCU_PERK_CURRENT_PAGE_ENDPOINT: &str = "/lol-perks/v1/currentpage";
pub const LCU_PERK_INVENTORY_ENDPOINT: &str = "/lol-perks/v1/inventory";

/// Prefix marking a rune page as ours. Pages without it belong to the player
/// (or another tool) and are never deleted.
pub const WINGBOT_PAGE_PREFIX: &str = "WingBot: ";

// Live Client Data API Endpoints
pub const LIVE_CLIENT_ALL_GAME_DATA_ENDPOINT: &str = "/liveclientdata/allgamedata";
pub const LIVE_CLIENT_GAMESTATS_ENDPOINT: &str = "/liveclientdata/gamestats";
pub const LIVE_CLIENT_PLAYER_SCORES_ENDPOINT: &str = "/liveclientdata/playerscores";
pub const LIVE_CLIENT_PLAYER_ITEMS_ENDPOINT: &str = "/liveclientdata/playeritems";
pub const LIVE_CLIENT_PLAYER_SUMMONER_SPELLS_ENDPOINT: &str = "/liveclientdata/playersummonerspells";
pub const LIVE_CLIENT_PLAYER_MAIN_RUNES_ENDPOINT: &str = "/liveclientdata/playermainrunes";
pub const LIVE_CLIENT_EVENT_DATA_ENDPOINT: &str = "/liveclientdata/eventdata";
pub const LIVE_CLIENT_ACTIVE_PLAYER_NAME_ENDPOINT: &str = "/liveclientdata/activeplayername";
pub const LIVE_CLIENT_PLAYER_LIST_ENDPOINT: &str = "/liveclientdata/playerlist";

// Authentication
pub const LCU_USERNAME: &str = "riot";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LCUConnection {
    pub port: u16,
    pub password: String,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionSelectState {
    pub phase: String,
    pub time_remaining: u64,
    pub my_team: Vec<PlayerInfo>,
    pub enemy_team: Vec<PlayerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub summoner_id: String,
    pub champion_id: Option<u64>,
    pub champion_name: Option<String>,
    pub position: Option<String>,
    pub is_me: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Champion {
    pub id: u64,
    pub name: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub stats: ChampionStats,
    pub abilities: Vec<ChampionAbility>,
    pub passive: ChampionPassive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionStats {
    pub hp: f64,
    pub hpperlevel: f64,
    pub mp: f64,
    pub mpperlevel: f64,
    pub movespeed: f64,
    pub armor: f64,
    pub armorperlevel: f64,
    pub spellblock: f64,
    pub spellblockperlevel: f64,
    pub attackrange: f64,
    pub hpregen: f64,
    pub hpregenperlevel: f64,
    pub mpregen: f64,
    pub mpregenperlevel: f64,
    pub crit: f64,
    pub critperlevel: f64,
    pub attackdamage: f64,
    pub attackdamageperlevel: f64,
    pub attackspeedperlevel: f64,
    pub attackspeed: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionAbility {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tooltip: String,
    pub cooldown: Vec<f64>,
    pub cost: Vec<f64>,
    pub range: Vec<f64>,
    pub maxrank: u8,
    pub effect: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionPassive {
    pub name: String,
    pub description: String,
    pub image: ChampionImage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionImage {
    pub full: String,
    pub sprite: String,
    pub group: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub plaintext: String,
    pub gold: u64,
    pub tags: Vec<String>,
    pub stats: std::collections::HashMap<String, f64>,
    pub image: ItemImage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemImage {
    pub full: String,
    pub sprite: String,
    pub group: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rune {
    pub id: u64,
    pub name: String,
    pub icon: String,
    pub short_desc: String,
    pub long_desc: String,
    pub slots: Vec<RuneSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuneSlot {
    pub runes: Vec<RuneDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuneDetail {
    pub id: u32,
    pub key: String,
    pub icon: String,
    pub name: String,
    pub short_desc: String,
    pub long_desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummonerSpell {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub tooltip: String,
    pub cooldown: Vec<f64>,
    pub cost: Vec<f64>,
    pub range: Vec<f64>,
    pub maxrank: u8,
    pub effect: Vec<f64>,
    pub image: SpellImage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellImage {
    pub full: String,
    pub sprite: String,
    pub group: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct LCUAPI {
    pub connection: Option<LCUConnection>,
    pub client: reqwest::Client,
    live_client: reqwest::Client,
}

impl LCUAPI {
    pub fn new() -> Self {
        Self {
            connection: None,
            client: reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            live_client: reqwest::Client::builder()
                // Riot serves the Live Client Data API over a self-signed cert.
                .danger_accept_invalid_certs(true)
                .timeout(Duration::from_secs(3))
                .build()
                .unwrap(),
        }
    }

    async fn live_client_get(&self, path: &str) -> Result<reqwest::Response, String> {
        let url = format!("{}{}", LIVE_CLIENT_BASE_URL, path);
        self.live_client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to request {url}: {e}"))
    }

    /// Per-player Live Client endpoints identify a player by `riotId`
    /// (`Name#TAG`). The older `summonerName` parameter is rejected with 400.
    async fn live_client_get_for_player(
        &self,
        path: &str,
        riot_id: &str,
    ) -> Result<reqwest::Response, String> {
        let url = format!("{}{}", LIVE_CLIENT_BASE_URL, path);
        self.live_client
            .get(&url)
            .query(&[("riotId", riot_id)])
            .send()
            .await
            .map_err(|e| format!("Failed to request {url}: {e}"))
    }

    /// Riot ID of the player whose client this is. Unavailable while
    /// spectating, which makes it the reliable spectator signal.
    pub async fn get_active_riot_id(&self) -> Result<String, String> {
        let response = self
            .live_client_get(LIVE_CLIENT_ACTIVE_PLAYER_NAME_ENDPOINT)
            .await?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to get active player name: HTTP {}",
                response.status()
            ));
        }

        let name: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse active player name: {}", e))?;

        name.as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .ok_or_else(|| "Active player name unavailable".to_string())
    }

    pub fn set_connection(&mut self, connection: LCUConnection) {
        self.connection = Some(connection);
    }

    // Helper method to make HTTP requests
    pub async fn make_request(&self, url: &str, method: &str, body: Option<serde_json::Value>) -> Result<reqwest::Response, String> {
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        
        let client = &self.client;
        let request = match method {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            _ => return Err(format!("Unsupported HTTP method: {}", method)),
        };
        
        let request = request.basic_auth(LCU_USERNAME, Some(&connection.password));
        
        if let Some(body_data) = body {
            request.json(&body_data).send().await
        } else {
            request.send().await
        }
        .map_err(|e| format!("Request failed: {}", e))
    }

    // Core LCU API methods
    pub async fn get_current_game_state(&self) -> Result<String, String> {
        self.resolve_game_phase().await
    }

    /// Reads phase from `/gameflow-phase`, falling back to `/session` when the phase
    /// endpoint is unavailable (common during GameStart and in-game).
    pub async fn resolve_game_phase(&self) -> Result<String, String> {
        let connection = self
            .connection
            .as_ref()
            .ok_or("No LCU connection established")?;

        let url = format!(
            "{}:{}{}",
            LCU_BASE_URL, connection.port, LCU_GAMEFLOW_PHASE_ENDPOINT
        );
        if let Ok(response) = self.make_request(&url, "GET", None).await {
            if response.status().is_success() {
                let phase: String = response
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse game state: {}", e))?;
                return Ok(phase);
            }
        }

        let session = self.get_game_flow_session().await?;
        session
            .get("phase")
            .and_then(|p| p.as_str())
            .map(str::to_string)
            .ok_or_else(|| "Game flow session missing phase".to_string())
    }

    /// Live patch as `major.minor` (e.g. `16.17`), read from the running
    /// client build rather than hardcoded — a stale patch silently degrades
    /// every AI recommendation.
    pub async fn get_patch_version(&self) -> Option<String> {
        let connection = self.connection.as_ref()?;
        let url = format!("{}:{}{}", LCU_BASE_URL, connection.port, LCU_BUILDS_ENDPOINT);
        let response = self.make_request(&url, "GET", None).await.ok()?;
        if !response.status().is_success() {
            return None;
        }
        let body: serde_json::Value = response.json().await.ok()?;
        let version = body.get("version")?.as_str()?;

        let mut parts = version.split('.');
        let major = parts.next()?;
        let minor = parts.next()?;
        if major.is_empty() || minor.is_empty() {
            return None;
        }
        Some(format!("{major}.{minor}"))
    }

    /// Queue of the current game, as `(id, slug)`.
    pub async fn get_current_queue(&self) -> Option<(u64, String)> {
        let session = self.get_game_flow_session().await.ok()?;
        let queue = session.get("gameData")?.get("queue")?;
        let id = queue.get("id")?.as_u64()?;
        Some((id, queue_slug(id)))
    }

    pub async fn get_game_flow_session(&self) -> Result<serde_json::Value, String> {
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        
        let url = format!("{}:{}{}", LCU_BASE_URL, connection.port, LCU_GAMEFLOW_SESSION_ENDPOINT);
        let response = self.make_request(&url, "GET", None).await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to get game flow session: HTTP {}", response.status()));
        }
        
        let session: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse game flow session: {}", e))?;
        
        Ok(session)
    }

    pub async fn is_live_client_available(&self) -> bool {
        match self
            .live_client_get(LIVE_CLIENT_ALL_GAME_DATA_ENDPOINT)
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    pub async fn detect_spectator_mode(&self) -> Result<bool, String> {
        println!("[LCU] Detecting spectator mode...");
        
        // Method 0: Check game flow session for spectator-specific phases
        match self.get_game_flow_session().await {
            Ok(session) => {
                if let Some(phase) = session.get("phase").and_then(|p| p.as_str()) {
                    println!("[LCU] Game flow phase: {}", phase);
                    // Check for spectator-specific phases
                    if phase == "Spectator" || phase == "WatchingGame" || phase.contains("Spectate") {
                        println!("[LCU] ✓ Spectator mode detected - Game flow phase is {}", phase);
                        return Ok(true);
                    }
                }
            }
            Err(e) => {
                println!("[LCU] Failed to get game flow session: {}", e);
            }
        }
        
        // Method 1: Check if we're in game but live client is not available
        let live_client_available = self.is_live_client_available().await;
        println!("[LCU] Live client available: {}", live_client_available);
        
        if !live_client_available {
            if let Ok(current_state) = self.resolve_game_phase().await {
                println!("[LCU] Current game state: {}", current_state);
                if current_state == "Spectator" || current_state == "WatchingGame" {
                    println!("[LCU] ✓ Spectator mode detected - phase is {}", current_state);
                    return Ok(true);
                }
            }
        }

        // Method 2: Try to get all game data and check for spectator-specific errors
        println!("[LCU] Trying to get all game data...");
        match self.get_all_game_data().await {
            Ok(data) => {
                println!("[LCU] Got all game data, checking for spectator mode indicators...");
                
                // Check if activePlayer has spectator mode error
                if let Some(active_player) = data.get("activePlayer") {
                    if let Some(error) = active_player.get("error").and_then(|e| e.as_str()) {
                        if error.contains("Spectator mode") {
                            println!("[LCU] ✓ Spectator mode detected - activePlayer error: {}", error);
                            return Ok(true);
                        }
                    }
                }
                
                // Check if we have game data but no active player data (spectator mode)
                if data.get("allPlayers").is_some() && data.get("activePlayer").is_none() {
                    println!("[LCU] ✓ Spectator mode detected - has allPlayers but no activePlayer");
                    return Ok(true);
                }
                
                // Check if we have allPlayers but activePlayer is null/empty
                if let Some(all_players) = data.get("allPlayers") {
                    if all_players.is_array() && !all_players.as_array().unwrap().is_empty() {
                        if data.get("activePlayer").is_none() || data.get("activePlayer").unwrap().is_null() {
                            println!("[LCU] ✓ Spectator mode detected - has allPlayers but activePlayer is null");
                            return Ok(true);
                        }
                    }
                }
            }
            Err(e) => {
                println!("[LCU] Failed to get all game data: {}", e);
            }
        }

        // Method 3: game stats are served to spectators, but the active player
        // is not — a spectating client has no "own" player to report.
        println!("[LCU] Trying to get live game stats...");
        match self.get_live_game_stats().await {
            Ok(_) => {
                println!("[LCU] Got live game stats, checking for an active player...");
                match self.get_active_riot_id().await {
                    Err(e) => {
                        println!("[LCU] ✓ Spectator mode detected - no active player ({})", e);
                        return Ok(true);
                    }
                    Ok(riot_id) => {
                        println!("[LCU] Active player is {}, not spectator mode", riot_id);
                    }
                }
            }
            Err(e) => {
                println!("[LCU] Failed to get live game stats: {}", e);
            }
        }

        // Method 4: Check if we're in a game but can't access player-specific endpoints
        println!("[LCU] Checking player-specific endpoints...");
        match self.get_current_summoner().await {
            Ok(_) => {
                println!("[LCU] Can access current summoner, not spectator mode");
            }
            Err(_) => {
                // If we can't get current summoner but we're in a game, might be spectator mode
                match self.get_current_game_state().await {
                    Ok(state) => {
                        if state == "InProgress" {
                            println!("[LCU] ✓ Spectator mode detected - InProgress but can't access current summoner");
                            return Ok(true);
                        }
                    }
                    Err(_) => {}
                }
            }
        }

        println!("[LCU] No spectator mode detected");
        Ok(false)
    }

    pub async fn get_game_snapshot(&self) -> Result<GameSnapshot, String> {
        let lcu_connected = self.has_connection();
        let mut snapshot = GameSnapshot::disconnected();
        snapshot.lcu_connected = lcu_connected;

        if !lcu_connected {
            return Ok(snapshot);
        }

        let game_phase = self
            .resolve_game_phase()
            .await
            .unwrap_or_else(|_| "Unknown".to_string());
        snapshot.game_phase = game_phase.clone();

        let live_client_available = self.is_live_client_available().await;
        snapshot.live_client_available = live_client_available;

        let is_spectator_mode = if game_phase == "InProgress" || (game_phase == "Unknown" && live_client_available) {
            self.detect_spectator_mode().await.unwrap_or(false)
        } else {
            false
        };
        snapshot.is_spectator_mode = is_spectator_mode;

        if let Ok(summoner) = self.get_current_summoner().await {
            snapshot.current_summoner = Some(summoner);
        }

        // LCU may return stale champ-select JSON after the game starts; ignore it then.
        let post_champ_select = matches!(
            snapshot.game_phase.as_str(),
            "InProgress" | "GameStart" | "EndOfGame" | "WaitingForStats" | "PreEndOfGame"
        );
        if !post_champ_select {
            if let Ok(cs) = self.get_champion_select_state().await {
                if Self::is_champion_select_session(&cs) {
                    snapshot.champion_select = Some(cs);
                    if snapshot.game_phase == "Unknown" {
                        snapshot.game_phase = "ChampSelect".to_string();
                    }
                }
            }
        }

        if snapshot.game_phase == "Unknown" {
            if live_client_available {
                snapshot.game_phase = "InProgress".to_string();
            } else if let Ok(session) = self.get_game_flow_session().await {
                if let Some(phase) = session.get("phase").and_then(|p| p.as_str()) {
                    if !phase.is_empty() {
                        snapshot.game_phase = phase.to_string();
                    }
                }
            }
        }

        if live_client_available && !is_spectator_mode {
            if let Ok(data) = self.get_all_game_data().await {
                snapshot.live_game = Some(data);
            }
            if let Ok(stats) = self.get_live_game_stats().await {
                snapshot.live_stats = Some(stats);
            }
            if let Ok(scores) = self.get_player_scores("").await {
                snapshot.player_scores = Some(scores);
            }
        } else if is_spectator_mode {
            if let Some(summoner) = snapshot.current_summoner.as_ref() {
                if let Some(puuid) = summoner.get("puuid").and_then(|v| v.as_str()) {
                    snapshot.spectator_game =
                        crate::spectator_service::fetch_spectator_game(puuid, self).await;
                }
            }
        }

        snapshot.timestamp = chrono::Utc::now().timestamp_millis() as u64;
        Ok(snapshot)
    }

    pub async fn get_comprehensive_game_state(&self) -> Result<GameSnapshot, String> {
        self.get_game_snapshot().await
    }

    pub async fn get_live_game_stats(&self) -> Result<serde_json::Value, String> {
        let response = self
            .live_client_get(LIVE_CLIENT_GAMESTATS_ENDPOINT)
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to get live game stats: HTTP {}", response.status()));
        }
        
        let stats: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse live game stats: {}", e))?;
        
        Ok(stats)
    }

    /// Scores for `riot_id`, or for the active player when it is empty.
    pub async fn get_player_scores(&self, riot_id: &str) -> Result<serde_json::Value, String> {
        let riot_id = if riot_id.trim().is_empty() {
            self.get_active_riot_id().await?
        } else {
            riot_id.to_string()
        };

        let response = self
            .live_client_get_for_player(LIVE_CLIENT_PLAYER_SCORES_ENDPOINT, &riot_id)
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to get player scores: HTTP {}", response.status()));
        }
        
        let scores: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse player scores: {}", e))?;
        
        Ok(scores)
    }

    pub async fn get_all_game_data(&self) -> Result<serde_json::Value, String> {
        let response = self
            .live_client_get(LIVE_CLIENT_ALL_GAME_DATA_ENDPOINT)
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to get all game data: HTTP {}", response.status()));
        }
        
        let data: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse all game data: {}", e))?;
        
        // Shape the data for better frontend processing
        let shaped_data = GameDataShaper::shape_live_client_data(&data)?;
        
        Ok(shaped_data)
    }


    // Original JSON-based methods (keeping for compatibility)
    pub async fn get_champions(&self) -> Result<serde_json::Value, String> {
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        
        let url = format!("{}:{}{}", LCU_BASE_URL, connection.port, LCU_CHAMPIONS_ENDPOINT);
        let response = self.make_request(&url, "GET", None).await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to fetch champions: HTTP {}", response.status()));
        }
        
        let champions: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse champions JSON: {}", e))?;
        
            Ok(champions)
    }

    pub async fn get_items(&self) -> Result<serde_json::Value, String> {
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        
        let url = format!("{}:{}{}", LCU_BASE_URL, connection.port, LCU_ITEMS_ENDPOINT);
        let response = self.make_request(&url, "GET", None).await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to fetch items: HTTP {}", response.status()));
        }
        
        let items: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse items JSON: {}", e))?;
        
            Ok(items)
    }

    pub async fn get_runes(&self) -> Result<serde_json::Value, String> {
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        
        let url = format!("{}:{}{}", LCU_BASE_URL, connection.port, LCU_RUNES_ENDPOINT);
        let response = self.make_request(&url, "GET", None).await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to fetch runes: HTTP {}", response.status()));
        }
        
        let runes: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse runes JSON: {}", e))?;
        
            Ok(runes)
    }

    pub async fn get_summoner_spells(&self) -> Result<serde_json::Value, String> {
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        
        let url = format!("{}:{}{}", LCU_BASE_URL, connection.port, LCU_SUMMONER_SPELLS_ENDPOINT);
        let response = self.make_request(&url, "GET", None).await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to fetch summoner spells: HTTP {}", response.status()));
        }
        
        let spells: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse summoner spells JSON: {}", e))?;
        
            Ok(spells)
    }

    // Additional methods needed for compatibility
    pub fn has_connection(&self) -> bool {
        self.connection.is_some()
    }

    /// LCU champ select session exposes phase under `timer.phase`, not at the root.
    pub fn is_champion_select_session(data: &serde_json::Value) -> bool {
        !data.is_null()
            && (data.get("myTeam").is_some()
                || data
                    .get("timer")
                    .and_then(|t| t.get("phase"))
                    .is_some())
    }

    pub fn get_connection_status(&self) -> String {
        if self.connection.is_some() {
            "Connected".to_string()
        } else {
            "League not running".to_string()
        }
    }

    /// Parses LCU/Riot lockfile content: `process:pid:port:password:protocol`.
    pub fn parse_lockfile_content(&self, content: &str) -> Result<LCUConnection, String> {
        Self::parse_lockfile_str(content)
    }

    fn parse_lockfile_str(content: &str) -> Result<LCUConnection, String> {
        let trimmed = content.trim();
        let parts: Vec<&str> = trimmed.split(':').collect();
        if parts.len() != 5 {
            return Err("Invalid lockfile format".to_string());
        }
        let port: u16 = parts[2]
            .parse()
            .map_err(|_| "Invalid port in lockfile".to_string())?;
        Ok(LCUConnection {
            port,
            password: parts[3].to_string(),
            protocol: parts[4].to_string(),
        })
    }

    /// `%ProgramData%`, via either env var Windows exposes for it.
    fn program_data_dir() -> Option<PathBuf> {
        std::env::var("ProgramData")
            .or_else(|_| std::env::var("ALLUSERSPROFILE"))
            .ok()
            .map(PathBuf::from)
    }

    /// Extracts League install directories from Riot's installation metadata.
    ///
    /// `installs_json` is `%ProgramData%\Riot Games\RiotClientInstalls.json`,
    /// whose `associated_client` map is keyed by each game's install directory.
    /// `product_settings_yaml` is the per-product settings file, which carries
    /// the resolved path in `product_install_full_path`. Riot writes these with
    /// forward slashes and a trailing separator.
    pub(crate) fn parse_install_dirs(
        installs_json: Option<&str>,
        product_settings_yaml: Option<&str>,
    ) -> Vec<PathBuf> {
        let mut seen = HashSet::new();
        let mut dirs = Vec::new();

        let mut push = |raw: &str| {
            let normalized = raw.trim().replace('/', "\\");
            let normalized = normalized.trim_end_matches('\\');
            if normalized.is_empty() {
                return;
            }
            if seen.insert(normalized.to_lowercase()) {
                dirs.push(PathBuf::from(normalized));
            }
        };

        if let Some(content) = installs_json {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
                if let Some(clients) = json.get("associated_client").and_then(|v| v.as_object()) {
                    for install_dir in clients.keys() {
                        push(install_dir);
                    }
                }
            }
        }

        if let Some(content) = product_settings_yaml {
            let re = Regex::new(r#"(?m)^\s*product_install_full_path:\s*"?([^"\r\n]+?)"?\s*$"#)
                .expect("static regex");
            if let Some(path) = re.captures(content).and_then(|c| c.get(1)) {
                push(path.as_str());
            }
        }

        dirs
    }

    /// League install directories, resolved from Riot's own installation
    /// metadata. This is how third-party clients locate an install on a
    /// non-default drive; hardcoded paths only cover a default install.
    pub(crate) fn league_install_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        if let Some(program_data) = Self::program_data_dir() {
            let riot = program_data.join("Riot Games");
            let installs = std::fs::read_to_string(riot.join("RiotClientInstalls.json")).ok();
            let product_settings = std::fs::read_to_string(
                riot.join("Metadata")
                    .join("league_of_legends.live")
                    .join("league_of_legends.live.product_settings.yaml"),
            )
            .ok();
            dirs = Self::parse_install_dirs(installs.as_deref(), product_settings.as_deref());
        }

        // Default-install fallbacks.
        let mut fallbacks: Vec<PathBuf> = vec![
            PathBuf::from(r"C:\Riot Games\League of Legends"),
            PathBuf::from(r"D:\Riot Games\League of Legends"),
        ];
        for var in ["ProgramFiles", "ProgramFiles(x86)"] {
            if let Ok(pf) = std::env::var(var) {
                fallbacks.push(PathBuf::from(pf).join("Riot Games").join("League of Legends"));
            }
        }
        for path in fallbacks {
            if !dirs.iter().any(|d| d == &path) {
                dirs.push(path);
            }
        }

        dirs
    }

    /// Candidate LCU lockfiles. Deliberately excludes the Riot Client lockfile
    /// under `%LOCALAPPDATA%\Riot Games\Riot Client\Config` — that is a
    /// different service on a different port, and its credentials 404 on every
    /// `/lol-*` endpoint.
    fn league_lockfile_paths() -> Vec<PathBuf> {
        Self::league_install_dirs()
            .into_iter()
            .map(|dir| dir.join("lockfile"))
            .collect()
    }

    pub(crate) fn parse_league_client_command_line(cmdline: &str) -> Result<LCUConnection, String> {
        // Must not match `--riotclient-app-port`; only the League `--app-port` arg.
        let port_re = Regex::new(r"(?:^|\s)--app-port=(\d+)").unwrap();
        let token_re = Regex::new(r#"(?:^|\s)--(?:remoting-auth-token|app-password)=([^\s"']+)"#).unwrap();
        
        let port: u16 = port_re
            .captures(cmdline)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse().ok())
            .ok_or_else(|| "Port not found in League client command line".to_string())?;
            
        let password = token_re
            .captures(cmdline)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| "Auth token not found in League client command line".to_string())?;
            
        if port == 0 || password.is_empty() {
            return Err("Invalid LCU credentials from process".to_string());
        }
        
        Ok(LCUConnection {
            port,
            password,
            protocol: "https".to_string(),
        })
    }

    /// Runs a console command without flashing a window on the user's screen.
    #[cfg(target_os = "windows")]
    fn hidden_command(program: &str) -> std::process::Command {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let mut cmd = std::process::Command::new(program);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd
    }

    /// Fallback only. `LeagueClientUx.exe` exits once a game starts, and its
    /// command line is unreadable while the anti-cheat protects the process,
    /// so this cannot be the primary source.
    #[cfg(target_os = "windows")]
    fn detect_via_windows_process_inspection() -> Result<LCUConnection, String> {
        let ps_cmd = "Get-CimInstance Win32_Process -Filter \"Name = 'LeagueClientUx.exe'\" | Select-Object -ExpandProperty CommandLine";
        let output = Self::hidden_command("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", ps_cmd])
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                if let Ok(conn) = Self::parse_league_client_command_line(&stdout) {
                    return Ok(conn);
                }
            }
        }

        Err("LeagueClientUx.exe process not found or command line inaccessible".to_string())
    }

    fn detect_league_client_blocking() -> Result<LCUConnection, String> {
        // Priority 1: the lockfile the client writes into its install directory.
        // It exists for the whole client lifetime — including once a game has
        // started and `LeagueClientUx.exe` is gone — and needs no subprocess.
        for path in Self::league_lockfile_paths() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(conn) = Self::parse_lockfile_str(&content) {
                    return Ok(conn);
                }
            }
        }

        // Priority 2: process command line, for an install Riot's metadata
        // does not list (portable copy, PBE, non-standard deployment).
        #[cfg(target_os = "windows")]
        {
            if let Ok(conn) = Self::detect_via_windows_process_inspection() {
                return Ok(conn);
            }

            // Priority 3: locate the install via the running process path.
            let ps_path_cmd = "(Get-Process LeagueClientUx -ErrorAction SilentlyContinue | Select-Object -First 1).Path";
            if let Ok(output) = Self::hidden_command("powershell")
                .args(["-NoProfile", "-NonInteractive", "-Command", ps_path_cmd])
                .output()
            {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() {
                    let mut lockfile_path = PathBuf::from(path_str);
                    lockfile_path.pop(); // Remove LeagueClientUx.exe
                    lockfile_path.push("lockfile");

                    if let Ok(content) = std::fs::read_to_string(&lockfile_path) {
                        if let Ok(conn) = Self::parse_lockfile_str(&content) {
                            return Ok(conn);
                        }
                    }
                }
            }
        }

        Err("League client not detected. Please ensure League of Legends is running.".to_string())
    }


    pub async fn detect_league_client(&self) -> Result<LCUConnection, String> {
        tokio::task::spawn_blocking(Self::detect_league_client_blocking)
            .await
            .map_err(|e| format!("LCU detection task failed: {}", e))?
    }

    pub async fn detect_connection_with_config(
        &self,
        config: &LeagueClientConfig,
    ) -> Result<LCUConnection, String> {
        if let (Some(port), Some(password)) = (config.manual_port, &config.manual_password) {
            return Ok(LCUConnection {
                port,
                password: password.clone(),
                protocol: "https".to_string(),
            });
        }
        if config.auto_detect {
            return self.detect_league_client().await;
        }
        Err("League client auto-detect is disabled and no manual connection configured".to_string())
    }

    pub async fn detect_connection(&self) -> Result<LCUConnection, String> {
        self.detect_league_client().await
    }

    pub async fn get_champion_select_state(&self) -> Result<serde_json::Value, String> {
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        
        let url = format!("{}:{}{}", LCU_BASE_URL, connection.port, LCU_CHAMP_SELECT_ENDPOINT);
        let response = self.make_request(&url, "GET", None).await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to get champion select state: HTTP {}", response.status()));
        }
        
        let state: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse champion select state: {}", e))?;
        
        Ok(state)
    }

    pub async fn get_live_game_data(&self) -> Result<serde_json::Value, String> {
        self.get_all_game_data().await
    }

    pub async fn get_league_status(&self) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "connected": self.has_connection(),
            "status": self.get_connection_status()
        }))
    }

    pub async fn is_league_running(&self) -> bool {
        if self.has_connection() {
            return true;
        }
        tokio::task::spawn_blocking(|| {
            Self::league_lockfile_paths().iter().any(|p| {
                std::fs::read_to_string(p)
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false)
            })
        })
        .await
        .unwrap_or(false)
    }

    // Placeholder methods for websocket compatibility
    pub async fn get_champion_select_raw(&self) -> Result<serde_json::Value, String> {
        self.get_champion_select_state().await
    }

    /// `/liveclientdata/playeritems` serves one player at a time. The player
    /// list already carries every player's items, so read them from there
    /// instead of issuing ten per-player requests.
    pub async fn get_all_players_items(&self) -> Result<serde_json::Value, String> {
        Ok(Self::collect_from_player_list(
            &self.get_player_list().await?,
            "items",
        ))
    }

    /// Maps each player's Riot ID to `field` from the live player list.
    pub(crate) fn collect_from_player_list(players: &serde_json::Value, field: &str) -> serde_json::Value {
        let mut out = serde_json::Map::new();
        if let Some(list) = players.as_array() {
            for player in list {
                let riot_id = player
                    .get("riotId")
                    .and_then(|v| v.as_str())
                    .or_else(|| player.get("summonerName").and_then(|v| v.as_str()));
                if let (Some(riot_id), Some(value)) = (riot_id, player.get(field)) {
                    out.insert(riot_id.to_string(), value.clone());
                }
            }
        }
        serde_json::Value::Object(out)
    }

    pub async fn get_current_summoner_match_history(&self) -> Result<serde_json::Value, String> {
        let summoner = self.get_current_summoner().await?;
        let puuid = summoner
            .get("puuid")
            .and_then(|v| v.as_str())
            .ok_or("Current summoner has no puuid")?;
        self.get_match_history(puuid, crate::match_history_service::DEFAULT_MATCH_COUNT)
            .await
    }

    pub async fn get_ranked_stats(&self) -> Result<serde_json::Value, String> {
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;

        let url = format!("{}:{}{}", LCU_BASE_URL, connection.port, LCU_RANKED_STATS_ENDPOINT);
        let response = self.make_request(&url, "GET", None).await?;

        if !response.status().is_success() {
            return Err(format!("Failed to get ranked stats: HTTP {}", response.status()));
        }

        let stats: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse ranked stats: {}", e))?;

        Ok(stats)
    }

    /// See [`Self::get_all_players_items`] — same reasoning for runes.
    pub async fn get_all_players_runes(&self) -> Result<serde_json::Value, String> {
        Ok(Self::collect_from_player_list(
            &self.get_player_list().await?,
            "runes",
        ))
    }

    pub async fn analyze_rune_setup(&self, _player_name: &str) -> Result<serde_json::Value, String> {
        // Placeholder implementation
        Ok(serde_json::json!({
            "analysis": "Rune analysis not implemented yet"
        }))
    }

    pub async fn get_lobby_status(&self) -> Result<serde_json::Value, String> {
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        
        let url = format!("{}:{}{}", LCU_BASE_URL, connection.port, LCU_LOBBY_ENDPOINT);
        let response = self.make_request(&url, "GET", None).await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to get lobby status: HTTP {}", response.status()));
        }
        
        let lobby: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse lobby status: {}", e))?;
        
            Ok(lobby)
    }

    pub async fn get_ready_check(&self) -> Result<serde_json::Value, String> {
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        
        let url = format!("{}:{}{}", LCU_BASE_URL, connection.port, LCU_READY_CHECK_ENDPOINT);
        let response = self.make_request(&url, "GET", None).await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to get ready check: HTTP {}", response.status()));
        }
        
        let ready_check: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse ready check: {}", e))?;
        
            Ok(ready_check)
    }

    pub async fn analyze_lobby(&self) -> Result<serde_json::Value, String> {
        let lobby = self.get_lobby_status().await?;
        let members = lobby
            .get("members")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let total = members.len();
        let ready_members = members
            .iter()
            .filter(|m| m.get("ready").and_then(|v| v.as_bool()).unwrap_or(false))
            .count();

        let ready_percentage = if total > 0 {
            (ready_members as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let champion_ids: std::collections::HashSet<u64> = members
            .iter()
            .filter_map(|m| m.get("championId").and_then(|v| v.as_u64()))
            .filter(|id| *id > 0)
            .collect();

        let parsed_members: Vec<serde_json::Value> = members
            .iter()
            .map(|m| {
                let game_name = m
                    .get("gameName")
                    .or_else(|| m.get("summonerName"))
                    .and_then(|v| v.as_str());
                let tag_line = m.get("tagLine").and_then(|v| v.as_str());
                serde_json::json!({
                    "puuid": m.get("puuid").cloned().unwrap_or(serde_json::Value::Null),
                    "game_name": game_name,
                    "tag_line": tag_line,
                    "riot_id": match (game_name, tag_line) {
                        (Some(n), Some(t)) => serde_json::json!(format!("{}#{}", n, t)),
                        _ => serde_json::Value::Null,
                    },
                    "position": m.get("firstPositionPreference")
                        .or_else(|| m.get("position"))
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                    "ready": m.get("ready").and_then(|v| v.as_bool()).unwrap_or(false),
                    "is_leader": m.get("isLeader").and_then(|v| v.as_bool()).unwrap_or(false),
                })
            })
            .collect();

        Ok(serde_json::json!({
            "team_analysis": {
                "total_members": total,
                "ready_members": ready_members,
                "ready_percentage": ready_percentage,
                "champion_diversity": champion_ids.len(),
            },
            "members": parsed_members,
            "queue_id": lobby
                .get("gameConfig")
                .and_then(|c| c.get("queueId"))
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        }))
    }

    pub async fn check_connection_health(&self) -> Result<serde_json::Value, String> {
        let lcu_healthy = if self.has_connection() {
            self.get_current_summoner().await.is_ok()
        } else {
            false
        };
        let live_client_healthy = self.is_live_client_available().await;
        let overall_healthy = lcu_healthy || live_client_healthy;

        Ok(serde_json::json!({
            "healthy": overall_healthy,
            "lcu_connected": lcu_healthy,
            "live_client_available": live_client_healthy,
            "status": if overall_healthy { "Connected" } else { "Disconnected" }
        }))
    }

    // Additional missing methods for lib.rs compatibility
    pub async fn get_current_summoner(&self) -> Result<serde_json::Value, String> {
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        
        let url = format!("{}:{}{}", LCU_BASE_URL, connection.port, LCU_SUMMONER_ENDPOINT);
        let response = self.make_request(&url, "GET", None).await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to get current summoner: HTTP {}", response.status()));
        }
        
        let summoner: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse current summoner: {}", e))?;
        
        Ok(summoner)
    }

    pub async fn get_champion_by_id(&self, champion_id: u64) -> Result<serde_json::Value, String> {
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        
        let url = format!("{}:{}{}/{}", LCU_BASE_URL, connection.port, LCU_CHAMPION_BY_ID_ENDPOINT, champion_id);
        let response = self.make_request(&url, "GET", None).await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to get champion by id: HTTP {}", response.status()));
        }
        
        let champion: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse champion: {}", e))?;
        
        Ok(champion)
    }

    pub async fn get_match_history(
        &self,
        puuid: &str,
        count: u32,
    ) -> Result<serde_json::Value, String> {
        let count = crate::match_history_service::clamp_match_count(Some(count));
        let page_size = crate::match_history_service::LCU_MATCH_PAGE_SIZE;
        let mut all_games: Vec<serde_json::Value> = Vec::new();
        let mut beg_index = 0u32;

        while (all_games.len() as u32) < count {
            let remaining = count - all_games.len() as u32;
            let page_count = remaining.min(page_size);
            let end_index = beg_index + page_count - 1;

            let page = self
                .get_match_history_page(puuid, beg_index, end_index)
                .await?;
            let games = page
                .get("games")
                .and_then(|g| g.get("games"))
                .and_then(|g| g.as_array())
                .cloned()
                .unwrap_or_default();

            if games.is_empty() {
                break;
            }

            let page_len = games.len();
            all_games.extend(games);
            if (page_len as u32) < page_count {
                break;
            }

            beg_index += page_count;
        }

        Ok(serde_json::json!({
            "games": {
                "gameCount": all_games.len(),
                "games": all_games,
            }
        }))
    }

    async fn get_match_history_page(
        &self,
        puuid: &str,
        beg_index: u32,
        end_index: u32,
    ) -> Result<serde_json::Value, String> {
        let connection = self
            .connection
            .as_ref()
            .ok_or("No LCU connection established")?;

        let url = format!(
            "{}:{}{}/{}/matches?begIndex={}&endIndex={}",
            LCU_BASE_URL,
            connection.port,
            LCU_MATCH_HISTORY_ENDPOINT,
            puuid,
            beg_index,
            end_index
        );
        let response = self.make_request(&url, "GET", None).await?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to get match history: HTTP {}",
                response.status()
            ));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse match history: {}", e))
    }

    pub async fn get_lobby(&self) -> Result<serde_json::Value, String> {
        self.get_lobby_status().await
    }

    pub async fn get_game_events(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}{}", LIVE_CLIENT_BASE_URL, LIVE_CLIENT_EVENT_DATA_ENDPOINT);
        let response = self.client.get(&url).send().await
            .map_err(|e| format!("Failed to get game events: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to get game events: HTTP {}", response.status()));
        }
        
        let events: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse game events: {}", e))?;
        
            Ok(events)
    }

    pub async fn get_active_player_name(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}{}", LIVE_CLIENT_BASE_URL, LIVE_CLIENT_ACTIVE_PLAYER_NAME_ENDPOINT);
        let response = self.client.get(&url).send().await
            .map_err(|e| format!("Failed to get active player name: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to get active player name: HTTP {}", response.status()));
        }
        
        let name: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse active player name: {}", e))?;
        
            Ok(name)
    }

    /// Living/dead state for every player, with seconds until respawn.
    ///
    /// `respawnTimer` is non-zero only while a player is dead, and Riot serves
    /// it for the whole lobby — including enemies.
    pub async fn get_death_timers(&self) -> Result<serde_json::Value, String> {
        let players = self.get_player_list().await?;
        let players = players.as_array().ok_or("Player list was not an array")?;

        let my_riot_id = self.get_active_riot_id().await.ok();
        let my_team = my_riot_id.as_deref().and_then(|name| {
            players
                .iter()
                .find(|p| p.get("riotId").and_then(|v| v.as_str()) == Some(name))
                .and_then(|p| p.get("team").and_then(|v| v.as_str()))
        });

        let mut allies = Vec::new();
        let mut enemies = Vec::new();

        for player in players {
            let team = player.get("team").and_then(|v| v.as_str());
            let respawn = player
                .get("respawnTimer")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let entry = serde_json::json!({
                "riot_id": player.get("riotId").and_then(|v| v.as_str()).unwrap_or(""),
                "champion": player.get("championName").and_then(|v| v.as_str()).unwrap_or(""),
                "position": player.get("position").and_then(|v| v.as_str()).unwrap_or(""),
                "level": player.get("level").and_then(|v| v.as_u64()).unwrap_or(0),
                "is_dead": player.get("isDead").and_then(|v| v.as_bool()).unwrap_or(false),
                "respawn_in_sec": (respawn * 10.0).round() / 10.0,
            });

            match (my_team, team) {
                (Some(mine), Some(t)) if mine == t => allies.push(entry),
                (Some(_), Some(_)) => enemies.push(entry),
                _ => allies.push(entry),
            }
        }

        Ok(serde_json::json!({ "allies": allies, "enemies": enemies }))
    }

    /// Kill and objective feed, newest last.
    pub async fn get_game_event_feed(&self) -> Result<serde_json::Value, String> {
        let raw = self.get_game_events().await?;
        let events = raw
            .get("Events")
            .and_then(|v| v.as_array())
            .ok_or("Event data had no Events array")?;

        let feed: Vec<serde_json::Value> = events
            .iter()
            .map(|e| {
                let mut entry = serde_json::Map::new();
                let copy = |key: &str, out: &mut serde_json::Map<String, serde_json::Value>| {
                    if let Some(v) = e.get(key) {
                        if !v.is_null() {
                            out.insert(key.to_string(), v.clone());
                        }
                    }
                };
                for key in [
                    "EventID", "EventName", "EventTime", "KillerName", "VictimName",
                    "Assisters", "DragonType", "Stolen", "TurretKilled", "InhibKilled",
                    "KillStreak", "Acer", "AcingTeam", "Recipient", "WardType",
                ] {
                    copy(key, &mut entry);
                }
                serde_json::Value::Object(entry)
            })
            .collect();

        Ok(serde_json::json!({ "events": feed, "count": feed.len() }))
    }

    fn lcu_url(&self, path: &str) -> Result<String, String> {
        let connection = self.connection.as_ref().ok_or("No LCU connection established")?;
        Ok(format!("{}:{}{}", LCU_BASE_URL, connection.port, path))
    }

    pub async fn get_rune_pages(&self) -> Result<serde_json::Value, String> {
        let url = self.lcu_url(LCU_PERK_PAGES_ENDPOINT)?;
        let response = self.make_request(&url, "GET", None).await?;
        if !response.status().is_success() {
            return Err(format!("Failed to get rune pages: HTTP {}", response.status()));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse rune pages: {}", e))
    }

    pub async fn get_current_rune_page(&self) -> Result<serde_json::Value, String> {
        let url = self.lcu_url(LCU_PERK_CURRENT_PAGE_ENDPOINT)?;
        let response = self.make_request(&url, "GET", None).await?;
        if !response.status().is_success() {
            return Err(format!("Failed to get current rune page: HTTP {}", response.status()));
        }
        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse current rune page: {}", e))
    }

    async fn delete_rune_page(&self, page_id: u64) -> Result<(), String> {
        let url = self.lcu_url(&format!("{}/{}", LCU_PERK_PAGES_ENDPOINT, page_id))?;
        let response = self.make_request(&url, "DELETE", None).await?;
        if !response.status().is_success() {
            return Err(format!("Failed to delete rune page {}: HTTP {}", page_id, response.status()));
        }
        Ok(())
    }

    /// Writes a rune page to the player's account and selects it.
    ///
    /// Only pages this tool created (see [`WINGBOT_PAGE_PREFIX`]) are ever
    /// removed to make room — a player's own pages are never touched. If the
    /// account is at its page limit with none of ours to reclaim, this fails
    /// rather than deleting something the player owns.
    pub async fn apply_rune_page(
        &self,
        name: &str,
        primary_style_id: u64,
        sub_style_id: u64,
        selected_perk_ids: &[u64],
    ) -> Result<serde_json::Value, String> {
        if primary_style_id == 0 || sub_style_id == 0 {
            return Err("Rune page needs both a primary and a secondary tree".to_string());
        }
        if selected_perk_ids.is_empty() {
            return Err("Rune page needs at least one selected perk".to_string());
        }
        if primary_style_id == sub_style_id {
            return Err("Primary and secondary rune trees must differ".to_string());
        }

        let page_name = if name.starts_with(WINGBOT_PAGE_PREFIX) {
            name.to_string()
        } else {
            format!("{}{}", WINGBOT_PAGE_PREFIX, name)
        };

        // Reclaim our own previous pages before adding a new one.
        if let Ok(pages) = self.get_rune_pages().await {
            if let Some(list) = pages.as_array() {
                for page in list {
                    let is_ours = page
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(|n| n.starts_with(WINGBOT_PAGE_PREFIX))
                        .unwrap_or(false);
                    let deletable = page
                        .get("isDeletable")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if is_ours && deletable {
                        if let Some(id) = page.get("id").and_then(|v| v.as_u64()) {
                            let _ = self.delete_rune_page(id).await;
                        }
                    }
                }
            }
        }

        let can_add = self
            .make_request(&self.lcu_url(LCU_PERK_INVENTORY_ENDPOINT)?, "GET", None)
            .await
            .ok()
            .and_then(|r| if r.status().is_success() { Some(r) } else { None });
        if let Some(response) = can_add {
            if let Ok(inv) = response.json::<serde_json::Value>().await {
                if inv.get("canAddCustomPage").and_then(|v| v.as_bool()) == Some(false) {
                    return Err(
                        "No free rune page slot. Delete a rune page in the client and retry —                          WingBot will not remove pages you created."
                            .to_string(),
                    );
                }
            }
        }

        let body = serde_json::json!({
            "name": page_name,
            "primaryStyleId": primary_style_id,
            "subStyleId": sub_style_id,
            "selectedPerkIds": selected_perk_ids,
            "current": true,
        });

        let url = self.lcu_url(LCU_PERK_PAGES_ENDPOINT)?;
        let response = self.make_request(&url, "POST", Some(body)).await?;
        let status = response.status();
        if !status.is_success() {
            let detail = response.text().await.unwrap_or_default();
            return Err(format!("Failed to create rune page: HTTP {} {}", status, detail));
        }

        let created: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse created rune page: {}", e))?;

        // `current: true` on create is usually honoured; select explicitly if not.
        if created.get("current").and_then(|v| v.as_bool()) != Some(true) {
            if let Some(id) = created.get("id").and_then(|v| v.as_u64()) {
                let url = self.lcu_url(LCU_PERK_CURRENT_PAGE_ENDPOINT)?;
                let _ = self.make_request(&url, "PUT", Some(serde_json::json!(id))).await;
            }
        }

        Ok(created)
    }

    pub async fn get_player_list(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}{}", LIVE_CLIENT_BASE_URL, LIVE_CLIENT_PLAYER_LIST_ENDPOINT);
        let response = self.client.get(&url).send().await
            .map_err(|e| format!("Failed to get player list: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to get player list: HTTP {}", response.status()));
        }
        
        let players: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse player list: {}", e))?;
        
        Ok(players)
    }

    pub async fn detect_connection_with_retry(&self, max_attempts: u32) -> Result<LCUConnection, String> {
        for attempt in 1..=max_attempts {
            match self.detect_connection().await {
                Ok(connection) => return Ok(connection),
                Err(e) => {
                    if attempt == max_attempts {
                        return Err(format!("Failed to detect connection after {} attempts: {}", max_attempts, e));
                    }
                    // Wait before retry
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                }
            }
        }
        Err("Failed to detect connection".to_string())
    }

    pub async fn set_connection_with_websocket(&self, connection: LCUConnection) -> Result<(), String> {
        // Connection is stored on the shared LCUAPI instance by the caller.
        let _ = connection;
        Ok(())
    }

    pub async fn get_connection_health_info(&self) -> Result<serde_json::Value, String> {
        let mut v = self.check_connection_health().await?;
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if let Some(obj) = v.as_object_mut() {
            obj.insert("last_check".to_string(), serde_json::json!(secs));
            if let Some(h) = obj.get("healthy").cloned() {
                obj.insert("is_healthy".to_string(), h);
            }
            obj.entry("consecutive_failures")
                .or_insert(serde_json::json!(0));
        }
        Ok(v)
    }

    pub async fn get_websocket_status(&self) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "connected": self.has_connection(),
            "status": self.get_connection_status()
        }))
    }

    pub async fn disconnect_websocket(&self) -> Result<(), String> {
        // Placeholder for websocket disconnection
        Ok(())
    }
}

/// Maps a Riot queue id to the slug the model is trained on.
pub fn queue_slug(queue_id: u64) -> String {
    match queue_id {
        420 => "ranked_solo",
        440 => "ranked_flex",
        400 => "draft_pick",
        430 => "blind_pick",
        450 => "aram",
        490 => "quickplay",
        700 | 720 => "clash",
        1700 | 1710 => "arena",
        1900 => "urf",
        _ => "unknown",
    }
    .to_string()
}
