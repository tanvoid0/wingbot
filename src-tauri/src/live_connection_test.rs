#[cfg(test)]
mod live_tests {
    use crate::lcu_api::LCUAPI;
    use tokio;

    #[tokio::test]
    async fn test_live_league_detection() {
        let api = LCUAPI::new();
        println!("Attempting to detect live League client...");
        
        match api.detect_connection().await {
            Ok(connection) => {
                println!("✅ Successfully detected League client!");
                println!("Port: {}", connection.port);
                println!("Protocol: {}", connection.protocol);
                
                // Now try to get game state
                let mut api_with_conn = LCUAPI::new();
                api_with_conn.set_connection(connection);
                
                match api_with_conn.get_current_game_state().await {
                    Ok(phase) => {
                        println!("✅ Current Game Phase: {}", phase);
                    }
                    Err(e) => {
                        println!("❌ Failed to get game phase: {}", e);
                    }
                }
                
                match api_with_conn.get_comprehensive_game_state().await {
                    Ok(state) => {
                        println!("✅ Comprehensive Game State retrieved");
                        println!("Phase: {}", state.game_phase);
                        println!("Live Client Available: {}", state.live_client_available);
                        if let Some(mode) = state.is_spectator_mode {
                            println!("Is Spectator Mode: {}", mode);
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to get comprehensive state: {}", e);
                    }
                }

                // Check live client data specifically
                if api_with_conn.is_live_client_available().await {
                    println!("✅ Live Client Data API is available!");
                    match api_with_conn.get_all_game_data().await {
                        Ok(data) => {
                            println!("✅ Successfully fetched all game data");
                            if let Some(game_data) = data.get("gameData") {
                                println!("Game Mode: {}", game_data.get("gameMode").unwrap_or(&serde_json::Value::Null));
                                println!("Game Time: {}", game_data.get("gameTime").unwrap_or(&serde_json::Value::Null));
                            }
                        }
                        Err(e) => {
                            println!("❌ Failed to fetch all game data: {}", e);
                        }
                    }
                } else {
                    println!("⚠️ Live Client Data API is NOT available (normal if not in-game or just started)");
                }
            }
            Err(e) => {
                println!("❌ Failed to detect League client: {}", e);
                println!("Ensure League of Legends is running and you are logged in.");
            }
        }
    }
}
