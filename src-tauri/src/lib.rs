use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use tokio::sync::Mutex as TokioMutex;
use tokio::task::JoinHandle;

mod game;
mod lcu_api;
mod lcu;
#[cfg(test)]
mod lcu_api_tests;
#[cfg(test)]
mod lcu_api_integration_tests;
#[cfg(test)]
mod lcu_api_mock_tests;
#[cfg(test)]
mod league_fixture_capture;
#[cfg(test)]
mod league_live_api_tests;
mod lcu_websocket;
mod config;
mod websocket_server;
mod data_processing;
mod league_launcher;
mod types;
mod league_session;
mod riot_api;
mod riot_api_cache;
mod match_history_service;
mod llm_service;
mod compact_game_state;
#[cfg(test)]
mod websocket_tests;

use game::{GameManager, GameState, Role};
use lcu_api::LCUAPI;
use lcu_websocket::LcuWebSocket;
use config::{ConfigManager, AppConfig, LeagueClientConfig, UIConfig, OllamaConfig};
use websocket_server::WebSocketServer;
use data_processing::GameDataShaper;
use league_launcher::{LeagueLauncher, LeagueRegion, LeagueInstallation};
use league_session::LeagueSession;
use std::sync::Arc;

// Global instances using OnceLock (Rust 1.70+ stdlib, no external dependency)
static GAME_MANAGER: OnceLock<TokioMutex<GameManager>> = OnceLock::new();
static LCU_API: OnceLock<TokioMutex<LCUAPI>> = OnceLock::new();
static LCU_WEBSOCKET: OnceLock<TokioMutex<LcuWebSocket>> = OnceLock::new();
static CONFIG_MANAGER: OnceLock<Mutex<ConfigManager>> = OnceLock::new();
static WEBSOCKET_SERVER: OnceLock<Mutex<Option<WebSocketServer>>> = OnceLock::new();
static WS_SERVER_HANDLE: OnceLock<Mutex<Option<JoinHandle<()>>>> = OnceLock::new();
static WS_SERVER_RUNNING: OnceLock<AtomicBool> = OnceLock::new();

// Helper functions to initialize and access globals
pub(crate) fn get_game_manager() -> &'static TokioMutex<GameManager> {
    GAME_MANAGER.get_or_init(|| TokioMutex::new(GameManager::new()))
}

pub(crate) fn get_lcu_api() -> &'static TokioMutex<LCUAPI> {
    LCU_API.get_or_init(|| TokioMutex::new(LCUAPI::new()))
}

pub(crate) fn get_lcu_websocket() -> &'static TokioMutex<LcuWebSocket> {
    LCU_WEBSOCKET.get_or_init(|| TokioMutex::new(LcuWebSocket::new()))
}

pub(crate) fn get_config_manager() -> &'static Mutex<ConfigManager> {
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

fn get_ws_server_handle() -> &'static Mutex<Option<JoinHandle<()>>> {
    WS_SERVER_HANDLE.get_or_init(|| Mutex::new(None))
}

fn ws_server_running_flag() -> &'static AtomicBool {
    WS_SERVER_RUNNING.get_or_init(|| AtomicBool::new(false))
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
    LeagueSession::ensure_lcu_connected().await?;
    let lcu_api = get_lcu_api().lock().await.clone();
    let game_manager = get_game_manager().lock().await;
    LeagueSession::populate_game_caches(&game_manager, &lcu_api).await
}

#[tauri::command]
async fn update_game_state(state: Value) -> Result<(), String> {
    let mut game_manager = get_game_manager().lock().await;
    let game_state: GameState = serde_json::from_value(state).map_err(|e| e.to_string())?;
    game_manager.update_state(game_state);
    Ok(())
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
    let _ = LeagueSession::ensure_lcu_connected().await;
    let state = LeagueSession::build_snapshot().await?;
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

#[tauri::command]
async fn update_ollama_config(config: Value) -> Result<(), String> {
    let mut config_manager = get_config_manager().lock().map_err(|e| e.to_string())?;
    let ollama_config: OllamaConfig = serde_json::from_value(config).map_err(|e| e.to_string())?;
    config_manager
        .update_ollama_config(ollama_config)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn test_ollama_connection(config: Option<Value>) -> Result<Value, String> {
    let ollama_config = if let Some(value) = config {
        serde_json::from_value(value).map_err(|e| format!("Invalid Ollama config: {}", e))?
    } else {
        let config_manager = get_config_manager().lock().map_err(|e| e.to_string())?;
        config_manager.get_config().ollama.clone()
    };
    llm_service::test_connection(&ollama_config).await
}

#[tauri::command]
async fn get_ai_recommendations() -> Result<String, String> {
    let ollama_config = {
        let config_manager = get_config_manager().lock().map_err(|e| e.to_string())?;
        config_manager.get_config().ollama.clone()
    };

    if !ollama_config.enabled {
        return Err("AI recommendations are disabled — enable Ollama in Settings".to_string());
    }

    llm_service::check_rate_limit(&ollama_config)?;

    let snapshot = LeagueSession::build_snapshot().await?;
    let lcu_api = get_lcu_api().lock().await;
    let compact = compact_game_state::build_compact_game_state(&snapshot, &lcu_api)
        .await
        .ok_or("No game data available — join champ select or start a game")?;

    let user_content =
        serde_json::to_string(&compact).map_err(|e| format!("Failed to encode game state: {}", e))?;

    llm_service::query(&ollama_config, &user_content).await
}

// WebSocket server management
#[tauri::command]
async fn start_websocket_server(port: u16) -> Result<String, String> {
    if ws_server_running_flag().load(Ordering::Relaxed) {
        return Ok(format!("WebSocket server already running on port {}", port));
    }

    let _ = LeagueSession::initialize().await;

    let running = Arc::new(AtomicBool::new(true));
    ws_server_running_flag().store(true, Ordering::Relaxed);

    let server = WebSocketServer::new(running.clone());

    {
        let mut ws_server = get_websocket_server().lock().map_err(|e| e.to_string())?;
        *ws_server = Some(server.clone());
    }

    let handle = tokio::spawn(async move {
        if let Err(e) = server.start(port).await {
            eprintln!("[WebSocket] Server error: {}", e);
        }
        running.store(false, Ordering::Relaxed);
        ws_server_running_flag().store(false, Ordering::Relaxed);
    });

    {
        let mut stored = get_ws_server_handle().lock().map_err(|e| e.to_string())?;
        *stored = Some(handle);
    }

    let _ = LeagueSession::connect_lcu_websocket_if_needed().await;

    Ok(format!("WebSocket server started on port {}", port))
}

#[tauri::command]
async fn stop_websocket_server() -> Result<String, String> {
    ws_server_running_flag().store(false, Ordering::Relaxed);

    if let Ok(mut stored) = get_ws_server_handle().lock() {
        if let Some(handle) = stored.take() {
            handle.abort();
        }
    }

    {
        let mut ws_server = get_websocket_server().lock().map_err(|e| e.to_string())?;
        if let Some(server) = ws_server.take() {
            server.shutdown();
        }
    }

    {
        let mut lcu_websocket = get_lcu_websocket().lock().await;
        lcu_websocket.disconnect().await;
    }

    Ok("WebSocket server stopped".to_string())
}

#[tauri::command]
async fn get_websocket_server_status() -> Result<Value, String> {
    let ws_server = get_websocket_server().lock().map_err(|e| e.to_string())?;
    let is_running = ws_server.is_some() && ws_server_running_flag().load(Ordering::Relaxed);
    let client_count = ws_server
        .as_ref()
        .map(|s| s.client_count())
        .unwrap_or(0);

    Ok(serde_json::json!({
        "running": is_running,
        "port": if is_running { serde_json::Value::Number(serde_json::Number::from(8765)) } else { serde_json::Value::Null },
        "clients": client_count
    }))
}

// System information
#[tauri::command]
async fn get_system_info() -> Result<Value, String> {
    let league_running = {
        let lcu_api = get_lcu_api().lock().await;
        lcu_api.is_league_running().await
    };

    let lcu_connected = {
        let lcu_api = get_lcu_api().lock().await;
        lcu_api.has_connection()
    };

    let info = serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "platform": std::env::consts::FAMILY,
        "league_running": league_running,
        "lcu_connected": lcu_connected
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
async fn get_match_history(puuid: String, match_count: Option<u32>) -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    let count = match_history_service::clamp_match_count(match_count);
    lcu_api.get_match_history(&puuid, count).await
}

#[tauri::command]
async fn get_player_match_analysis(
    puuid: Option<String>,
    force_refresh: Option<bool>,
    match_count: Option<u32>,
) -> Result<Value, String> {
    let force = force_refresh.unwrap_or(false);
    let count = match_history_service::clamp_match_count(match_count);
    let lcu_api = get_lcu_api().lock().await.clone();
    match puuid {
        Some(id) => {
            match_history_service::MatchHistoryService::fetch_player_analysis(
                &lcu_api, &id, count, force,
            )
            .await
        }
        None => {
            match_history_service::MatchHistoryService::fetch_current_player_analysis(
                &lcu_api, count, force,
            )
            .await
        }
    }
}

#[tauri::command]
async fn get_ranked_stats() -> Result<Value, String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    lcu_api.get_ranked_stats().await
}

#[tauri::command]
async fn test_riot_api_connection(
    api_key: Option<String>,
    platform: Option<String>,
) -> Result<Value, String> {
    let riot_config = {
        let config_manager = get_config_manager().lock().map_err(|e| e.to_string())?;
        let mut config = config::resolve_riot_config(&config_manager.get_config().riot_api);
        if let Some(key) = api_key.filter(|k| !k.trim().is_empty()) {
            config.api_key = Some(config::sanitize_riot_api_key(&key));
        }
        if let Some(p) = platform.filter(|p| !p.trim().is_empty()) {
            config.platform = p;
        }
        config
    };

    let lcu_api = get_lcu_api().lock().await.clone();
    let summoner = lcu_api.get_current_summoner().await?;
    let puuid = summoner
        .get("puuid")
        .and_then(|v| v.as_str())
        .ok_or("Log into the League client to test Riot API with your account")?;

    let mut config = riot_config;
    config.routing_region = riot_api::routing_from_summoner(&summoner, &config);

    let client = riot_api::RiotApiClient::new(config);
    if !client.is_configured() {
        return Err(
            "Riot API key not configured — paste your key in Settings"
                .to_string(),
        );
    }

    client.test_connection(puuid).await.map(|value| {
        if let Ok(mut mgr) = get_config_manager().lock() {
            let _ = mgr.record_riot_api_verification(true, None);
        }
        value
    }).map_err(|e| {
        if let Ok(mut mgr) = get_config_manager().lock() {
            let _ = mgr.record_riot_api_verification(false, Some(e.clone()));
        }
        e
    })
}

#[tauri::command]
async fn get_match_data_status() -> Result<Value, String> {
    let (resolved, last_verified_at, last_error) = {
        let config_manager = get_config_manager().lock().map_err(|e| e.to_string())?;
        let saved = config_manager.get_config().riot_api.clone();
        let resolved = config::resolve_riot_config(&saved);
        (resolved, saved.last_verified_at, saved.last_error)
    };

    let lcu_api = get_lcu_api().lock().await.clone();
    let lcu_connected = lcu_api.has_connection();
    let league_running = lcu_api.is_league_running().await;

    let key_configured = resolved.api_key.is_some();
    let key_masked = resolved
        .api_key
        .as_ref()
        .map(|k| config::mask_riot_api_key(k))
        .unwrap_or_default();

    let key_expired = config::is_riot_key_expired(&last_error, &last_verified_at);

    let active_source = if lcu_connected {
        "lcu"
    } else if key_configured && !key_expired {
        "riot"
    } else {
        "none"
    };

    let needs_daily_key = active_source != "lcu";
    let recommendation = match active_source {
        "lcu" => "Match history loads from the League client. You do not need a Riot API key while logged in.",
        "riot" => "League client is closed — using your saved Riot API key. Dev keys expire every 24 hours.",
        _ if key_expired => "Your Riot API key expired. Regenerate one and paste it below (about once per day).",
        _ => "Log into the League client for automatic stats, or save a Riot API key as a fallback.",
    };

    Ok(serde_json::json!({
        "lcu_connected": lcu_connected,
        "league_running": league_running,
        "riot_key_configured": key_configured,
        "riot_key_masked": key_masked,
        "riot_key_expired": key_expired,
        "last_verified_at": last_verified_at,
        "last_error": last_error,
        "active_source": active_source,
        "needs_daily_key": needs_daily_key,
        "recommendation": recommendation,
    }))
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

    LeagueSession::set_connection_and_websocket(lcu_connection).await
}

#[tauri::command]
async fn check_connection_health() -> Result<bool, String> {
    let lcu_api = get_lcu_api().lock().await.clone();
    let health = lcu_api.check_connection_health().await?;
    Ok(health.get("healthy").and_then(|h| h.as_bool()).unwrap_or(false))
}

#[tauri::command]
async fn get_connection_health_info() -> Result<Value, String> {
    let _ = LeagueSession::ensure_lcu_connected().await;
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
    let mut lcu_websocket = get_lcu_websocket().lock().await;
    lcu_websocket.disconnect().await;
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
    let _ = get_config_manager().lock().map(|_| ());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| {
            tauri::async_runtime::spawn(async {
                let _ = LeagueSession::initialize().await;
            });
            Ok(())
        })
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
            update_league_client_config,
            update_ui_config,
            update_ollama_config,
            test_ollama_connection,
            get_ai_recommendations,

            // System information
            get_system_info,

            // New LCU API endpoints
            get_current_summoner,
            get_champion_by_id,
            get_match_history,
            get_player_match_analysis,
            get_ranked_stats,
            get_match_data_status,
            test_riot_api_connection,
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
