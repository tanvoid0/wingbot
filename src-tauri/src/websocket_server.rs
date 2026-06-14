use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex as TokioMutex, RwLock};
use tokio::time::{interval, Duration};
use uuid::Uuid;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tokio::net::{TcpListener, TcpStream};
use crate::data_processing::DeltaDetector;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebSocketMessage {
    // Client to Server messages
    Subscribe { topics: Vec<String> },
    Unsubscribe { topics: Vec<String> },
    Ping,
    
    // Server to Client messages
    Pong,
    GameStateUpdate { data: serde_json::Value },
    GameStateDelta { changes: serde_json::Value },
    LCUConnectionUpdate { connected: bool, status: String },
    LiveGameDataUpdate { data: serde_json::Value },
    LiveGameDataDelta { changes: serde_json::Value },
    PlayerStatsUpdate { player_id: String, stats: serde_json::Value },
    GameTimeUpdate { game_time: f64 },
    GameEvent { event_type: String, data: serde_json::Value },
    ChampionSelectUpdate { data: serde_json::Value },
    ChampionSelectDelta { changes: serde_json::Value },
    PlayerItemsUpdate { player_id: String, items: serde_json::Value },
    AllPlayersItemsUpdate { items: serde_json::Value },
    BuildRecommendation { player_id: String, recommendations: serde_json::Value },
    MatchHistoryUpdate { player_id: String, analysis: serde_json::Value },
    PlayerStatsAnalysis { player_id: String, stats: serde_json::Value },
    RuneAnalysisUpdate { player_id: String, analysis: serde_json::Value },
    AllPlayersRunesUpdate { runes: serde_json::Value },
    RuneRecommendation { player_id: String, recommendations: serde_json::Value },
    LobbyUpdate { data: serde_json::Value },
    LobbyAnalysis { analysis: serde_json::Value },
    ReadyCheckUpdate { data: serde_json::Value },
    ReadyCheckAction { action: String, result: serde_json::Value },
    LCUWebSocketEvent { event: serde_json::Value },
    ConnectionHealthUpdate { health: serde_json::Value },
    Error { message: String },
}

#[derive(Debug, Clone)]
pub struct ClientConnection {
    pub id: Uuid,
    pub topics: Vec<String>,
    pub last_ping: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct WebSocketServer {
    clients: Arc<RwLock<HashMap<Uuid, ClientConnection>>>,
    broadcast_tx: broadcast::Sender<WebSocketMessage>,
    /// Same instance as Tauri commands — avoid cloning at server start (stale `connection`).
    lcu_api: &'static TokioMutex<crate::lcu_api::LCUAPI>,
    previous_game_state: Arc<RwLock<Option<serde_json::Value>>>,
    previous_live_game_data: Arc<RwLock<Option<serde_json::Value>>>,
    previous_game_time: Arc<RwLock<Option<f64>>>,
    lcu_websocket_connected: Arc<RwLock<bool>>,
    connection_health: Arc<RwLock<ConnectionHealth>>,
}

#[derive(Debug, Clone)]
pub struct ConnectionHealth {
    pub last_check: std::time::SystemTime,
    pub is_healthy: bool,
    pub consecutive_failures: u32,
    pub last_error: Option<String>,
    pub websocket_connected: bool,
}

impl WebSocketServer {
    // Helper function to detect changes in game state
    fn detect_game_state_changes(&self, current: &serde_json::Value, previous: &Option<serde_json::Value>) -> Option<serde_json::Value> {
        if let Some(prev) = previous {
            if current == prev {
                return None; // No changes
            }
            
            // Create a delta object with only the changed fields
            let mut changes = serde_json::Map::new();
            
            if let (Some(current_obj), Some(prev_obj)) = (current.as_object(), prev.as_object()) {
                // Priority fields that should always be checked for changes
                let priority_fields = ["gameData", "activePlayer", "allPlayers", "events", "timestamp"];
                
                for field in &priority_fields {
                    if let (Some(current_value), Some(prev_value)) = (current_obj.get(*field), prev_obj.get(*field)) {
                        if current_value != prev_value {
                            changes.insert(field.to_string(), current_value.clone());
                        }
                    } else if let Some(current_value) = current_obj.get(*field) {
                        // New field
                        changes.insert(field.to_string(), current_value.clone());
                    }
                }
                
                // Check other fields for changes
                for (key, current_value) in current_obj {
                    if !priority_fields.contains(&key.as_str()) {
                        if let Some(prev_value) = prev_obj.get(key) {
                            if current_value != prev_value {
                                changes.insert(key.clone(), current_value.clone());
                            }
                        } else {
                            // New field
                            changes.insert(key.clone(), current_value.clone());
                        }
                    }
                }
            }
            
            if changes.is_empty() {
                return None;
            } else {
                println!("[WebSocket] Game state changes detected: {} fields changed", changes.len());
                Some(serde_json::Value::Object(changes))
            }
        } else {
            // First time, send full state
            println!("[WebSocket] Sending initial game state");
            Some(current.clone())
        }
    }
    
    // Helper function to detect changes in live game data
    fn detect_live_game_data_changes(&self, current: &serde_json::Value, previous: &Option<serde_json::Value>) -> Option<serde_json::Value> {
        if let Some(prev) = previous {
            if current == prev {
                return None; // No changes
            }
            
            // For live game data, we'll focus on key fields that change frequently
            let mut changes = serde_json::Map::new();
            
            if let (Some(current_obj), Some(prev_obj)) = (current.as_object(), prev.as_object()) {
                // Check game time - always include it in delta updates for timer functionality
                println!("[WebSocket] Checking for game time in delta detection...");
                if let Some(game_data) = current_obj.get("gameData") {
                    println!("[WebSocket] gameData found in delta detection: {:?}", game_data);
                    if let Some(current_time) = game_data.get("gameTime") {
                        println!("[WebSocket] Including game time in delta: {:?}", current_time);
                        changes.insert("gameTime".to_string(), current_time.clone());
                    } else {
                        println!("[WebSocket] gameTime NOT found in gameData during delta detection");
                    }
                } else {
                    println!("[WebSocket] gameData NOT found in current_obj during delta detection");
                }
                
                // Check allPlayers for changes
                if let (Some(current_players), Some(prev_players)) = (current_obj.get("allPlayers"), prev_obj.get("allPlayers")) {
                    if current_players != prev_players {
                        changes.insert("allPlayers".to_string(), current_players.clone());
                    }
                }
                
                // Check events for new events
                if let (Some(current_events), Some(prev_events)) = (current_obj.get("events"), prev_obj.get("events")) {
                    if current_events != prev_events {
                        changes.insert("events".to_string(), current_events.clone());
                    }
                }
            }
            
            if changes.is_empty() {
                println!("[WebSocket] No changes detected in live game data");
                None
            } else {
                println!("[WebSocket] Changes detected: {:?}", changes);
                Some(serde_json::Value::Object(changes))
            }
        } else {
            // First time, send full data
            Some(current.clone())
        }
    }

    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);
        
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
            lcu_api: crate::get_lcu_api(),
            previous_game_state: Arc::new(RwLock::new(None)),
            previous_live_game_data: Arc::new(RwLock::new(None)),
            previous_game_time: Arc::new(RwLock::new(None)),
            lcu_websocket_connected: Arc::new(RwLock::new(false)),
            connection_health: Arc::new(RwLock::new(ConnectionHealth {
                last_check: std::time::SystemTime::now(),
                is_healthy: false,
                consecutive_failures: 0,
                last_error: None,
                websocket_connected: false,
            })),
        }
    }

    pub async fn get_client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    pub async fn start(&self, port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await?;
        
        println!("[WebSocket] Server listening on {}", addr);
        
        // Start the game state monitoring task
        let server_clone = self.clone_for_task();
        tokio::spawn(async move {
            server_clone.monitor_game_state().await;
        });
        
        // Start the champion select monitoring task
        let server_clone = self.clone_for_task();
        tokio::spawn(async move {
            server_clone.monitor_champion_select().await;
        });
        
        // Start the match history monitoring task
        let server_clone = self.clone_for_task();
        tokio::spawn(async move {
            server_clone.monitor_match_history().await;
        });
        
        // Start the rune analysis monitoring task
        let server_clone = self.clone_for_task();
        tokio::spawn(async move {
            server_clone.monitor_rune_analysis().await;
        });
        
        // Start the lobby monitoring task
        let server_clone = self.clone_for_task();
        tokio::spawn(async move {
            server_clone.monitor_lobby().await;
        });
        
        // Start the client ping task
        let server_clone = self.clone_for_task();
        tokio::spawn(async move {
            server_clone.ping_clients().await;
        });
        
        // Start LCU WebSocket connection monitoring
        let server_clone = self.clone_for_task();
        tokio::spawn(async move {
            server_clone.monitor_lcu_websocket().await;
        });
        
        // Start connection health monitoring
        let server_clone = self.clone_for_task();
        tokio::spawn(async move {
            server_clone.monitor_connection_health().await;
        });
        
        // Accept incoming connections
        while let Ok((stream, addr)) = listener.accept().await {
            println!("[WebSocket] New connection from {}", addr);
            
            let server_clone = self.clone_for_task();
            tokio::spawn(async move {
                if let Err(e) = server_clone.handle_connection(stream).await {
                    eprintln!("[WebSocket] Error handling connection: {}", e);
                }
            });
        }
        
        Ok(())
    }

    fn clone_for_task(&self) -> WebSocketServerTask {
        WebSocketServerTask {
            clients: Arc::clone(&self.clients),
            broadcast_tx: self.broadcast_tx.clone(),
            lcu_api: self.lcu_api,
            previous_game_state: Arc::clone(&self.previous_game_state),
            previous_live_game_data: Arc::clone(&self.previous_live_game_data),
            previous_game_time: Arc::clone(&self.previous_game_time),
            lcu_websocket_connected: Arc::clone(&self.lcu_websocket_connected),
            connection_health: Arc::clone(&self.connection_health),
        }
    }


    async fn monitor_game_state(&self) {
        let mut interval = interval(Duration::from_millis(1000)); // Check every second
        
        loop {
            interval.tick().await;
            
            // Get comprehensive game state
            let game_state = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.get_comprehensive_game_state().await
            };
            
            match game_state {
                Ok(state) => {
                    // Check for changes in game state
                    let previous_state = self.previous_game_state.read().await;
                    let state_value = serde_json::to_value(&state).unwrap_or_else(|_| serde_json::Value::Null);
                    if let Some(changes) = DeltaDetector::detect_game_state_changes(&state_value, &*previous_state) {
                        if previous_state.is_some() {
                            // Send delta update
                            let msg = WebSocketMessage::GameStateDelta { changes };
                            let _ = self.broadcast_tx.send(msg);
                        } else {
                            // Send full update for first time
                            let msg = WebSocketMessage::GameStateUpdate { data: state_value.clone() };
                            let _ = self.broadcast_tx.send(msg);
                        }
                        
                        // Update previous state
                        drop(previous_state);
                        let mut prev_state = self.previous_game_state.write().await;
                        *prev_state = Some(state_value);
                    }
                }
                Err(e) => {
                    let msg = WebSocketMessage::Error { 
                        message: format!("Failed to get game state: {}", e) 
                    };
                    let _ = self.broadcast_tx.send(msg);
                }
            }
            
            // Check LCU connection status
            let connection_status = {
                let lcu_api = self.lcu_api.lock().await;
                (lcu_api.has_connection(), lcu_api.get_connection_status())
            };
            
            let msg = WebSocketMessage::LCUConnectionUpdate {
                connected: connection_status.0,
                status: connection_status.1,
            };
            let _ = self.broadcast_tx.send(msg);
            
            // If live client is available, get live game data
            let live_client_available = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.is_live_client_available().await
            };
            
            println!("[WebSocket] Live client available: {}", live_client_available);
            
            if live_client_available {
                let live_game_data = {
                    let lcu_api = self.lcu_api.lock().await;
                    lcu_api.get_all_game_data().await
                };
                
                match live_game_data {
                    Ok(data) => {
                        // Debug: Print the entire Live Client API response structure
                        println!("[WebSocket] Live Client API response structure: {:?}", data);
                        
                        // Debug: Check if gameData exists and what it contains
                        if let Some(game_data) = data.get("gameData") {
                            println!("[WebSocket] gameData found: {:?}", game_data);
                            if let Some(game_time) = game_data.get("gameTime") {
                                println!("[WebSocket] gameTime found in gameData: {:?}", game_time);
                            } else {
                                println!("[WebSocket] gameTime NOT found in gameData");
                            }
                        } else {
                            println!("[WebSocket] gameData NOT found in response");
                        }
                        
                        // Debug: Check all top-level keys
                        if let Some(obj) = data.as_object() {
                            println!("[WebSocket] Top-level keys: {:?}", obj.keys().collect::<Vec<_>>());
                        }
                        
                        // Check for changes in live game data
                        let previous_data = self.previous_live_game_data.read().await;
                        if let Some(changes) = self.detect_live_game_data_changes(&data, &*previous_data) {
                            if previous_data.is_some() {
                                // Send delta update
                                let msg = WebSocketMessage::LiveGameDataDelta { changes };
                                let _ = self.broadcast_tx.send(msg);
                            } else {
                                // Send full update for first time
                                let msg = WebSocketMessage::LiveGameDataUpdate { data: data.clone() };
                                let _ = self.broadcast_tx.send(msg);
                            }
                            
                            // Update previous data
                            drop(previous_data);
                            let mut prev_data = self.previous_live_game_data.write().await;
                            *prev_data = Some(data.clone());
                        }
                        
                        // Check for game time changes specifically
                        println!("[WebSocket] Looking for game time in: gameData.gameTime");
                        if let Some(game_time) = data.get("gameData")
                            .and_then(|gd| gd.get("gameTime"))
                            .and_then(|t| t.as_f64()) {
                            let previous_time = self.previous_game_time.read().await;
                            if previous_time.map_or(true, |prev| (game_time - prev).abs() > 0.1) {
                                println!("[WebSocket] Game time changed: {} -> {}", 
                                    previous_time.unwrap_or(0.0), game_time);
                                let msg = WebSocketMessage::GameTimeUpdate { game_time };
                                let _ = self.broadcast_tx.send(msg);
                                
                                drop(previous_time);
                                let mut prev_time = self.previous_game_time.write().await;
                                *prev_time = Some(game_time);
                            }
                        } else {
                            println!("[WebSocket] No game time found in data: {:?}", data.get("gameData").and_then(|gd| gd.get("gameTime")));
                        }
                    }
                    Err(e) => {
                        let msg = WebSocketMessage::Error { 
                            message: format!("Failed to get live game data: {}", e) 
                        };
                        let _ = self.broadcast_tx.send(msg);
                    }
                }
            }
        }
    }

    async fn monitor_champion_select(&self) {
        let mut interval = interval(Duration::from_millis(2000)); // Check every 2 seconds
        
        loop {
            interval.tick().await;
            
            // Check for champion select data
            let champion_select_data = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.get_champion_select_raw().await
            };
            
            match champion_select_data {
                Ok(data) => {
                    // Only send updates if we have valid champion select data
                    if !data.is_null() && data.get("phase").is_some() {
                        let msg = WebSocketMessage::ChampionSelectUpdate { data };
                        let _ = self.broadcast_tx.send(msg);
                    }
                }
                Err(_) => {
                    // Champion select not available (not in champ select phase)
                    // This is normal, so we don't send error messages
                }
            }
            
            // Check for item updates if we have live game data
            let live_client_available_for_items = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.is_live_client_available().await
            };
            
            if live_client_available_for_items {
                let all_players_items = {
                    let lcu_api = self.lcu_api.lock().await;
                    lcu_api.get_all_players_items().await
                };
                
                match all_players_items {
                    Ok(items_data) => {
                        if !items_data.is_null() {
                            let msg = WebSocketMessage::AllPlayersItemsUpdate { items: items_data };
                            let _ = self.broadcast_tx.send(msg);
                        }
                    }
                    Err(_) => {
                        // Items not available (not in game or no live client)
                        // This is normal, so we don't send error messages
                    }
                }
            }
        }
    }

    async fn monitor_match_history(&self) {
        let mut interval = interval(Duration::from_secs(30)); // Check every 30 seconds
        
        loop {
            interval.tick().await;
            
            // Check for current summoner's match history
            let match_history = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.get_current_summoner_match_history().await
            };
            
            match match_history {
                Ok(analysis) => {
                    if !analysis.is_null() {
                        // Send match history analysis for current player
                        let msg = WebSocketMessage::MatchHistoryUpdate { 
                            player_id: "current".to_string(), 
                            analysis 
                        };
                        let _ = self.broadcast_tx.send(msg);
                    }
                }
                Err(_) => {
                    // Match history not available (not logged in or no matches)
                    // This is normal, so we don't send error messages
                }
            }
        }
    }

    async fn monitor_rune_analysis(&self) {
        let mut interval = interval(Duration::from_secs(10)); // Check every 10 seconds
        
        loop {
            interval.tick().await;
            
            // Check for live client availability
            let live_client_available = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.is_live_client_available().await
            };
            
            if live_client_available {
                // Get all players' runes
                let all_players_runes = {
                    let lcu_api = self.lcu_api.lock().await;
                    lcu_api.get_all_players_runes().await
                };
                
                match all_players_runes {
                    Ok(runes_data) => {
                        if !runes_data.is_null() {
                            let msg = WebSocketMessage::AllPlayersRunesUpdate { runes: runes_data.clone() };
                            let _ = self.broadcast_tx.send(msg);
                            
                            // Analyze runes for each player
                            if let Some(runes_obj) = runes_data.as_object() {
                                for (player_name, _) in runes_obj {
                                    let rune_analysis = {
                                        let lcu_api = self.lcu_api.lock().await;
                                        lcu_api.analyze_rune_setup(player_name).await
                                    };
                                    
                                    match rune_analysis {
                                        Ok(analysis) => {
                                            if !analysis.is_null() {
                                                let msg = WebSocketMessage::RuneAnalysisUpdate { 
                                                    player_id: player_name.clone(), 
                                                    analysis 
                                                };
                                                let _ = self.broadcast_tx.send(msg);
                                            }
                                        }
                                        Err(_) => {
                                            // Rune analysis not available for this player
                                            // This is normal, so we don't send error messages
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Runes not available (not in game or no live client)
                        // This is normal, so we don't send error messages
                    }
                }
            }
        }
    }

    async fn monitor_lobby(&self) {
        let mut interval = interval(Duration::from_secs(5)); // Check every 5 seconds
        
        loop {
            interval.tick().await;
            
            // Check for lobby status
            let lobby_status = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.get_lobby_status().await
            };
            
            match lobby_status {
                Ok(status) => {
                    if !status.is_null() {
                        let msg = WebSocketMessage::LobbyUpdate { data: status };
                        let _ = self.broadcast_tx.send(msg);
                    }
                }
                Err(_) => {
                    // Lobby not available (not in lobby)
                    // This is normal, so we don't send error messages
                }
            }
            
            // Check for ready check status
            let ready_check = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.get_ready_check().await
            };
            
            match ready_check {
                Ok(data) => {
                    if !data.is_null() {
                        let msg = WebSocketMessage::ReadyCheckUpdate { data };
                        let _ = self.broadcast_tx.send(msg);
                    }
                }
                Err(_) => {
                    // Ready check not available (not in ready check phase)
                    // This is normal, so we don't send error messages
                }
            }
            
            // Analyze lobby composition
            let lobby_analysis = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.analyze_lobby().await
            };
            
            match lobby_analysis {
                Ok(analysis) => {
                    if !analysis.is_null() {
                        let msg = WebSocketMessage::LobbyAnalysis { analysis };
                        let _ = self.broadcast_tx.send(msg);
                    }
                }
                Err(_) => {
                    // Lobby analysis not available (not in lobby or no members)
                    // This is normal, so we don't send error messages
                }
            }
        }
    }

    async fn ping_clients(&self) {
        let mut interval = interval(Duration::from_secs(30)); // Ping every 30 seconds
        
        loop {
            interval.tick().await;
            
            let clients_to_remove = {
                let mut clients = self.clients.write().await;
                let mut to_remove = Vec::new();
                
                for (client_id, client) in clients.iter_mut() {
                    if client.last_ping.elapsed() > Duration::from_secs(60) {
                        to_remove.push(*client_id);
                    }
                }
                
                for client_id in &to_remove {
                    clients.remove(client_id);
                }
                
                to_remove
            };
            
            if !clients_to_remove.is_empty() {
                println!("[WebSocket] Removed {} inactive clients", clients_to_remove.len());
            }
        }
    }
}

// Helper struct for tasks that need to own the server data
#[derive(Clone)]
struct WebSocketServerTask {
    clients: Arc<RwLock<HashMap<Uuid, ClientConnection>>>,
    broadcast_tx: broadcast::Sender<WebSocketMessage>,
    lcu_api: &'static TokioMutex<crate::lcu_api::LCUAPI>,
    previous_game_state: Arc<RwLock<Option<serde_json::Value>>>,
    previous_live_game_data: Arc<RwLock<Option<serde_json::Value>>>,
    previous_game_time: Arc<RwLock<Option<f64>>>,
    lcu_websocket_connected: Arc<RwLock<bool>>,
    connection_health: Arc<RwLock<ConnectionHealth>>,
}

impl WebSocketServerTask {
    async fn monitor_champion_select(&self) {
        let mut interval = interval(Duration::from_millis(2000)); // Check every 2 seconds
        
        loop {
            interval.tick().await;
            
            // Check for champion select data
            let champion_select_data = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.get_champion_select_raw().await
            };
            
            match champion_select_data {
                Ok(data) => {
                    // Only send updates if we have valid champion select data
                    if !data.is_null() && data.get("phase").is_some() {
                        let msg = WebSocketMessage::ChampionSelectUpdate { data };
                        let _ = self.broadcast_tx.send(msg);
                    }
                }
                Err(_) => {
                    // Champion select not available (not in champ select phase)
                    // This is normal, so we don't send error messages
                }
            }
            
            // Check for item updates if we have live game data
            let live_client_available_for_items = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.is_live_client_available().await
            };
            
            if live_client_available_for_items {
                let all_players_items = {
                    let lcu_api = self.lcu_api.lock().await;
                    lcu_api.get_all_players_items().await
                };
                
                match all_players_items {
                    Ok(items_data) => {
                        if !items_data.is_null() {
                            let msg = WebSocketMessage::AllPlayersItemsUpdate { items: items_data };
                            let _ = self.broadcast_tx.send(msg);
                        }
                    }
                    Err(_) => {
                        // Items not available (not in game or no live client)
                        // This is normal, so we don't send error messages
                    }
                }
            }
        }
    }

    async fn monitor_match_history(&self) {
        let mut interval = interval(Duration::from_secs(30)); // Check every 30 seconds
        
        loop {
            interval.tick().await;
            
            // Check for current summoner's match history
            let match_history = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.get_current_summoner_match_history().await
            };
            
            match match_history {
                Ok(analysis) => {
                    if !analysis.is_null() {
                        // Send match history analysis for current player
                        let msg = WebSocketMessage::MatchHistoryUpdate { 
                            player_id: "current".to_string(), 
                            analysis 
                        };
                        let _ = self.broadcast_tx.send(msg);
                    }
                }
                Err(_) => {
                    // Match history not available (not logged in or no matches)
                    // This is normal, so we don't send error messages
                }
            }
        }
    }

    async fn monitor_rune_analysis(&self) {
        let mut interval = interval(Duration::from_secs(10)); // Check every 10 seconds
        
        loop {
            interval.tick().await;
            
            // Check for live client availability
            let live_client_available = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.is_live_client_available().await
            };
            
            if live_client_available {
                // Get all players' runes
                let all_players_runes = {
                    let lcu_api = self.lcu_api.lock().await;
                    lcu_api.get_all_players_runes().await
                };
                
                match all_players_runes {
                    Ok(runes_data) => {
                        if !runes_data.is_null() {
                            let msg = WebSocketMessage::AllPlayersRunesUpdate { runes: runes_data.clone() };
                            let _ = self.broadcast_tx.send(msg);
                            
                            // Analyze runes for each player
                            if let Some(runes_obj) = runes_data.as_object() {
                                for (player_name, _) in runes_obj {
                                    let rune_analysis = {
                                        let lcu_api = self.lcu_api.lock().await;
                                        lcu_api.analyze_rune_setup(player_name).await
                                    };
                                    
                                    match rune_analysis {
                                        Ok(analysis) => {
                                            if !analysis.is_null() {
                                                let msg = WebSocketMessage::RuneAnalysisUpdate { 
                                                    player_id: player_name.clone(), 
                                                    analysis 
                                                };
                                                let _ = self.broadcast_tx.send(msg);
                                            }
                                        }
                                        Err(_) => {
                                            // Rune analysis not available for this player
                                            // This is normal, so we don't send error messages
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Runes not available (not in game or no live client)
                        // This is normal, so we don't send error messages
                    }
                }
            }
        }
    }

    async fn monitor_lobby(&self) {
        let mut interval = interval(Duration::from_secs(5)); // Check every 5 seconds
        
        loop {
            interval.tick().await;
            
            // Check for lobby status
            let lobby_status = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.get_lobby_status().await
            };
            
            match lobby_status {
                Ok(status) => {
                    if !status.is_null() {
                        let msg = WebSocketMessage::LobbyUpdate { data: status };
                        let _ = self.broadcast_tx.send(msg);
                    }
                }
                Err(_) => {
                    // Lobby not available (not in lobby)
                    // This is normal, so we don't send error messages
                }
            }
            
            // Check for ready check status
            let ready_check = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.get_ready_check().await
            };
            
            match ready_check {
                Ok(data) => {
                    if !data.is_null() {
                        let msg = WebSocketMessage::ReadyCheckUpdate { data };
                        let _ = self.broadcast_tx.send(msg);
                    }
                }
                Err(_) => {
                    // Ready check not available (not in ready check phase)
                    // This is normal, so we don't send error messages
                }
            }
            
            // Analyze lobby composition
            let lobby_analysis = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.analyze_lobby().await
            };
            
            match lobby_analysis {
                Ok(analysis) => {
                    if !analysis.is_null() {
                        let msg = WebSocketMessage::LobbyAnalysis { analysis };
                        let _ = self.broadcast_tx.send(msg);
                    }
                }
                Err(_) => {
                    // Lobby analysis not available (not in lobby or no members)
                    // This is normal, so we don't send error messages
                }
            }
        }
    }

    // Helper function to detect changes in game state
    fn detect_game_state_changes(&self, current: &serde_json::Value, previous: &Option<serde_json::Value>) -> Option<serde_json::Value> {
        if let Some(prev) = previous {
            if current == prev {
                return None; // No changes
            }
            
            // Create a delta object with only the changed fields
            let mut changes = serde_json::Map::new();
            
            if let (Some(current_obj), Some(prev_obj)) = (current.as_object(), prev.as_object()) {
                for (key, current_value) in current_obj {
                    if let Some(prev_value) = prev_obj.get(key) {
                        if current_value != prev_value {
                            changes.insert(key.clone(), current_value.clone());
                        }
                    } else {
                        // New field
                        changes.insert(key.clone(), current_value.clone());
                    }
                }
            }
            
            if changes.is_empty() {
                println!("[WebSocket] No changes detected in live game data");
                None
            } else {
                println!("[WebSocket] Changes detected: {:?}", changes);
                Some(serde_json::Value::Object(changes))
            }
        } else {
            // First time, send full state
            Some(current.clone())
        }
    }
    
    // Helper function to detect changes in live game data
    fn detect_live_game_data_changes(&self, current: &serde_json::Value, previous: &Option<serde_json::Value>) -> Option<serde_json::Value> {
        if let Some(prev) = previous {
            if current == prev {
                return None; // No changes
            }
            
            // For live game data, we'll focus on key fields that change frequently
            let mut changes = serde_json::Map::new();
            
            if let (Some(current_obj), Some(prev_obj)) = (current.as_object(), prev.as_object()) {
                // Check game time - always include it in delta updates for timer functionality
                println!("[WebSocket] Checking for game time in delta detection...");
                if let Some(game_data) = current_obj.get("gameData") {
                    println!("[WebSocket] gameData found in delta detection: {:?}", game_data);
                    if let Some(current_time) = game_data.get("gameTime") {
                        println!("[WebSocket] Including game time in delta: {:?}", current_time);
                        changes.insert("gameTime".to_string(), current_time.clone());
                    } else {
                        println!("[WebSocket] gameTime NOT found in gameData during delta detection");
                    }
                } else {
                    println!("[WebSocket] gameData NOT found in current_obj during delta detection");
                }
                
                // Check allPlayers for changes
                if let (Some(current_players), Some(prev_players)) = (current_obj.get("allPlayers"), prev_obj.get("allPlayers")) {
                    if current_players != prev_players {
                        changes.insert("allPlayers".to_string(), current_players.clone());
                    }
                }
                
                // Check events for new events
                if let (Some(current_events), Some(prev_events)) = (current_obj.get("events"), prev_obj.get("events")) {
                    if current_events != prev_events {
                        changes.insert("events".to_string(), current_events.clone());
                    }
                }
            }
            
            if changes.is_empty() {
                println!("[WebSocket] No changes detected in live game data");
                None
            } else {
                println!("[WebSocket] Changes detected: {:?}", changes);
                Some(serde_json::Value::Object(changes))
            }
        } else {
            // First time, send full data
            Some(current.clone())
        }
    }

    async fn handle_connection(&self, stream: TcpStream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ws_stream = accept_async(stream).await?;
        let (ws_sender, ws_receiver) = ws_stream.split();
        
        let client_id = Uuid::new_v4();
        let broadcast_rx = self.broadcast_tx.subscribe();
        
        // Add client to registry
        {
            let mut clients = self.clients.write().await;
            clients.insert(client_id, ClientConnection {
                id: client_id,
                topics: vec!["all".to_string()], // Default subscription
                last_ping: std::time::Instant::now(),
            });
        }
        
        println!("[WebSocket] Client {} connected", client_id);
        
        // Handle the client connection
        if let Err(e) = self.handle_client_connection(client_id, ws_sender, ws_receiver, broadcast_rx).await {
            eprintln!("[WebSocket] Error handling client {}: {}", client_id, e);
        }
        
        Ok(())
    }

    async fn handle_client_connection(
        &self,
        client_id: Uuid,
        mut ws_sender: futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, Message>,
        mut ws_receiver: futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>>,
        mut broadcast_rx: broadcast::Receiver<WebSocketMessage>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Send initial welcome message
        let welcome_msg = {
            let api = self.lcu_api.lock().await;
            WebSocketMessage::LCUConnectionUpdate {
                connected: api.has_connection(),
                status: api.get_connection_status(),
            }
        };
        
        if let Ok(msg) = serde_json::to_string(&welcome_msg) {
            let _ = ws_sender.send(Message::Text(msg)).await;
        }
        
        // Handle incoming messages and broadcast messages concurrently
        let clients_clone = Arc::clone(&self.clients);
        
        // Create channels for communication between tasks
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(100);
        
        // Spawn task to handle incoming messages
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = ws_receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                            match ws_msg {
                                WebSocketMessage::Subscribe { topics } => {
                                    let mut clients = clients_clone.write().await;
                                    if let Some(client) = clients.get_mut(&client_id) {
                                        client.topics.extend(topics);
                                        client.topics.sort();
                                        client.topics.dedup();
                                    }
                                }
                                WebSocketMessage::Unsubscribe { topics } => {
                                    let mut clients = clients_clone.write().await;
                                    if let Some(client) = clients.get_mut(&client_id) {
                                        client.topics.retain(|t| !topics.contains(t));
                                    }
                                }
                                WebSocketMessage::Ping => {
                                    let mut clients = clients_clone.write().await;
                                    if let Some(client) = clients.get_mut(&client_id) {
                                        client.last_ping = std::time::Instant::now();
                                    }
                                    let pong = WebSocketMessage::Pong;
                                    if let Ok(msg) = serde_json::to_string(&pong) {
                                        let _ = tx_clone.send(Message::Text(msg)).await;
                                    }
                                }
                                _ => {
                                    // Ignore other message types from client
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        println!("[WebSocket] Client {} disconnected", client_id);
                        break;
                    }
                    Ok(Message::Ping(data)) => {
                        let _ = tx_clone.send(Message::Pong(data)).await;
                    }
                    Ok(Message::Pong(_)) => {
                        // Update last ping time
                        let mut clients = clients_clone.write().await;
                        if let Some(client) = clients.get_mut(&client_id) {
                            client.last_ping = std::time::Instant::now();
                        }
                    }
                    Err(e) => {
                        eprintln!("[WebSocket] Error receiving message: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });
        
        // Spawn task to handle broadcast messages
        let clients_clone = Arc::clone(&self.clients);
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            while let Ok(msg) = broadcast_rx.recv().await {
                // Check if client is subscribed to this message type
                let should_send = {
                    let clients = clients_clone.read().await;
                    if let Some(client) = clients.get(&client_id) {
                        client.topics.contains(&"all".to_string()) || 
                        match &msg {
                            WebSocketMessage::GameStateUpdate { .. } => 
                                client.topics.contains(&"game_state".to_string()),
                            WebSocketMessage::LCUConnectionUpdate { .. } => 
                                client.topics.contains(&"lcu_connection".to_string()),
                            WebSocketMessage::LiveGameDataUpdate { .. } => 
                                client.topics.contains(&"live_game".to_string()),
                            WebSocketMessage::GameEvent { .. } => 
                                client.topics.contains(&"game_events".to_string()),
                            _ => true, // Always send pings, pongs, errors
                        }
                    } else {
                        false
                    }
                };
                
                if should_send {
                    if let Ok(msg_str) = serde_json::to_string(&msg) {
                        if tx_clone.send(Message::Text(msg_str)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });
        
        // Handle outgoing messages
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
        
        // Clean up client
        {
            let mut clients = self.clients.write().await;
            clients.remove(&client_id);
        }
        
        Ok(())
    }

    async fn monitor_game_state(&self) {
        let mut interval = interval(Duration::from_millis(1000));
        
        loop {
            interval.tick().await;
            
            let game_state = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.get_comprehensive_game_state().await
            };
            
            match game_state {
                Ok(state) => {
                    let data = serde_json::to_value(state).unwrap_or_else(|_| serde_json::Value::Null);
                    let msg = WebSocketMessage::GameStateUpdate { data };
                    let _ = self.broadcast_tx.send(msg);
                }
                Err(e) => {
                    let msg = WebSocketMessage::Error { 
                        message: format!("Failed to get game state: {}", e) 
                    };
                    let _ = self.broadcast_tx.send(msg);
                }
            }
            
            let connection_status = {
                let lcu_api = self.lcu_api.lock().await;
                (lcu_api.has_connection(), lcu_api.get_connection_status())
            };
            
            let msg = WebSocketMessage::LCUConnectionUpdate {
                connected: connection_status.0,
                status: connection_status.1,
            };
            let _ = self.broadcast_tx.send(msg);
            
            let live_client_available = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.is_live_client_available().await
            };
            
            if live_client_available {
                let live_game_data = {
                    let lcu_api = self.lcu_api.lock().await;
                    lcu_api.get_all_game_data().await
                };
                
                match live_game_data {
                    Ok(data) => {
                        // Debug: Print the entire Live Client API response structure
                        println!("[WebSocket] Live Client API response structure: {:?}", data);
                        
                        // Debug: Check if gameData exists and what it contains
                        if let Some(game_data) = data.get("gameData") {
                            println!("[WebSocket] gameData found: {:?}", game_data);
                            if let Some(game_time) = game_data.get("gameTime") {
                                println!("[WebSocket] gameTime found in gameData: {:?}", game_time);
                            } else {
                                println!("[WebSocket] gameTime NOT found in gameData");
                            }
                        } else {
                            println!("[WebSocket] gameData NOT found in response");
                        }
                        
                        // Debug: Check all top-level keys
                        if let Some(obj) = data.as_object() {
                            println!("[WebSocket] Top-level keys: {:?}", obj.keys().collect::<Vec<_>>());
                        }
                        
                        // Check for changes in live game data
                        let previous_data = self.previous_live_game_data.read().await;
                        if let Some(changes) = self.detect_live_game_data_changes(&data, &*previous_data) {
                            if previous_data.is_some() {
                                // Send delta update
                                let msg = WebSocketMessage::LiveGameDataDelta { changes };
                                let _ = self.broadcast_tx.send(msg);
                            } else {
                                // Send full update for first time
                                let msg = WebSocketMessage::LiveGameDataUpdate { data: data.clone() };
                                let _ = self.broadcast_tx.send(msg);
                            }
                            
                            // Update previous data
                            drop(previous_data);
                            let mut prev_data = self.previous_live_game_data.write().await;
                            *prev_data = Some(data.clone());
                        }
                        
                        // Check for game time changes specifically
                        println!("[WebSocket] Looking for game time in: gameData.gameTime");
                        if let Some(game_time) = data.get("gameData")
                            .and_then(|gd| gd.get("gameTime"))
                            .and_then(|t| t.as_f64()) {
                            let previous_time = self.previous_game_time.read().await;
                            if previous_time.map_or(true, |prev| (game_time - prev).abs() > 0.1) {
                                println!("[WebSocket] Game time changed: {} -> {}", 
                                    previous_time.unwrap_or(0.0), game_time);
                                let msg = WebSocketMessage::GameTimeUpdate { game_time };
                                let _ = self.broadcast_tx.send(msg);
                                
                                drop(previous_time);
                                let mut prev_time = self.previous_game_time.write().await;
                                *prev_time = Some(game_time);
                            }
                        } else {
                            println!("[WebSocket] No game time found in data: {:?}", data.get("gameData").and_then(|gd| gd.get("gameTime")));
                        }
                    }
                    Err(e) => {
                        let msg = WebSocketMessage::Error { 
                            message: format!("Failed to get live game data: {}", e) 
                        };
                        let _ = self.broadcast_tx.send(msg);
                    }
                }
            }
        }
    }

    async fn ping_clients(&self) {
        let mut interval = interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            let clients_to_remove = {
                let mut clients = self.clients.write().await;
                let mut to_remove = Vec::new();
                
                for (client_id, client) in clients.iter_mut() {
                    if client.last_ping.elapsed() > Duration::from_secs(60) {
                        to_remove.push(*client_id);
                    }
                }
                
                for client_id in &to_remove {
                    clients.remove(client_id);
                }
                
                to_remove
            };
            
            if !clients_to_remove.is_empty() {
                println!("[WebSocket] Removed {} inactive clients", clients_to_remove.len());
            }
        }
    }

    async fn monitor_lcu_websocket(&self) {
        let mut interval = interval(Duration::from_secs(5)); // Check every 5 seconds
        
        loop {
            interval.tick().await;
            
            // Check if LCU connection exists or Live Client Data API is available
            let has_connection = {
                let lcu_api = self.lcu_api.lock().await;
                lcu_api.has_connection() || lcu_api.is_live_client_available().await
            };
            
            if has_connection {
                let is_connected = *self.lcu_websocket_connected.read().await;
                
                if !is_connected {
                    println!("[WebSocket] Connection available (LCU or Live Client Data API)...");
                    
                    // Mark as connected since we have either LCU or Live Client Data API available
                    *self.lcu_websocket_connected.write().await = true;
                    
                    // Update connection health
                    let mut health = self.connection_health.write().await;
                    health.websocket_connected = true;
                    health.is_healthy = true;
                    health.consecutive_failures = 0;
                    health.last_error = None;
                }
            } else {
                // No connection available, disconnect WebSocket
                if *self.lcu_websocket_connected.read().await {
                    println!("[WebSocket] Connection lost (LCU and Live Client Data API unavailable), disconnecting WebSocket");
                    *self.lcu_websocket_connected.write().await = false;
                    
                    // Update connection health
                    let mut health = self.connection_health.write().await;
                    health.websocket_connected = false;
                }
            }
        }
    }


    async fn monitor_connection_health(&self) {
        let mut interval = interval(Duration::from_secs(10)); // Check every 10 seconds
        
        loop {
            interval.tick().await;
            
            let base_health = {
                let lcu_api = self.lcu_api.lock().await;
                match lcu_api.check_connection_health().await {
                    Ok(health_data) => health_data,
                    Err(e) => serde_json::json!({
                        "healthy": false,
                        "lcu_connected": false,
                        "live_client_available": false,
                        "status": "error",
                        "error": e,
                    }),
                }
            };
            let health_check = base_health
                .get("healthy")
                .and_then(|h| h.as_bool())
                .unwrap_or(false);
            let lcu_ok = base_health
                .get("lcu_connected")
                .and_then(|h| h.as_bool())
                .unwrap_or(false);
            let live_ok = base_health
                .get("live_client_available")
                .and_then(|h| h.as_bool())
                .unwrap_or(false);
            
            let websocket_connected = *self.lcu_websocket_connected.read().await;
            
            // Update connection health
            let mut health = self.connection_health.write().await;
            health.last_check = std::time::SystemTime::now();
            health.is_healthy = health_check;
            health.websocket_connected = websocket_connected;
            
            if !health_check {
                health.consecutive_failures += 1;
                health.last_error = Some(
                    if !lcu_ok && !live_ok {
                        "LCU and Live Client Data API unreachable".to_string()
                    } else if !lcu_ok {
                        "LCU not connected".to_string()
                    } else {
                        "Live Client Data API unreachable".to_string()
                    },
                );
            } else {
                health.consecutive_failures = 0;
                health.last_error = None;
            }
            
            let last_secs = health
                .last_check
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let mut health_info = base_health;
            if let serde_json::Value::Object(ref mut map) = health_info {
                map.insert(
                    "is_healthy".to_string(),
                    serde_json::json!(health.is_healthy),
                );
                map.insert(
                    "websocket_connected".to_string(),
                    serde_json::json!(health.websocket_connected),
                );
                map.insert(
                    "consecutive_failures".to_string(),
                    serde_json::json!(health.consecutive_failures),
                );
                map.insert(
                    "last_error".to_string(),
                    serde_json::json!(health.last_error),
                );
                map.insert("last_check".to_string(), serde_json::json!(last_secs));
            }
            
            let msg = WebSocketMessage::ConnectionHealthUpdate { health: health_info };
            let _ = self.broadcast_tx.send(msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_message_serialization() {
        let msg = WebSocketMessage::Subscribe {
            topics: vec!["game_state".to_string(), "live_game".to_string()],
        };
        
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            WebSocketMessage::Subscribe { topics } => {
                assert_eq!(topics.len(), 2);
                assert!(topics.contains(&"game_state".to_string()));
                assert!(topics.contains(&"live_game".to_string()));
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[tokio::test]
    async fn test_websocket_server_creation() {
        let server = WebSocketServer::new();
        
        // Test that server can be created
        assert_eq!(server.clients.read().await.len(), 0);
    }
}
