use serde::{Deserialize, Serialize};
use std::process::Command;
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
        Self {
            connection: None,
            client: reqwest::Client::new(),
        }
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
        // Parse League client command line arguments to find port and password
        let output = Command::new("wmic")
            .args(&["process", "where", "name='LeagueClientUx.exe'", "get", "commandline"])
            .output()
            .map_err(|e| format!("Failed to execute wmic: {}", e))?;

        let command_line = String::from_utf8_lossy(&output.stdout);
        
        // Parse command line for --app-port and --remoting-auth-token
        let port = self.extract_port(&command_line)?;
        let password = self.extract_password(&command_line)?;
        
        Ok(LCUConnection {
            port,
            password,
            protocol: "https".to_string(),
        })
    }

    fn detect_unix_connection(&self) -> Result<LCUConnection, String> {
        // TODO: Implement Unix/Linux detection
        Err("Unix/Linux detection not implemented yet".to_string())
    }

    fn extract_port(&self, command_line: &str) -> Result<u16, String> {
        // Extract port from --app-port argument
        let port_pattern = r"--app-port=(\d+)";
        let re = regex::Regex::new(port_pattern)
            .map_err(|e| format!("Failed to create regex: {}", e))?;
        
        if let Some(captures) = re.captures(command_line) {
            if let Some(port_str) = captures.get(1) {
                return port_str.as_str().parse::<u16>()
                    .map_err(|e| format!("Failed to parse port: {}", e));
            }
        }
        
        Err("Port not found in command line".to_string())
    }

    fn extract_password(&self, command_line: &str) -> Result<String, String> {
        // Extract password from --remoting-auth-token argument
        let password_pattern = r"--remoting-auth-token=([a-zA-Z0-9]+)";
        let re = regex::Regex::new(password_pattern)
            .map_err(|e| format!("Failed to create regex: {}", e))?;
        
        if let Some(captures) = re.captures(command_line) {
            if let Some(password) = captures.get(1) {
                return Ok(password.as_str().to_string());
            }
        }
        
        Err("Password not found in command line".to_string())
    }

    pub async fn get_champion_select_state(&self) -> Result<ChampionSelectState, String> {
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        
        let url = format!("{}://127.0.0.1:{}/lol-champ-select/v1/session", 
                         connection.protocol, connection.port);
        
        let response = timeout(
            Duration::from_secs(5),
            self.client
                .get(&url)
                .basic_auth("riot", Some(&connection.password))
                .send()
        ).await
        .map_err(|_| "Request timeout".to_string())?
        .map_err(|e| format!("Request failed: {}", e))?;

        if response.status().is_success() {
            let champ_select: ChampionSelectState = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(champ_select)
        } else {
            Err(format!("Failed to get champion select state: {}", response.status()))
        }
    }

    pub async fn get_live_game_data(&self) -> Result<LiveGameData, String> {
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        
        let url = format!("{}://127.0.0.1:{}/lol-gameflow/v1/session", 
                         connection.protocol, connection.port);
        
        let response = timeout(
            Duration::from_secs(5),
            self.client
                .get(&url)
                .basic_auth("riot", Some(&connection.password))
                .send()
        ).await
        .map_err(|_| "Request timeout".to_string())?
        .map_err(|e| format!("Request failed: {}", e))?;

        if response.status().is_success() {
            let game_data: LiveGameData = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(game_data)
        } else {
            Err(format!("Failed to get live game data: {}", response.status()))
        }
    }

    pub fn is_league_running(&self) -> bool {
        if cfg!(target_os = "windows") {
            let output = Command::new("tasklist")
                .args(&["/FI", "IMAGENAME eq LeagueClientUx.exe"])
                .output();
            
            if let Ok(output) = output {
                let output_str = String::from_utf8_lossy(&output.stdout);
                output_str.contains("LeagueClientUx.exe")
            } else {
                false
            }
        } else {
            // TODO: Implement Unix/Linux process check
            false
        }
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
}
