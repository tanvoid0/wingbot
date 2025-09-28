use crate::domain::lcu::*;
use crate::domain::common::{Result, Validatable};
use crate::lcu_api::{SummonerInfo, GameFlowSession, MatchHistory, LCU_SUMMONER_ENDPOINT, LCU_MATCH_HISTORY_ENDPOINT, LCU_PLAYER_PREFERENCES_ENDPOINT};
use std::time::Duration;
use tokio::time::timeout;
use reqwest::Client;
use std::process::Command;
use regex::Regex;

pub struct LCUService {
    client: Client,
    timeout: Duration,
    connection: Option<LCUConnection>,
}

impl LCUService {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .danger_accept_invalid_certs(true)
                .use_rustls_tls()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap_or_else(|_| Client::new()),
            timeout: Duration::from_secs(5),
            connection: None,
        }
    }
    
    pub async fn detect_connection(&mut self) -> Result<LCUConnection> {
        // Windows-specific implementation
        #[cfg(target_os = "windows")]
        {
            // Move the blocking command execution to a separate thread
            let output = tokio::task::spawn_blocking(|| {
                Command::new("wmic")
                    .args(&["process", "where", "name='LeagueClientUx.exe'", "get", "commandline", "/format:list"])
                    .output()
            }).await
            .map_err(|_| crate::domain::common::DomainError::InternalError("Failed to spawn wmic task".to_string()))?
            .map_err(|_| crate::domain::common::DomainError::InternalError("Failed to execute wmic".to_string()))?;
            
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            // Parse port and password from command line
            let port_regex = Regex::new(r"app-port=(\d+)").unwrap();
            let password_regex = Regex::new(r"app-password=([^\s]+)").unwrap();
            
            let port = port_regex.captures(&output_str)
                .and_then(|cap| cap.get(1))
                .and_then(|m| m.as_str().parse::<u16>().ok())
                .ok_or_else(|| crate::domain::common::DomainError::NotFound("Port not found in command line".to_string()))?;
            
            let password = password_regex.captures(&output_str)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str().to_string())
                .ok_or_else(|| crate::domain::common::DomainError::NotFound("Password not found in command line".to_string()))?;
            
            let connection = LCUConnection {
                port,
                password,
                protocol: "https".to_string(),
            };
            
            connection.validate()?;
            self.connection = Some(connection.clone());
            Ok(connection)
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // TODO: Implement Unix/Linux specific logic
            Err(crate::domain::common::DomainError::InternalError("LCU detection not implemented for this platform".to_string()))
        }
    }
    
    pub async fn get_champion_select_state(&self) -> Result<ChampionSelectState> {
        let connection = self.connection.as_ref()
            .ok_or_else(|| crate::domain::common::DomainError::ConnectionError("No LCU connection established".to_string()))?;
        
        let url = format!("{}://127.0.0.1:{}{}", 
            connection.protocol, 
            connection.port, 
            LCU_CHAMP_SELECT_ENDPOINT
        );
        
        let response = timeout(
            self.timeout,
            self.client.get(&url)
                .basic_auth("riot", Some(&connection.password))
                .send()
        ).await
        .map_err(|_| crate::domain::common::DomainError::TimeoutError("Request timeout".to_string()))?
        .map_err(|e| crate::domain::common::DomainError::ConnectionError(format!("Request failed: {}", e)))?;
        
        if response.status().is_success() {
            let state: ChampionSelectState = response
                .json()
                .await
                .map_err(|e| crate::domain::common::DomainError::ParseError(format!("Failed to parse response: {}", e)))?;
            
            state.validate()?;
            Ok(state)
        } else {
            Err(crate::domain::common::DomainError::ConnectionError(format!("LCU API returned error: {}", response.status())))
        }
    }
    
    pub async fn get_live_game_data(&self) -> Result<LiveGameData> {
        let url = format!("{}{}", LIVE_CLIENT_BASE_URL, LIVE_CLIENT_ALL_GAME_DATA_ENDPOINT);
        
        let response = timeout(
            self.timeout,
            self.client.get(&url).send()
        ).await
        .map_err(|_| crate::domain::common::DomainError::TimeoutError("Request timeout".to_string()))?
        .map_err(|e| crate::domain::common::DomainError::ConnectionError(format!("Request failed: {}", e)))?;
        
        if response.status().is_success() {
            let data: LiveGameData = response
                .json()
                .await
                .map_err(|e| crate::domain::common::DomainError::ParseError(format!("Failed to parse response: {}", e)))?;
            
            data.validate()?;
            Ok(data)
        } else {
            Err(crate::domain::common::DomainError::ConnectionError(format!("Live Client API returned error: {}", response.status())))
        }
    }
    
    pub async fn get_connection_status(&self) -> String {
        if self.connection.is_some() {
            "Connected".to_string()
        } else {
            "Disconnected".to_string()
        }
    }
    
    pub async fn is_league_running(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            let result = tokio::task::spawn_blocking(|| {
                Command::new("wmic")
                    .args(&["process", "where", "name='LeagueClientUx.exe'", "get", "processid", "/format:list"])
                    .output()
            }).await;
            
            match result {
                Ok(Ok(output)) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    output_str.contains("ProcessId=")
                }
                _ => false
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // TODO: Implement Unix/Linux specific logic
            false
        }
    }
    
    // Live Client Data API methods
    pub async fn get_live_game_stats(&self) -> Result<serde_json::Value> {
        let url = format!("{}{}", LIVE_CLIENT_BASE_URL, LIVE_CLIENT_GAMESTATS_ENDPOINT);
        
        let response = timeout(
            self.timeout,
            self.client.get(&url).send()
        ).await
        .map_err(|_| crate::domain::common::DomainError::TimeoutError("Request timeout".to_string()))?
        .map_err(|e| crate::domain::common::DomainError::ConnectionError(format!("Request failed: {}", e)))?;
        
        if response.status().is_success() {
            let stats: serde_json::Value = response
                .json()
                .await
                .map_err(|e| crate::domain::common::DomainError::ParseError(format!("Failed to parse response: {}", e)))?;
            Ok(stats)
        } else {
            Err(crate::domain::common::DomainError::ConnectionError(format!("Failed to get game stats: {}", response.status())))
        }
    }
    
    pub async fn get_player_scores(&self, riot_id: &str) -> Result<serde_json::Value> {
        let url = format!("{}{}{}?riotId={}", 
            LIVE_CLIENT_BASE_URL, 
            LIVE_CLIENT_PLAYER_SCORES_ENDPOINT, 
            "", 
            riot_id
        );
        
        let response = timeout(
            self.timeout,
            self.client.get(&url).send()
        ).await
        .map_err(|_| crate::domain::common::DomainError::TimeoutError("Request timeout".to_string()))?
        .map_err(|e| crate::domain::common::DomainError::ConnectionError(format!("Request failed: {}", e)))?;
        
        if response.status().is_success() {
            let scores: serde_json::Value = response
                .json()
                .await
                .map_err(|e| crate::domain::common::DomainError::ParseError(format!("Failed to parse response: {}", e)))?;
            Ok(scores)
        } else {
            Err(crate::domain::common::DomainError::ConnectionError(format!("Failed to get player scores: {}", response.status())))
        }
    }
    
    pub async fn get_player_items(&self, riot_id: &str) -> Result<serde_json::Value> {
        let url = format!("{}{}{}?riotId={}", 
            LIVE_CLIENT_BASE_URL, 
            LIVE_CLIENT_PLAYER_ITEMS_ENDPOINT, 
            "", 
            riot_id
        );
        
        let response = timeout(
            self.timeout,
            self.client.get(&url).send()
        ).await
        .map_err(|_| crate::domain::common::DomainError::TimeoutError("Request timeout".to_string()))?
        .map_err(|e| crate::domain::common::DomainError::ConnectionError(format!("Request failed: {}", e)))?;
        
        if response.status().is_success() {
            let items: serde_json::Value = response
                .json()
                .await
                .map_err(|e| crate::domain::common::DomainError::ParseError(format!("Failed to parse response: {}", e)))?;
            Ok(items)
        } else {
            Err(crate::domain::common::DomainError::ConnectionError(format!("Failed to get player items: {}", response.status())))
        }
    }
    
    pub async fn get_player_summoner_spells(&self, riot_id: &str) -> Result<serde_json::Value> {
        let url = format!("{}{}{}?riotId={}", 
            LIVE_CLIENT_BASE_URL, 
            LIVE_CLIENT_PLAYER_SUMMONER_SPELLS_ENDPOINT, 
            "", 
            riot_id
        );
        
        let response = timeout(
            self.timeout,
            self.client.get(&url).send()
        ).await
        .map_err(|_| crate::domain::common::DomainError::TimeoutError("Request timeout".to_string()))?
        .map_err(|e| crate::domain::common::DomainError::ConnectionError(format!("Request failed: {}", e)))?;
        
        if response.status().is_success() {
            let spells: serde_json::Value = response
                .json()
                .await
                .map_err(|e| crate::domain::common::DomainError::ParseError(format!("Failed to parse response: {}", e)))?;
            Ok(spells)
        } else {
            Err(crate::domain::common::DomainError::ConnectionError(format!("Failed to get summoner spells: {}", response.status())))
        }
    }
    
    pub async fn get_player_runes(&self, riot_id: &str) -> Result<serde_json::Value> {
        let url = format!("{}{}{}?riotId={}", 
            LIVE_CLIENT_BASE_URL, 
            LIVE_CLIENT_PLAYER_RUNES_ENDPOINT, 
            "", 
            riot_id
        );
        
        let response = timeout(
            self.timeout,
            self.client.get(&url).send()
        ).await
        .map_err(|_| crate::domain::common::DomainError::TimeoutError("Request timeout".to_string()))?
        .map_err(|e| crate::domain::common::DomainError::ConnectionError(format!("Request failed: {}", e)))?;
        
        if response.status().is_success() {
            let runes: serde_json::Value = response
                .json()
                .await
                .map_err(|e| crate::domain::common::DomainError::ParseError(format!("Failed to parse response: {}", e)))?;
            Ok(runes)
        } else {
            Err(crate::domain::common::DomainError::ConnectionError(format!("Failed to get player runes: {}", response.status())))
        }
    }
    
    pub async fn get_game_events(&self) -> Result<serde_json::Value> {
        let url = format!("{}{}", LIVE_CLIENT_BASE_URL, LIVE_CLIENT_EVENT_DATA_ENDPOINT);
        
        let response = timeout(
            self.timeout,
            self.client.get(&url).send()
        ).await
        .map_err(|_| crate::domain::common::DomainError::TimeoutError("Request timeout".to_string()))?
        .map_err(|e| crate::domain::common::DomainError::ConnectionError(format!("Request failed: {}", e)))?;
        
        if response.status().is_success() {
            let events: serde_json::Value = response
                .json()
                .await
                .map_err(|e| crate::domain::common::DomainError::ParseError(format!("Failed to parse response: {}", e)))?;
            Ok(events)
        } else {
            Err(crate::domain::common::DomainError::ConnectionError(format!("Failed to get game events: {}", response.status())))
        }
    }
    
    pub async fn get_all_game_data(&self) -> Result<serde_json::Value> {
        let url = format!("{}{}", LIVE_CLIENT_BASE_URL, LIVE_CLIENT_ALL_GAME_DATA_ENDPOINT);
        
        let response = timeout(
            self.timeout,
            self.client.get(&url).send()
        ).await
        .map_err(|_| crate::domain::common::DomainError::TimeoutError("Request timeout".to_string()))?
        .map_err(|e| crate::domain::common::DomainError::ConnectionError(format!("Request failed: {}", e)))?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response
                .json()
                .await
                .map_err(|e| crate::domain::common::DomainError::ParseError(format!("Failed to parse response: {}", e)))?;
            Ok(data)
        } else {
            Err(crate::domain::common::DomainError::ConnectionError(format!("Failed to get all game data: {}", response.status())))
        }
    }
    
    pub async fn is_live_client_available(&self) -> Result<bool> {
        let url = format!("{}{}", LIVE_CLIENT_BASE_URL, LIVE_CLIENT_GAMESTATS_ENDPOINT);
        
        match timeout(
            Duration::from_secs(2), // Short timeout for availability check
            self.client.get(&url).send()
        ).await {
            Ok(Ok(response)) => Ok(response.status().is_success()),
            _ => Ok(false)
        }
    }
    
    pub async fn get_comprehensive_game_state(&self) -> Result<ComprehensiveGameState> {
        let lcu_connected = self.connection.is_some();
        let live_client_available = self.is_live_client_available().await.unwrap_or(false);
        
        let mut state = ComprehensiveGameState {
            lcu_connected,
            live_client_available,
            game_phase: GAME_PHASE_UNKNOWN.to_string(),
            champion_select: None,
            live_game: None,
            timestamp: crate::domain::common::utils::now_timestamp(),
        };
        
        // Try to get champion select state if LCU is connected
        if lcu_connected {
            if let Ok(champ_select) = self.get_champion_select_state().await {
                state.game_phase = GAME_PHASE_CHAMPION_SELECT.to_string();
                state.champion_select = Some(champ_select);
            }
        }
        
        // Try to get live game data if Live Client is available
        if live_client_available {
            if let Ok(live_game) = self.get_live_game_data().await {
                state.game_phase = GAME_PHASE_IN_GAME.to_string();
                state.live_game = Some(live_game);
            }
        } else if lcu_connected {
            // Check if we're in spectator mode by trying to get game flow session
            if let Ok(session) = self.get_game_flow_session().await {
                if session.phase == "InProgress" {
                    // We're in a game but live client is not available - likely spectator mode
                    state.game_phase = GAME_PHASE_IN_GAME.to_string();
                    state.live_client_available = false; // Ensure this is false for spectator mode
                }
            }
        }
        
        state.validate()?;
        Ok(state)
    }
    
    // LCU API specific methods
    pub async fn get_current_summoner(&self) -> Result<SummonerInfo> {
        let connection = self.connection.as_ref()
            .ok_or_else(|| crate::domain::common::DomainError::ConnectionError("No LCU connection established".to_string()))?;
        
        let url = format!("{}://127.0.0.1:{}{}", 
            connection.protocol, 
            connection.port, 
            LCU_SUMMONER_ENDPOINT
        );
        
        let response = timeout(
            self.timeout,
            self.client.get(&url)
                .basic_auth("riot", Some(&connection.password))
                .send()
        ).await
        .map_err(|_| crate::domain::common::DomainError::TimeoutError("Request timeout".to_string()))?
        .map_err(|e| crate::domain::common::DomainError::ConnectionError(format!("Request failed: {}", e)))?;
        
        if response.status().is_success() {
            let summoner: SummonerInfo = response
                .json()
                .await
                .map_err(|e| crate::domain::common::DomainError::ParseError(format!("Failed to parse response: {}", e)))?;
            
            Ok(summoner)
        } else {
            Err(crate::domain::common::DomainError::ConnectionError(format!("LCU API returned error: {}", response.status())))
        }
    }
    
    pub async fn get_game_flow_session(&self) -> Result<GameFlowSession> {
        let connection = self.connection.as_ref()
            .ok_or_else(|| crate::domain::common::DomainError::ConnectionError("No LCU connection established".to_string()))?;
        
        let url = format!("{}://127.0.0.1:{}{}", 
            connection.protocol, 
            connection.port, 
            LCU_GAMEFLOW_ENDPOINT
        );
        
        let response = timeout(
            self.timeout,
            self.client.get(&url)
                .basic_auth("riot", Some(&connection.password))
                .send()
        ).await
        .map_err(|_| crate::domain::common::DomainError::TimeoutError("Request timeout".to_string()))?
        .map_err(|e| crate::domain::common::DomainError::ConnectionError(format!("Request failed: {}", e)))?;
        
        if response.status().is_success() {
            let session: GameFlowSession = response
                .json()
                .await
                .map_err(|e| crate::domain::common::DomainError::ParseError(format!("Failed to parse response: {}", e)))?;
            
            Ok(session)
        } else {
            Err(crate::domain::common::DomainError::ConnectionError(format!("LCU API returned error: {}", response.status())))
        }
    }
    
    pub async fn get_match_history(&self, puuid: &str) -> Result<Vec<MatchHistory>> {
        let connection = self.connection.as_ref()
            .ok_or_else(|| crate::domain::common::DomainError::ConnectionError("No LCU connection established".to_string()))?;
        
        let url = format!("{}://127.0.0.1:{}{}/{}/matches", 
            connection.protocol, 
            connection.port, 
            LCU_MATCH_HISTORY_ENDPOINT,
            puuid
        );
        
        let response = timeout(
            self.timeout,
            self.client.get(&url)
                .basic_auth("riot", Some(&connection.password))
                .send()
        ).await
        .map_err(|_| crate::domain::common::DomainError::TimeoutError("Request timeout".to_string()))?
        .map_err(|e| crate::domain::common::DomainError::ConnectionError(format!("Request failed: {}", e)))?;
        
        if response.status().is_success() {
            let history: Vec<MatchHistory> = response
                .json()
                .await
                .map_err(|e| crate::domain::common::DomainError::ParseError(format!("Failed to parse response: {}", e)))?;
            
            Ok(history)
        } else {
            Err(crate::domain::common::DomainError::ConnectionError(format!("LCU API returned error: {}", response.status())))
        }
    }
    
    pub async fn get_player_preferences(&self) -> Result<serde_json::Value> {
        let connection = self.connection.as_ref()
            .ok_or_else(|| crate::domain::common::DomainError::ConnectionError("No LCU connection established".to_string()))?;
        
        let url = format!("{}://127.0.0.1:{}{}", 
            connection.protocol, 
            connection.port, 
            LCU_PLAYER_PREFERENCES_ENDPOINT
        );
        
        let response = timeout(
            self.timeout,
            self.client.get(&url)
                .basic_auth("riot", Some(&connection.password))
                .send()
        ).await
        .map_err(|_| crate::domain::common::DomainError::TimeoutError("Request timeout".to_string()))?
        .map_err(|e| crate::domain::common::DomainError::ConnectionError(format!("Request failed: {}", e)))?;
        
        if response.status().is_success() {
            let preferences: serde_json::Value = response
                .json()
                .await
                .map_err(|e| crate::domain::common::DomainError::ParseError(format!("Failed to parse response: {}", e)))?;
            
            Ok(preferences)
        } else {
            Err(crate::domain::common::DomainError::ConnectionError(format!("LCU API returned error: {}", response.status())))
        }
    }
}
