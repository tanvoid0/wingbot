use serde_json::Value;
use std::sync::{Mutex, OnceLock};
use tokio::sync::Mutex as TokioMutex;

mod game;
mod ai_service;
mod lcu_api;
mod lcu;
mod lcu_api_tests;
mod lcu_api_integration_tests;
mod lcu_api_mock_tests;
mod live_connection_test;
mod lcu_websocket;
mod config;
mod websocket_server;
mod data_processing;
mod league_launcher;
#[cfg(test)]
mod websocket_tests;

use game::{GameManager, GameState, Role};
use ai_service::{AIService, AIRequest, ChampionSelectRequest};
use lcu_api::LCUAPI;
use lcu_websocket::LcuWebSocket;
use config::{ConfigManager, AppConfig, AIServiceConfig, LeagueClientConfig, UIConfig};
use websocket_server::WebSocketServer;
use data_processing::GameDataShaper;
use league_launcher::{LeagueLauncher, LeagueRegion, LeagueInstallation};

// Global instances using OnceLock (Rust 1.70+ stdlib, no external dependency)
static GAME_MANAGER: OnceLock<TokioMutex<GameManager>> = OnceLock::new();
static AI_SERVICE: OnceLock<Mutex<AIService>> = OnceLock::new();
static LCU_API: OnceLock<TokioMutex<LCUAPI>> = OnceLock::new();
static LCU_WEBSOCKET: OnceLock<TokioMutex<LcuWebSocket>> = OnceLock::new();
static CONFIG_MANAGER: OnceLock<Mutex<ConfigManager>> = OnceLock::new();
static WEBSOCKET_SERVER: OnceLock<Mutex<Option<WebSocketServer>>> = OnceLock::new();

// Helper functions to initialize and access globals
fn get_game_manager() -> &'static TokioMutex<GameManager> {
    GAME_MANAGER.get_or_init(|| TokioMutex::new(GameManager::new()))
}

fn get_ai_service() -> &'static Mutex<AIService> {
    AI_SERVICE.get_or_init(|| Mutex::new(AIService::default()))
}

pub(crate) fn get_lcu_api() -> &'static TokioMutex<LCUAPI> {
    LCU_API.get_or_init(|| TokioMutex::new(LCUAPI::new()))
}

fn get_lcu_websocket() -> &'static TokioMutex<LcuWebSocket> {
    LCU_WEBSOCKET.get_or_init(|| TokioMutex::new(LcuWebSocket::new()))
}

fn get_config_manager() -> &'static Mutex<ConfigManager> {
    CONFIG_MANAGER.get_or_init(|| {
        Mutex::new(ConfigManager::new().unwrap_or_else(|_| {
            eprintln!("Failed to load config, using defaults");
            ConfigManager::new().unwrap_or_else(|_| {
                panic!("Failed to create config manager")
            })
        }))
    })
}

fn get_websocket_server() -> &'static Mutex<Option<WebSocketServer>> {
    WEBSOCKET_SERVER.get_or_init(|| Mutex::new(None))
}

// Game state management
#[tauri::command]
async fn get_game_state() -> Result<Value, String> {
    let game_manager = get_game_manager().lock().await;
    let state = game_manager.get_current_state();
    Ok(serde_json::to_value(state).map_err(|e| e.to_string())?)
}

// Real LCU data commands
#[tauri::command]
async fn get_champions() -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await;
    let champions = lcu_api.get_champions().await?;
    Ok(champions)
}

#[tauri::command]
async fn get_items() -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await;
    let items = lcu_api.get_items().await?;
    Ok(items)
}

#[tauri::command]
async fn get_runes() -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await;
    let runes = lcu_api.get_runes().await?;
    Ok(runes)
}

#[tauri::command]
async fn get_summoner_spells() -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await;
    let spells = lcu_api.get_summoner_spells().await?;
    Ok(spells)
}

// AI-powered recommendations
#[tauri::command]
async fn get_champion_suggestions(enemy_team: Value, ally_team: Value, role: String) -> Result<Value, String> {
    let game_manager = get_game_manager().lock().await;
    
    // Parse role
    let role_enum = match role.to_lowercase().as_str() {
        "top" => Role::Top,
        "jungle" => Role::Jungle,
        "mid" => Role::Mid,
        "adc" => Role::ADC,
        "support" => Role::Support,
        _ => return Err("Invalid role specified".to_string()),
    };
    
    // Parse teams (simplified for now)
    let enemy_players: Vec<game::Player> = serde_json::from_value(enemy_team)
        .map_err(|e| format!("Failed to parse enemy team: {}", e))?;
    let ally_players: Vec<game::Player> = serde_json::from_value(ally_team)
        .map_err(|e| format!("Failed to parse ally team: {}", e))?;
    
    let suggestions = game_manager.get_champion_suggestions(&enemy_players, &ally_players, &role_enum).await?;
    Ok(serde_json::to_value(suggestions).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn get_ability_recommendations(champion: String, game_time: u64, enemy_champions: Vec<String>) -> Result<Value, String> {
    let game_manager = get_game_manager().lock().await;
    
    let recommendations = game_manager.get_ability_recommendations(&champion, game_time, &enemy_champions).await?;
    Ok(serde_json::to_value(recommendations).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn populate_game_data_caches() -> Result<(), String> {
    let game_manager = get_game_manager().lock().await;
    game_manager.populate_caches().await?;
    Ok(())
}

#[tauri::command]
async fn update_game_state(state: Value) -> Result<(), String> {
    let mut game_manager = get_game_manager().lock().await;
    let game_state: GameState = serde_json::from_value(state).map_err(|e| e.to_string())?;
    game_manager.update_state(game_state);
    Ok(())
}


// AI service integration
#[tauri::command]
async fn ai_chat(session_id: String, message: String, context: Option<Value>) -> Result<Value, String> {
    let request = AIRequest {
        session_id,
        message,
        context: context.map(|c| {
            serde_json::from_value(c).unwrap_or_default()
        }),
    };
    
    // Clone the base URL from the AI service before dropping the mutex guard
    let base_url = {
        let ai_service = get_ai_service().lock().map_err(|e| e.to_string())?;
        ai_service.base_url.clone()
    };
    
    // Create a new AI service instance for this request
    let ai_service = AIService::new(base_url);
    let response = ai_service.generate_response(request).await?;
    
    Ok(serde_json::to_value(response).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn generate_tts_audio(text: String) -> Result<Vec<u8>, String> {
    // Clone the base URL from the AI service before dropping the mutex guard
    let base_url = {
        let ai_service = get_ai_service().lock().map_err(|e| e.to_string())?;
        ai_service.base_url.clone()
    };
    
    let client = reqwest::Client::new();
    let url = format!("{}/voice/tts", base_url);
    
    let request_body = serde_json::json!({
        "text": text
    });
    
    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to send TTS request: {}", e))?;
    
    if response.status().is_success() {
        let audio_data = response.bytes().await
            .map_err(|e| format!("Failed to read audio data: {}", e))?;
        Ok(audio_data.to_vec())
    } else {
        Err(format!("TTS service error: {}", response.status()))
    }
}

#[tauri::command]
async fn ai_champion_select(
    session_id: String,
    message: String,
    champ_select_data: Value,
) -> Result<Value, String> {
    let request = ChampionSelectRequest {
        session_id,
        message,
        champ_select_data,
    };
    
    // Clone the base URL from the AI service before dropping the mutex guard
    let base_url = {
        let ai_service = get_ai_service().lock().map_err(|e| e.to_string())?;
        ai_service.base_url.clone()
    };
    
    // Create a new AI service instance for this request
    let ai_service = AIService::new(base_url);
    let response = ai_service.get_champion_suggestions(request).await?;
    Ok(serde_json::to_value(response).map_err(|e| e.to_string())?)
}

// League client integration
#[tauri::command]
async fn detect_league_client() -> Result<Value, String> {
    let lcu_api_guard = get_lcu_api().lock().await;
    
    match lcu_api_guard.detect_connection().await {
        Ok(connection) => {
            Ok(serde_json::to_value(connection).map_err(|e| e.to_string())?)
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
async fn get_champion_select_state() -> Result<Value, String> {
    // Clone the LCU API instance to avoid holding the mutex across await
    let lcu_api = {
        let api = get_lcu_api().lock().await;
        api.clone()
    };
    
    let state = lcu_api.get_champion_select_state().await?;
    Ok(serde_json::to_value(state).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn get_live_game_data() -> Result<Value, String> {
    // Clone the LCU API instance to avoid holding the mutex across await
    let lcu_api = {
        let api = get_lcu_api().lock().await;
        api.clone()
    };
    
    let data = lcu_api.get_live_game_data().await?;
    Ok(serde_json::to_value(data).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn get_league_status() -> Result<String, String> {
    let lcu_api = get_lcu_api().lock().await;
    Ok(lcu_api.get_connection_status())
}

#[tauri::command]
async fn get_comprehensive_game_state() -> Result<Value, String> {
    // First, try to detect and establish connection
    {
        let mut lcu_api = get_lcu_api().lock().await;
        
        // If no connection exists, try to detect it
        if !lcu_api.has_connection() {
            println!("[LCU] No connection found, attempting to detect League client...");
            match lcu_api.detect_connection().await {
                Ok(connection) => {
                    println!("[LCU] Successfully detected League client connection");
                    lcu_api.set_connection(connection);
                }
                Err(e) => {
                    println!("[LCU] Failed to detect League client: {}", e);
                    // Continue anyway to return current state
                }
            }
        }
    } // Mutex is released here
    
    // Clone the LCU API instance to avoid holding the mutex across await
    let lcu_api = {
        let api = get_lcu_api().lock().await;
        api.clone()
    };
    
    let state = lcu_api.get_comprehensive_game_state().await?;
    Ok(serde_json::to_value(state).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn get_live_game_stats(_riot_id: String) -> Result<Value, String> {
    // Clone the LCU API instance to avoid holding the mutex across await
    let lcu_api = {
        let api = get_lcu_api().lock().await;
        api.clone()
    };
    
    let stats = lcu_api.get_live_game_stats().await?;
    Ok(stats)
}

#[tauri::command]
async fn get_player_scores(riot_id: String) -> Result<Value, String> {
    // Clone the LCU API instance to avoid holding the mutex across await
    let lcu_api = {
        let api = get_lcu_api().lock().await;
        api.clone()
    };
    
    let scores = lcu_api.get_player_scores(&riot_id).await?;
    Ok(scores)
}

#[tauri::command]
async fn get_all_game_data() -> Result<Value, String> {
    // Clone the LCU API instance to avoid holding the mutex across await
    let lcu_api = {
        let api = get_lcu_api().lock().await;
        api.clone()
    };
    
    let data = lcu_api.get_all_game_data().await?;
    Ok(data)
}

#[tauri::command]
async fn debug_lcu_connection() -> Result<Value, String> {
    println!("[DEBUG] Starting LCU connection debug...");
    
    // First, try to detect and establish connection
    {
        let mut lcu_api = get_lcu_api().lock().await;
        
        // If no connection exists, try to detect it
        if !lcu_api.has_connection() {
            println!("[DEBUG] No connection found, attempting to detect League client...");
            match lcu_api.detect_connection().await {
                Ok(connection) => {
                    println!("[DEBUG] Successfully detected League client connection");
                    lcu_api.set_connection(connection);
                }
                Err(e) => {
                    println!("[DEBUG] Failed to detect League client: {}", e);
                    return Err(format!("Failed to detect League client: {}", e));
                }
            }
        } else {
            println!("[DEBUG] Connection already exists");
        }
    } // Mutex is released here
    
    // Clone the LCU API instance to avoid holding the mutex across await
    let lcu_api = {
        let api = get_lcu_api().lock().await;
        api.clone()
    };
    
    let mut debug_info = serde_json::json!({
        "connection_status": "connected",
        "live_client_available": false,
        "game_flow_phase": null,
        "game_flow_session": null,
        "champion_select_state": null,
        "errors": []
    });
    
    // Test live client availability
    println!("[DEBUG] Testing live client availability...");
    let live_client_available = lcu_api.is_live_client_available().await;
    debug_info["live_client_available"] = serde_json::json!(live_client_available);
    println!("[DEBUG] Live client available: {}", live_client_available);
    
    // Test game flow phase
    println!("[DEBUG] Testing game flow phase...");
    match lcu_api.get_current_game_state().await {
        Ok(phase) => {
            println!("[DEBUG] Game flow phase: {}", phase);
            debug_info["game_flow_phase"] = serde_json::Value::String(phase);
        }
        Err(e) => {
            debug_info["errors"].as_array_mut().unwrap().push(serde_json::json!({
                "method": "get_current_game_state",
                "error": e
            }));
            println!("[DEBUG] Failed to get game flow phase: {}", e);
        }
    }
    
    // Test game flow session
    println!("[DEBUG] Testing game flow session...");
    match lcu_api.get_game_flow_session().await {
        Ok(session) => {
            debug_info["game_flow_session"] = session;
            println!("[DEBUG] Game flow session retrieved successfully");
        }
        Err(e) => {
            debug_info["errors"].as_array_mut().unwrap().push(serde_json::json!({
                "method": "get_game_flow_session",
                "error": e
            }));
            println!("[DEBUG] Failed to get game flow session: {}", e);
        }
    }
    
    // Test champion select state
    println!("[DEBUG] Testing champion select state...");
    match lcu_api.get_champion_select_state().await {
        Ok(champ_select) => {
            debug_info["champion_select_state"] = serde_json::to_value(champ_select).unwrap_or(serde_json::Value::Null);
            println!("[DEBUG] Champion select state retrieved successfully");
        }
        Err(e) => {
            debug_info["errors"].as_array_mut().unwrap().push(serde_json::json!({
                "method": "get_champion_select_state",
                "error": e
            }));
            println!("[DEBUG] Failed to get champion select state: {}", e);
        }
    }
    
    println!("[DEBUG] Debug info: {}", serde_json::to_string_pretty(&debug_info).unwrap_or_default());
    Ok(debug_info)
}

// Configuration management
#[tauri::command]
async fn get_config() -> Result<Value, String> {
    let config_manager = get_config_manager().lock().map_err(|e| e.to_string())?;
    let config = config_manager.get_config();
    Ok(serde_json::to_value(config).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn update_config(config: Value) -> Result<(), String> {
    let mut config_manager = get_config_manager().lock().map_err(|e| e.to_string())?;
    let new_config: AppConfig = serde_json::from_value(config).map_err(|e| e.to_string())?;
    config_manager.update_config(new_config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn update_ai_service_config(config: Value) -> Result<(), String> {
    let mut config_manager = get_config_manager().lock().map_err(|e| e.to_string())?;
    let ai_config: AIServiceConfig = serde_json::from_value(config).map_err(|e| e.to_string())?;
    config_manager.update_ai_service_config(ai_config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn update_league_client_config(config: Value) -> Result<(), String> {
    let mut config_manager = get_config_manager().lock().map_err(|e| e.to_string())?;
    let lc_config: LeagueClientConfig = serde_json::from_value(config).map_err(|e| e.to_string())?;
    config_manager.update_league_client_config(lc_config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn update_ui_config(config: Value) -> Result<(), String> {
    let mut config_manager = get_config_manager().lock().map_err(|e| e.to_string())?;
    let ui_config: UIConfig = serde_json::from_value(config).map_err(|e| e.to_string())?;
    config_manager.update_ui_config(ui_config).map_err(|e| e.to_string())?;
    Ok(())
}

// WebSocket server management
#[tauri::command]
async fn start_websocket_server(port: u16) -> Result<String, String> {
    let server = WebSocketServer::new();
    
    // Store the server instance
    {
        let mut ws_server = get_websocket_server().lock().map_err(|e| e.to_string())?;
        *ws_server = Some(server.clone());
    }
    
    // Start the server in a background task
    tokio::spawn(async move {
        if let Err(e) = server.start(port).await {
            eprintln!("[WebSocket] Server error: {}", e);
        }
    });
    
    Ok(format!("WebSocket server started on port {}", port))
}

#[tauri::command]
async fn stop_websocket_server() -> Result<String, String> {
    let mut ws_server = get_websocket_server().lock().map_err(|e| e.to_string())?;
    *ws_server = None;
    Ok("WebSocket server stopped".to_string())
}

#[tauri::command]
async fn get_websocket_server_status() -> Result<Value, String> {
    let ws_server = get_websocket_server().lock().map_err(|e| e.to_string())?;
    let is_running = ws_server.is_some();
    
    Ok(serde_json::json!({
        "running": is_running,
        "port": if is_running { serde_json::Value::Number(serde_json::Number::from(8765)) } else { serde_json::Value::Null }, // Default port
        "clients": 0 // TODO: Get actual client count
    }))
}

// System information
#[tauri::command]
async fn get_system_info() -> Result<Value, String> {
    let league_running = {
        let lcu_api = get_lcu_api().lock().await;
        lcu_api.is_league_running().await
    };
    
    let ai_service_status = {
        let ai_service = get_ai_service().lock().map_err(|e| e.to_string())?.clone();
        ai_service.get_server_status().await
    };
    
    let info = serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "platform": std::env::consts::FAMILY,
        "league_running": league_running,
        "ai_service_status": ai_service_status
    });
    
    Ok(info)
}

// New LCU API endpoints
#[tauri::command]
async fn get_current_summoner() -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    lcu_api.get_current_summoner().await
}

#[tauri::command]
async fn get_champion_by_id(champion_id: u64) -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    lcu_api.get_champion_by_id(champion_id).await
}

#[tauri::command]
async fn get_match_history(puuid: String) -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    lcu_api.get_match_history(&puuid).await
}

#[tauri::command]
async fn get_lobby() -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    lcu_api.get_lobby().await
}

#[tauri::command]
async fn get_ready_check() -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    lcu_api.get_ready_check().await
}

#[tauri::command]
async fn get_game_events() -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    lcu_api.get_game_events().await
}

#[tauri::command]
async fn get_active_player_name() -> Result<String, String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    let result = lcu_api.get_active_player_name().await?;
    match result.as_str() {
        Some(name) => Ok(name.to_string()),
        None => Err("Failed to get active player name".to_string()),
    }
}

#[tauri::command]
async fn get_player_list() -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    lcu_api.get_player_list().await
}

// Enhanced LCU API commands with retry logic and WebSocket support
#[tauri::command]
async fn detect_league_client_with_retry(max_attempts: u32) -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    let connection = lcu_api.detect_connection_with_retry(max_attempts).await?;
    
    {
        let mut lcu_api = get_lcu_api().lock().await;
        lcu_api.set_connection(connection.clone());
    }
    
    Ok(serde_json::to_value(connection).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn set_connection_with_websocket(connection: Value) -> Result<(), String> {
    let lcu_connection: crate::lcu_api::LCUConnection = serde_json::from_value(connection)
        .map_err(|e| format!("Failed to parse connection: {}", e))?;
    
    let lcu_api = get_lcu_api().lock().await.clone();
    lcu_api.set_connection_with_websocket(lcu_connection).await?;
    
    Ok(())
}

#[tauri::command]
async fn check_connection_health() -> Result<bool, String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    let health = lcu_api.check_connection_health().await?;
    Ok(health.get("healthy").and_then(|h| h.as_bool()).unwrap_or(false))
}

#[tauri::command]
async fn get_connection_health_info() -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    Ok(lcu_api.get_connection_health_info().await?)
}

#[tauri::command]
async fn get_websocket_status() -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    Ok(lcu_api.get_websocket_status().await?)
}

#[tauri::command]
async fn disconnect_websocket() -> Result<(), String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    lcu_api.disconnect_websocket().await?;
    Ok(())
}

// LCU WebSocket commands
#[tauri::command]
async fn connect_lcu_websocket() -> Result<(), String> {
    let mut lcu_websocket = get_lcu_websocket().lock().await;
    let lcu_api = get_lcu_api().lock().await;
    
    // Get current connection info
    if let Some(connection) = lcu_api.connection.as_ref() {
        lcu_websocket.connect(connection.clone()).await?;
        Ok(())
    } else {
        Err("No LCU connection available".to_string())
    }
}

#[tauri::command]
async fn disconnect_lcu_websocket() -> Result<(), String> {
    let mut lcu_websocket = get_lcu_websocket().lock().await;
    lcu_websocket.disconnect().await;
    Ok(())
}

#[tauri::command]
async fn is_lcu_websocket_connected() -> Result<bool, String> {
    let lcu_websocket = get_lcu_websocket().lock().await;
    Ok(lcu_websocket.is_connected().await)
}

// Debug commands for inspecting raw LCU data
#[tauri::command]
async fn debug_raw_lcu_data() -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    
    // Get raw Live Client Data API response
    match lcu_api.get_all_game_data().await {
        Ok(data) => {
            println!("[DEBUG] Raw LCU data structure:");
            println!("{:#}", serde_json::to_string_pretty(&data).unwrap_or_else(|_| "Failed to serialize".to_string()));
            Ok(data)
        }
        Err(e) => {
            println!("[DEBUG] Error getting raw LCU data: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
async fn debug_live_client_data() -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    
    // Get comprehensive game state with all available data
    match lcu_api.get_comprehensive_game_state().await {
        Ok(state) => {
            let data = serde_json::to_value(state).map_err(|e| e.to_string())?;
            println!("[DEBUG] Comprehensive game state:");
            println!("{:#}", serde_json::to_string_pretty(&data).unwrap_or_else(|_| "Failed to serialize".to_string()));
            Ok(data)
        }
        Err(e) => {
            println!("[DEBUG] Error getting comprehensive game state: {}", e);
            Err(e)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // Game state management
            get_game_state,
            update_game_state,
            get_champion_suggestions,
            get_ability_recommendations,
            
            // Real LCU data commands
            get_champions,
            get_items,
            get_runes,
            get_summoner_spells,
            populate_game_data_caches,
            
            // AI service integration
            ai_chat,
            ai_champion_select,
            generate_tts_audio,
            
            // League client integration
            detect_league_client,
            get_champion_select_state,
            get_live_game_data,
            get_league_status,
            get_comprehensive_game_state,
            get_live_game_stats,
            get_player_scores,
            get_all_game_data,
            debug_lcu_connection,
            
            // WebSocket server management
            start_websocket_server,
            stop_websocket_server,
            get_websocket_server_status,
            
            // Configuration management
            get_config,
            update_config,
            update_ai_service_config,
            update_league_client_config,
            update_ui_config,
            
            // System information
            get_system_info,
            
            // New LCU API endpoints
            get_current_summoner,
            get_champion_by_id,
            get_match_history,
            get_lobby,
            get_ready_check,
            get_game_events,
            get_active_player_name,
            get_player_list,
            
            // Enhanced LCU API commands
            detect_league_client_with_retry,
            set_connection_with_websocket,
            check_connection_health,
            get_connection_health_info,
            get_websocket_status,
            disconnect_websocket,
            
            // LCU WebSocket management
            connect_lcu_websocket,
            disconnect_lcu_websocket,
            is_lcu_websocket_connected,
            
            // Debug commands
            debug_raw_lcu_data,
            debug_live_client_data,
            
            // Overlay commands
            get_cs_tracker_data,
            get_jungle_timers,
            get_objective_timers,
            get_counter_item_suggestions,
            
            // League launcher commands
            detect_league_installations,
            get_available_regions,
            launch_league_of_legends,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Overlay Commands

// Helper function to safely extract array from JSON
fn extract_array_slice<'a>(value: &'a Value, path: &[&str]) -> &'a [Value] {
    let mut current = value;
    for key in path {
        current = match current.get(key) {
            Some(v) => v,
            None => return &[],
        };
    }
    current.as_array().map(|v| v.as_slice()).unwrap_or(&[])
}

#[tauri::command]
async fn get_cs_tracker_data() -> Result<serde_json::Value, String> {
    let lcu_api = get_lcu_api().lock().await;
    
    // Get live game data
    let live_data = lcu_api.get_all_game_data().await?;
    
    // Extract active player and game time
    let active_player = live_data.get("activePlayer")
        .ok_or("No active player data")?;
    let game_time = live_data.get("gameData")
        .and_then(|gd| gd.get("gameTime"))
        .and_then(|gt| gt.as_f64())
        .unwrap_or(0.0);
    
    // Use GameDataShaper to extract CS data
    let cs_data = GameDataShaper::extract_cs_data(active_player, game_time);
    
    Ok(cs_data)
}

#[tauri::command]
async fn get_jungle_timers() -> Result<serde_json::Value, String> {
    let lcu_api = get_lcu_api().lock().await;
    
    // Get live game data
    let live_data = lcu_api.get_all_game_data().await?;
    
    // Extract events and game time
    let events = extract_array_slice(&live_data, &["events", "Events"]);
    let game_time = live_data.get("gameData")
        .and_then(|gd| gd.get("gameTime"))
        .and_then(|gt| gt.as_f64())
        .unwrap_or(0.0);
    
    // Use GameDataShaper to extract jungle camp events
    let jungle_data = GameDataShaper::extract_jungle_camp_events(events, game_time);
    
    Ok(jungle_data)
}

#[tauri::command]
async fn get_objective_timers() -> Result<serde_json::Value, String> {
    let lcu_api = get_lcu_api().lock().await;
    
    // Get live game data
    let live_data = lcu_api.get_all_game_data().await?;
    
    // Extract events and game time
    let events = extract_array_slice(&live_data, &["events", "Events"]);
    let game_time = live_data.get("gameData")
        .and_then(|gd| gd.get("gameTime"))
        .and_then(|gt| gt.as_f64())
        .unwrap_or(0.0);
    
    // Use GameDataShaper to extract objective timers
    let objective_data = GameDataShaper::extract_objective_timers(game_time, events);
    
    Ok(objective_data)
}

#[tauri::command]
async fn get_counter_item_suggestions() -> Result<serde_json::Value, String> {
    let lcu_api = get_lcu_api().lock().await;
    
    // Get live game data
    let live_data = lcu_api.get_all_game_data().await?;
    
    // Extract all players
    let all_players = extract_array_slice(&live_data, &["allPlayers"]);
    
    // Use GameDataShaper to extract counter item suggestions
    let suggestions = GameDataShaper::extract_counter_item_suggestions(all_players);
    
    Ok(suggestions)
}

// League Launcher Commands

#[tauri::command]
async fn detect_league_installations() -> Result<Vec<LeagueInstallation>, String> {
    let mut launcher = LeagueLauncher::new();
    launcher.detect_installations().await
}

#[tauri::command]
async fn get_available_regions() -> Result<Vec<LeagueRegion>, String> {
    let mut launcher = LeagueLauncher::new();
    launcher.detect_installations().await?;
    launcher.load_regions().await
}

#[tauri::command]
async fn launch_league_of_legends(region: String, custom_args: Option<Vec<String>>) -> Result<(), String> {
    let mut launcher = LeagueLauncher::new();
    launcher.detect_installations().await?;
    launcher.load_regions().await?;
    launcher.launch_league(&region, custom_args).await
}
