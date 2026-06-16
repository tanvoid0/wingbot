use crate::config::LeagueClientConfig;
use crate::game::GameManager;
use crate::lcu_api::LCUAPI;
use crate::types::GameSnapshot;

/// Coordinates LCU REST, LCU WebSocket, and live client data access.
pub struct LeagueSession;

impl LeagueSession {
    pub fn league_client_config() -> LeagueClientConfig {
        super::get_config_manager()
            .lock()
            .map(|m| m.get_config().league_client.clone())
            .unwrap_or_default()
    }

    pub fn refresh_interval_ms() -> u64 {
        super::get_config_manager()
            .lock()
            .map(|m| {
                let cfg = m.get_config();
                if cfg.auto_refresh.enabled {
                    cfg.auto_refresh.interval_seconds.saturating_mul(1000).max(500)
                } else {
                    2000
                }
            })
            .unwrap_or(2000)
    }

    /// Fast fallback poll while in champion select (LCU push is primary).
    pub fn champion_select_poll_interval_ms() -> u64 {
        250
    }

    /// Slower safety-net poll when LCU WebSocket push is active.
    pub fn champion_select_slow_poll_interval_ms() -> u64 {
        3000
    }

    pub async fn ensure_lcu_connected() -> Result<(), String> {
        let config = Self::league_client_config();
        let mut lcu_api = super::get_lcu_api().lock().await;

        if lcu_api.has_connection() {
            if lcu_api.get_current_summoner().await.is_ok() {
                return Ok(());
            }
            println!("[LeagueSession] Stale LCU credentials, re-detecting...");
            lcu_api.connection = None;
        }

        println!("[LeagueSession] Detecting League client...");
        let connection = lcu_api.detect_connection_with_config(&config).await?;
        lcu_api.set_connection(connection);
        println!("[LeagueSession] LCU connection established");
        Ok(())
    }

    pub async fn connect_lcu_websocket_if_needed() -> Result<(), String> {
        Self::ensure_lcu_connected().await?;

        let lcu_websocket = super::get_lcu_websocket().lock().await;
        if lcu_websocket.is_connected().await {
            return Ok(());
        }
        drop(lcu_websocket);

        let connection = {
            let lcu_api = super::get_lcu_api().lock().await;
            lcu_api
                .connection
                .clone()
                .ok_or("No LCU connection available for WebSocket".to_string())?
        };

        let mut lcu_websocket = super::get_lcu_websocket().lock().await;
        lcu_websocket.connect(connection).await?;
        println!("[LeagueSession] LCU WebSocket connected");
        Ok(())
    }

    pub async fn build_snapshot() -> Result<GameSnapshot, String> {
        let _ = Self::ensure_lcu_connected().await;
        let lcu_api = super::get_lcu_api().lock().await;
        lcu_api.get_game_snapshot().await
    }

    pub async fn populate_game_caches(game_manager: &GameManager, lcu_api: &LCUAPI) -> Result<(), String> {
        game_manager.populate_caches_from_lcu(lcu_api).await
    }

    pub async fn initialize() -> Result<(), String> {
        if Self::ensure_lcu_connected().await.is_ok() {
            let _ = Self::connect_lcu_websocket_if_needed().await;
            let lcu_api = super::get_lcu_api().lock().await.clone();
            let game_manager = super::get_game_manager().lock().await;
            let _ = Self::populate_game_caches(&game_manager, &lcu_api).await;
        }
        Ok(())
    }

    pub async fn set_connection_and_websocket(
        connection: crate::lcu_api::LCUConnection,
    ) -> Result<(), String> {
        {
            let mut lcu_api = super::get_lcu_api().lock().await;
            lcu_api.set_connection(connection.clone());
        }
        let mut lcu_websocket = super::get_lcu_websocket().lock().await;
        lcu_websocket.disconnect().await;
        lcu_websocket.connect(connection).await
    }
}
