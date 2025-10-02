use serde_json::Value;
use std::sync::Mutex;
use lazy_static::lazy_static;

mod game;
mod ai_service;
mod lcu_api;
mod lcu_api_tests;
mod lcu_api_integration_tests;
mod lcu_api_mock_tests;
mod config;

use game::{GameManager, GameState, Role};
use ai_service::{AIService, AIRequest, ChampionSelectRequest};
use lcu_api::LCUAPI;
use config::{ConfigManager, AppConfig, AIServiceConfig, LeagueClientConfig, UIConfig};

// Global instances
lazy_static! {
    static ref GAME_MANAGER: Mutex<GameManager> = Mutex::new(GameManager::new());
    static ref AI_SERVICE: Mutex<AIService> = Mutex::new(AIService::default());
    static ref LCU_API: Mutex<LCUAPI> = Mutex::new(LCUAPI::new());
    static ref CONFIG_MANAGER: Mutex<ConfigManager> = Mutex::new(
        ConfigManager::new().unwrap_or_else(|_| {
            eprintln!("Failed to load config, using defaults");
            ConfigManager::new().unwrap_or_else(|_| {
                panic!("Failed to create config manager")
            })
        })
    );
}

// Game state management
#[tauri::command]
async fn get_game_state() -> Result<Value, String> {
    let game_manager = GAME_MANAGER.lock().map_err(|e| e.to_string())?;
    let state = game_manager.get_current_state();
    Ok(serde_json::to_value(state).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn update_game_state(state: Value) -> Result<(), String> {
    let mut game_manager = GAME_MANAGER.lock().map_err(|e| e.to_string())?;
    let game_state: GameState = serde_json::from_value(state).map_err(|e| e.to_string())?;
    game_manager.update_state(game_state);
    Ok(())
}

// Champion select recommendations
#[tauri::command]
async fn get_champion_suggestions(
    _enemy_team: Vec<String>,
    _ally_team: Vec<String>,
    _role: String,
) -> Result<Vec<Value>, String> {
    let game_manager = GAME_MANAGER.lock().map_err(|e| e.to_string())?;
    
    // Convert string data to Player structs (simplified for now)
    let suggestions = game_manager.get_champion_suggestions(
        &[], // TODO: Convert enemy_team to Player structs
        &[], // TODO: Convert ally_team to Player structs
        &Role::Mid, // TODO: Convert role string to Role enum
    );
    
    Ok(suggestions.into_iter()
        .map(|s| serde_json::to_value(s).unwrap())
        .collect())
}

#[tauri::command]
async fn get_ability_recommendations(
    _champion: String,
    _game_time: u64,
    _enemy_champions: Vec<String>,
) -> Result<Vec<Value>, String> {
    let game_manager = GAME_MANAGER.lock().map_err(|e| e.to_string())?;
    
    let recommendations = game_manager.get_ability_recommendations(
        &_champion,
        _game_time,
        &_enemy_champions,
    );
    
    Ok(recommendations.into_iter()
        .map(|r| serde_json::to_value(r).unwrap())
        .collect())
}

// AI service integration
#[tauri::command]
async fn ai_chat(session_id: String, message: String) -> Result<Value, String> {
    let request = AIRequest {
        session_id,
        message,
        context: None,
    };
    
    // Clone the base URL from the AI service before dropping the mutex guard
    let base_url = {
        let ai_service = AI_SERVICE.lock().map_err(|e| e.to_string())?;
        ai_service.base_url.clone()
    };
    
    // Create a new AI service instance for this request
    let ai_service = AIService::new(base_url);
    let response = ai_service.generate_response(request).await?;
    
    Ok(serde_json::to_value(response).map_err(|e| e.to_string())?)
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
        let ai_service = AI_SERVICE.lock().map_err(|e| e.to_string())?;
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
    let mut lcu_api = LCU_API.lock().map_err(|e| e.to_string())?;
    
    match lcu_api.detect_connection() {
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
        let api = LCU_API.lock().map_err(|e| e.to_string())?;
        api.clone()
    };
    
    let state = lcu_api.get_champion_select_state().await?;
    Ok(serde_json::to_value(state).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn get_live_game_data() -> Result<Value, String> {
    // Clone the LCU API instance to avoid holding the mutex across await
    let lcu_api = {
        let api = LCU_API.lock().map_err(|e| e.to_string())?;
        api.clone()
    };
    
    let data = lcu_api.get_live_game_data().await?;
    Ok(serde_json::to_value(data).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn get_league_status() -> Result<String, String> {
    let lcu_api = LCU_API.lock().map_err(|e| e.to_string())?;
    Ok(lcu_api.get_connection_status())
}

#[tauri::command]
async fn get_comprehensive_game_state() -> Result<Value, String> {
    // First, try to detect and establish connection
    {
        let mut lcu_api = LCU_API.lock().map_err(|e| e.to_string())?;
        
        // If no connection exists, try to detect it
        if !lcu_api.has_connection() {
            println!("[LCU] No connection found, attempting to detect League client...");
            match lcu_api.detect_connection() {
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
        let api = LCU_API.lock().map_err(|e| e.to_string())?;
        api.clone()
    };
    
    let state = lcu_api.get_comprehensive_game_state().await?;
    Ok(state)
}

#[tauri::command]
async fn get_live_game_stats(_riot_id: String) -> Result<Value, String> {
    // Clone the LCU API instance to avoid holding the mutex across await
    let lcu_api = {
        let api = LCU_API.lock().map_err(|e| e.to_string())?;
        api.clone()
    };
    
    let stats = lcu_api.get_live_game_stats().await?;
    Ok(stats)
}

#[tauri::command]
async fn get_player_scores(riot_id: String) -> Result<Value, String> {
    // Clone the LCU API instance to avoid holding the mutex across await
    let lcu_api = {
        let api = LCU_API.lock().map_err(|e| e.to_string())?;
        api.clone()
    };
    
    let scores = lcu_api.get_player_scores(&riot_id).await?;
    Ok(scores)
}

#[tauri::command]
async fn get_all_game_data() -> Result<Value, String> {
    // Clone the LCU API instance to avoid holding the mutex across await
    let lcu_api = {
        let api = LCU_API.lock().map_err(|e| e.to_string())?;
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
        let mut lcu_api = LCU_API.lock().map_err(|e| e.to_string())?;
        
        // If no connection exists, try to detect it
        if !lcu_api.has_connection() {
            println!("[DEBUG] No connection found, attempting to detect League client...");
            match lcu_api.detect_connection() {
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
        let api = LCU_API.lock().map_err(|e| e.to_string())?;
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
            debug_info["game_flow_phase"] = phase;
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
    let config_manager = CONFIG_MANAGER.lock().map_err(|e| e.to_string())?;
    let config = config_manager.get_config();
    Ok(serde_json::to_value(config).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn update_config(config: Value) -> Result<(), String> {
    let mut config_manager = CONFIG_MANAGER.lock().map_err(|e| e.to_string())?;
    let new_config: AppConfig = serde_json::from_value(config).map_err(|e| e.to_string())?;
    config_manager.update_config(new_config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn update_ai_service_config(config: Value) -> Result<(), String> {
    let mut config_manager = CONFIG_MANAGER.lock().map_err(|e| e.to_string())?;
    let ai_config: AIServiceConfig = serde_json::from_value(config).map_err(|e| e.to_string())?;
    config_manager.update_ai_service_config(ai_config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn update_league_client_config(config: Value) -> Result<(), String> {
    let mut config_manager = CONFIG_MANAGER.lock().map_err(|e| e.to_string())?;
    let lc_config: LeagueClientConfig = serde_json::from_value(config).map_err(|e| e.to_string())?;
    config_manager.update_league_client_config(lc_config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn update_ui_config(config: Value) -> Result<(), String> {
    let mut config_manager = CONFIG_MANAGER.lock().map_err(|e| e.to_string())?;
    let ui_config: UIConfig = serde_json::from_value(config).map_err(|e| e.to_string())?;
    config_manager.update_ui_config(ui_config).map_err(|e| e.to_string())?;
    Ok(())
}

// System information
#[tauri::command]
async fn get_system_info() -> Result<Value, String> {
    let lcu_api = LCU_API.lock().map_err(|e| e.to_string())?;
    let _ai_service = AI_SERVICE.lock().map_err(|e| e.to_string())?;
    
    let info = serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "platform": std::env::consts::FAMILY,
        "league_running": lcu_api.is_league_running(),
        "ai_service_status": "Checking..." // We'll update this separately
    });
    
    Ok(info)
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
            
            // AI service integration
            ai_chat,
            ai_champion_select,
            
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
            
            // Configuration management
            get_config,
            update_config,
            update_ai_service_config,
            update_league_client_config,
            update_ui_config,
            
            // System information
            get_system_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
