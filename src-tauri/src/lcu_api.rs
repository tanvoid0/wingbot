use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;

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
    pub bans: Vec<u64>,
    pub picks: Vec<PickInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub summoner_id: u64,
    pub summoner_name: String,
    pub champion_id: Option<u64>,
    pub champion_name: Option<String>,
    pub role: Option<String>,
    pub pick_order: Option<u8>,
    pub team: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickInfo {
    pub champion_id: u64,
    pub champion_name: String,
    pub team: u8,
    pub pick_order: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveGameData {
    pub game_mode: String,
    pub game_time: u64,
    pub players: Vec<LivePlayerInfo>,
    pub events: Vec<GameEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivePlayerInfo {
    pub summoner_name: String,
    pub champion_name: String,
    pub team: u8,
    pub level: u8,
    pub items: Vec<u32>,
    pub summoner_spells: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub data: serde_json::Value,
}

#[derive(Clone)]
pub struct LCUAPI {
    connection: Option<LCUConnection>,
    client: reqwest::Client,
}

impl LCUAPI {
    pub fn new() -> Self {
        // Create a client that accepts self-signed certificates for localhost
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
            
        Self {
            connection: None,
            client,
        }
    }

    pub fn set_connection(&mut self, connection: LCUConnection) {
        self.connection = Some(connection);
    }

    pub fn has_connection(&self) -> bool {
        self.connection.is_some()
    }

    pub fn detect_connection(&mut self) -> Result<LCUConnection, String> {
        // Try to find League client process and extract connection info
        if cfg!(target_os = "windows") {
            self.detect_windows_connection()
        } else {
            self.detect_unix_connection()
        }
    }

    fn detect_windows_connection(&self) -> Result<LCUConnection, String> {
        println!("[LCU] Starting League client detection...");
        
        // Use the proper lockfile method - this is how legitimate apps like Blitz.gg do it
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let user_lockfile = format!(r"{}\AppData\Local\Riot Games\Riot Client\Config\lockfile", user_profile);
        
        let mut lockfile_paths = vec![
            // Common League installation paths
            r"C:\Riot Games\League of Legends\lockfile".to_string(),
            r"C:\Program Files\Riot Games\League of Legends\lockfile".to_string(),
            r"C:\Program Files (x86)\Riot Games\League of Legends\lockfile".to_string(),
            // User-specific paths
            user_lockfile,
        ];

        // Try to find additional paths through smarter detection
        if let Ok(additional_paths) = self.find_additional_league_paths() {
            println!("[LCU] Found {} additional League installation paths", additional_paths.len());
            for path in additional_paths {
                let lockfile_path = format!(r"{}\lockfile", path);
                lockfile_paths.push(lockfile_path);
            }
        }

        println!("[LCU] Checking {} potential lockfile locations:", lockfile_paths.len());
        for (i, lockfile_path) in lockfile_paths.iter().enumerate() {
            println!("[LCU] {}. Checking: {}", i + 1, lockfile_path);
            
            if std::path::Path::new(lockfile_path).exists() {
                println!("[LCU] ✓ Lockfile found at: {}", lockfile_path);
                if let Ok(connection) = self.parse_lockfile(lockfile_path) {
                    println!("[LCU] ✓ Successfully parsed lockfile - Port: {}, Protocol: {}", connection.port, connection.protocol);
                    return Ok(connection);
                } else {
                    println!("[LCU] ✗ Failed to parse lockfile at: {}", lockfile_path);
                }
            } else {
                println!("[LCU] ✗ Lockfile not found at: {}", lockfile_path);
            }
        }

        Err("Could not find League client lockfile. Make sure League of Legends is running.".to_string())
    }

    fn find_additional_league_paths(&self) -> Result<Vec<String>, String> {
        println!("[LCU] Searching for additional League installation paths...");
        let mut paths = Vec::new();

        // Method 1: Check Windows Registry for Riot Games installations
        if let Ok(registry_paths) = self.find_league_paths_from_registry() {
            println!("[LCU] Found {} paths from registry", registry_paths.len());
            paths.extend(registry_paths);
        }

        // Method 2: Check desktop shortcuts
        if let Ok(shortcut_paths) = self.find_league_paths_from_shortcuts() {
            println!("[LCU] Found {} paths from shortcuts", shortcut_paths.len());
            paths.extend(shortcut_paths);
        }

        // Method 3: Check common drive locations
        if let Ok(drive_paths) = self.find_league_paths_from_drives() {
            println!("[LCU] Found {} paths from drive search", drive_paths.len());
            paths.extend(drive_paths);
        }

        // Remove duplicates
        paths.sort();
        paths.dedup();
        
        println!("[LCU] Total unique paths found: {}", paths.len());
        Ok(paths)
    }

    fn find_league_paths_from_registry(&self) -> Result<Vec<String>, String> {
        // This would require winreg crate, for now return empty
        // TODO: Implement registry search for Riot Games installations
        Ok(vec![])
    }

    fn find_league_paths_from_shortcuts(&self) -> Result<Vec<String>, String> {
        let paths = Vec::new();
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        
        // Common shortcut locations
        let shortcut_locations = vec![
            format!(r"{}\Desktop", user_profile),
            format!(r"{}\AppData\Roaming\Microsoft\Windows\Start Menu\Programs", user_profile),
            r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs".to_string(),
        ];

        for location in shortcut_locations {
            if let Ok(entries) = std::fs::read_dir(&location) {
                for entry in entries.flatten() {
                    if let Some(path) = entry.path().to_str() {
                        if path.to_lowercase().contains("league") || path.to_lowercase().contains("riot") {
                            println!("[LCU] Found potential shortcut: {}", path);
                            // TODO: Parse .lnk files to get target paths
                        }
                    }
                }
            }
        }

        Ok(paths)
    }

    fn find_league_paths_from_drives(&self) -> Result<Vec<String>, String> {
        let mut paths = Vec::new();
        
        // Check common drive letters for Riot Games folder
        let drives = vec!["C:", "D:", "E:", "F:", "G:"];
        let possible_folders = vec![
            "Riot Games\\League of Legends",
            "Program Files\\Riot Games\\League of Legends",
            "Program Files (x86)\\Riot Games\\League of Legends",
            "Games\\Riot Games\\League of Legends",
            "Steam\\steamapps\\common\\League of Legends",
        ];

        for drive in drives {
            for folder in &possible_folders {
                let full_path = format!(r"{}\{}", drive, folder);
                if std::path::Path::new(&full_path).exists() {
                    println!("[LCU] Found League installation at: {}", full_path);
                    paths.push(full_path);
                }
            }
        }

        Ok(paths)
    }

    fn parse_lockfile(&self, path: &str) -> Result<LCUConnection, String> {
        println!("[LCU] Parsing lockfile: {}", path);
        let lockfile_content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read lockfile at {}: {}", path, e))?;

        println!("[LCU] Lockfile content: {}", lockfile_content.trim());

        // Lockfile format: "name:pid:port:password:protocol"
        let parts: Vec<&str> = lockfile_content.trim().split(':').collect();
        
        if parts.len() < 5 {
            return Err(format!("Invalid lockfile format. Expected 5 parts, got {}. Content: {}", parts.len(), lockfile_content.trim()));
        }

        let port = parts[2].parse::<u16>()
            .map_err(|e| format!("Failed to parse port from lockfile: {}", e))?;
        
        let password = parts[3].to_string();
        let protocol = parts[4].to_string();

        println!("[LCU] Parsed connection - Port: {}, Protocol: {}", port, protocol);

        Ok(LCUConnection {
            port,
            password,
            protocol,
        })
    }

    fn detect_unix_connection(&self) -> Result<LCUConnection, String> {
        // TODO: Implement Unix/Linux detection
        Err("Unix/Linux detection not implemented yet".to_string())
    }


    pub async fn get_champion_select_state(&self) -> Result<ChampionSelectState, String> {
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        
        let url = format!("{}://127.0.0.1:{}/lol-champ-select/v1/session", 
                         connection.protocol, connection.port);
        
        println!("[LCU] Requesting champion select state from: {}", url);
        
        let response = timeout(
            Duration::from_secs(5),
            self.client
                .get(&url)
                .basic_auth("riot", Some(&connection.password))
                .send()
        ).await
        .map_err(|_| "Request timeout".to_string())?
        .map_err(|e| format!("Request failed: {}", e))?;

        println!("[LCU] Champion select response status: {}", response.status());

        if response.status().is_success() {
            let champ_select: ChampionSelectState = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            println!("[LCU] Successfully retrieved champion select state");
            Ok(champ_select)
        } else {
            Err(format!("Failed to get champion select state: {}", response.status()))
        }
    }

    pub async fn get_live_game_data(&self) -> Result<LiveGameData, String> {
        // This should get data from the Live Client API (port 2999), not LCU API
        let url = "https://127.0.0.1:2999/liveclientdata/allgamedata";
        
        println!("[LCU] Requesting live game data from: {}", url);
        
        let response = timeout(
            Duration::from_secs(5),
            self.client.get(url).send()
        ).await
        .map_err(|_| "Request timeout".to_string())?
        .map_err(|e| format!("Request failed: {}", e))?;

        println!("[LCU] Live game data response status: {}", response.status());

        if response.status().is_success() {
            // Parse the Live Client API response
            let raw_data: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            
            // Convert to our LiveGameData structure
            // For now, create a basic structure
            let game_data = LiveGameData {
                game_mode: raw_data.get("gameData")
                    .and_then(|gd| gd.get("gameMode"))
                    .and_then(|gm| gm.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                game_time: raw_data.get("gameData")
                    .and_then(|gd| gd.get("gameTime"))
                    .and_then(|gt| gt.as_f64())
                    .unwrap_or(0.0) as u64,
                players: vec![], // TODO: Parse players from raw_data
                events: vec![],  // TODO: Parse events from raw_data
            };
            
            println!("[LCU] ✓ Successfully retrieved live game data");
            Ok(game_data)
        } else {
            println!("[LCU] ✗ Failed to get live game data: {}", response.status());
            Err(format!("Failed to get live game data: {}", response.status()))
        }
    }

    pub fn is_league_running(&self) -> bool {
        // Check if lockfile exists - this is more reliable than process checking
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let user_lockfile = format!(r"{}\AppData\Local\Riot Games\Riot Client\Config\lockfile", user_profile);
        
        let lockfile_paths = vec![
            r"C:\Riot Games\League of Legends\lockfile",
            r"C:\Program Files\Riot Games\League of Legends\lockfile",
            r"C:\Program Files (x86)\Riot Games\League of Legends\lockfile",
            &user_lockfile,
        ];

        for lockfile_path in lockfile_paths {
            if std::path::Path::new(lockfile_path).exists() {
                return true;
            }
        }
        
        false
    }

    pub fn get_connection_status(&self) -> String {
        if self.connection.is_some() {
            "Connected".to_string()
        } else if self.is_league_running() {
            "League running, not connected".to_string()
        } else {
            "League not running".to_string()
        }
    }

    // Comprehensive game state that combines LCU and Live Client data
    pub async fn get_comprehensive_game_state(&self) -> Result<serde_json::Value, String> {
        println!("[LCU] Getting comprehensive game state...");
        
        let lcu_connected = self.connection.is_some();
        let live_client_available = self.is_live_client_available().await;
        
        println!("[LCU] LCU Connected: {}, Live Client Available: {}", lcu_connected, live_client_available);
        
        let mut state = serde_json::json!({
            "lcu_connected": lcu_connected,
            "live_client_available": live_client_available,
            "game_phase": "unknown",
            "champion_select": null,
            "live_game": null,
            "game_mode": "Unknown",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });
        
        // Try to get live game data if Live Client is available
        if live_client_available {
            println!("[LCU] Attempting to get live game data...");
            if let Ok(live_game) = self.get_live_game_data().await {
                println!("[LCU] ✓ Live game data retrieved successfully");
                state["game_phase"] = serde_json::json!("in_game");
                state["game_mode"] = serde_json::json!(live_game.game_mode);
                state["live_game"] = serde_json::to_value(live_game).unwrap_or(serde_json::Value::Null);
            } else {
                println!("[LCU] ✗ Failed to get live game data");
            }
        } else if lcu_connected {
            // Try to get champion select state first
            println!("[LCU] Attempting to get champion select state...");
            match self.get_champion_select_state().await {
                Ok(champ_select) => {
                    println!("[LCU] ✓ Champion select state retrieved successfully");
                    state["game_phase"] = serde_json::json!("champion_select");
                    state["champion_select"] = serde_json::to_value(champ_select).unwrap_or(serde_json::Value::Null);
                }
                Err(e) => {
                    println!("[LCU] ✗ Failed to get champion select state: {}", e);
                    
                    // Check if we're in spectator mode by trying to get game flow session
                    println!("[LCU] Live client not available, checking for spectator mode...");
                    
                    // First try to get the current game state
                    match self.get_current_game_state().await {
                        Ok(current_state) => {
                            println!("[LCU] Current game state: {}", current_state);
                            if let Some(phase) = current_state.as_str() {
                                match phase {
                                    "InProgress" => {
                                        println!("[LCU] ✓ Spectator mode detected - in game but live client not available");
                                        state["game_phase"] = serde_json::json!("in_game");
                                        state["live_client_available"] = serde_json::json!(false);
                                    }
                                    "ChampSelect" => {
                                        println!("[LCU] ✓ Champion select detected");
                                        state["game_phase"] = serde_json::json!("champion_select");
                                    }
                                    "Lobby" => {
                                        println!("[LCU] In lobby");
                                        state["game_phase"] = serde_json::json!("lobby");
                                    }
                                    "ReadyCheck" => {
                                        println!("[LCU] Ready check");
                                        state["game_phase"] = serde_json::json!("ready_check");
                                    }
                                    _ => {
                                        println!("[LCU] Unknown game state: {}", phase);
                                        state["game_phase"] = serde_json::json!(phase);
                                    }
                                }
                            } else {
                                println!("[LCU] Current game state is not a string: {:?}", current_state);
                            }
                        }
                        Err(e) => {
                            println!("[LCU] ✗ Failed to get current game state: {}", e);
                            
                            // Fallback to game flow session
                            match self.get_game_flow_session().await {
                                Ok(session) => {
                                    println!("[LCU] Game flow session: {:?}", session);
                                    if let Some(phase) = session.get("phase").and_then(|p| p.as_str()) {
                                        println!("[LCU] Game flow phase: {}", phase);
                                        if phase == "InProgress" {
                                            println!("[LCU] ✓ Spectator mode detected - in game but live client not available");
                                            state["game_phase"] = serde_json::json!("in_game");
                                            state["live_client_available"] = serde_json::json!(false);
                                        } else {
                                            println!("[LCU] Game flow phase is not InProgress: {}", phase);
                                        }
                                    } else {
                                        println!("[LCU] No phase field found in game flow session");
                                    }
                                }
                                Err(e2) => {
                                    println!("[LCU] ✗ Failed to get game flow session: {}", e2);
                                    println!("[LCU] Both game state detection methods failed - setting phase to unknown");
                                }
                            }
                        }
                    }
                }
            }
        }
        
        println!("[LCU] Final game state: {}", serde_json::to_string_pretty(&state).unwrap_or_default());
        Ok(state)
    }

    // Live game stats from Live Client API
    pub async fn get_live_game_stats(&self) -> Result<serde_json::Value, String> {
        let url = "https://127.0.0.1:2999/liveclientdata/gamestats";
        
        let response = timeout(
            Duration::from_secs(5),
            self.client.get(url).send()
        ).await
        .map_err(|_| "Request timeout".to_string())?
        .map_err(|e| format!("Request failed: {}", e))?;
        
        if response.status().is_success() {
            let stats: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(stats)
        } else {
            Err(format!("Live Client API returned error: {}", response.status()))
        }
    }

    // Player scores from Live Client API
    pub async fn get_player_scores(&self, riot_id: &str) -> Result<serde_json::Value, String> {
        let url = format!("https://127.0.0.1:2999/liveclientdata/playerscores?riotId={}", riot_id);
        
        let response = timeout(
            Duration::from_secs(5),
            self.client.get(&url).send()
        ).await
        .map_err(|_| "Request timeout".to_string())?
        .map_err(|e| format!("Request failed: {}", e))?;
        
        if response.status().is_success() {
            let scores: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(scores)
        } else {
            Err(format!("Failed to get player scores: {}", response.status()))
        }
    }

    // All game data from Live Client API
    pub async fn get_all_game_data(&self) -> Result<serde_json::Value, String> {
        let url = "https://127.0.0.1:2999/liveclientdata/allgamedata";
        
        let response = timeout(
            Duration::from_secs(5),
            self.client.get(url).send()
        ).await
        .map_err(|_| "Request timeout".to_string())?
        .map_err(|e| format!("Request failed: {}", e))?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(data)
        } else {
            Err(format!("Failed to get all game data: {}", response.status()))
        }
    }

    // Check if Live Client is available
    pub async fn is_live_client_available(&self) -> bool {
        let url = "https://127.0.0.1:2999/liveclientdata/gamestats";
        
        match timeout(
            Duration::from_secs(2), // Short timeout for availability check
            self.client.get(url).send()
        ).await {
            Ok(Ok(response)) => {
                let is_available = response.status().is_success();
                println!("[LCU] Live Client API check: {} -> {}", url, is_available);
                is_available
            },
            Ok(Err(e)) => {
                println!("[LCU] Live Client API error: {}", e);
                false
            },
            Err(_) => {
                println!("[LCU] Live Client API timeout");
                false
            }
        }
    }

    // Get game flow session to detect spectator mode
    pub async fn get_game_flow_session(&self) -> Result<serde_json::Value, String> {
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        
        let url = format!("{}://127.0.0.1:{}/lol-gameflow/v1/session", 
                         connection.protocol, connection.port);
        
        println!("[LCU] Requesting game flow session from: {}", url);
        
        let response = timeout(
            Duration::from_secs(5),
            self.client
                .get(&url)
                .basic_auth("riot", Some(&connection.password))
                .send()
        ).await
        .map_err(|_| "Request timeout".to_string())?
        .map_err(|e| format!("Request failed: {}", e))?;

        println!("[LCU] Game flow session response status: {}", response.status());

        if response.status().is_success() {
            let session: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            println!("[LCU] Successfully retrieved game flow session");
            Ok(session)
        } else {
            Err(format!("Failed to get game flow session: {}", response.status()))
        }
    }

    // Helper method for testing - parse lockfile content without file I/O
    #[cfg(test)]
    pub fn parse_lockfile_content(&self, content: &str) -> Result<LCUConnection, String> {
        let parts: Vec<&str> = content.trim().split(':').collect();
        
        if parts.len() < 5 {
            return Err(format!("Invalid lockfile format. Expected 5 parts, got {}. Content: {}", parts.len(), content.trim()));
        }

        let port = parts[2].parse::<u16>()
            .map_err(|e| format!("Failed to parse port from lockfile: {}", e))?;
        
        let password = parts[3].to_string();
        let protocol = parts[4].to_string();

        Ok(LCUConnection {
            port,
            password,
            protocol,
        })
    }

    // Public test methods
    #[cfg(test)]
    pub fn test_find_additional_league_paths(&self) -> Result<Vec<String>, String> {
        self.find_additional_league_paths()
    }

    #[cfg(test)]
    pub fn test_find_league_paths_from_registry(&self) -> Result<Vec<String>, String> {
        self.find_league_paths_from_registry()
    }

    #[cfg(test)]
    pub fn test_find_league_paths_from_shortcuts(&self) -> Result<Vec<String>, String> {
        self.find_league_paths_from_shortcuts()
    }

    #[cfg(test)]
    pub fn test_find_league_paths_from_drives(&self) -> Result<Vec<String>, String> {
        self.find_league_paths_from_drives()
    }

    // Get current game state from LCU API
    pub async fn get_current_game_state(&self) -> Result<serde_json::Value, String> {
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        
        let url = format!("{}://127.0.0.1:{}/lol-gameflow/v1/gameflow-phase", 
                         connection.protocol, connection.port);
        
        println!("[LCU] Requesting current game state from: {}", url);
        
        let response = timeout(
            Duration::from_secs(5),
            self.client
                .get(&url)
                .basic_auth("riot", Some(&connection.password))
                .send()
        ).await
        .map_err(|_| "Request timeout".to_string())?
        .map_err(|e| format!("Request failed: {}", e))?;

        println!("[LCU] Current game state response status: {}", response.status());

        if response.status().is_success() {
            let state: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            println!("[LCU] Successfully retrieved current game state: {}", state);
            Ok(state)
        } else {
            Err(format!("Failed to get current game state: {}", response.status()))
        }
    }
}
