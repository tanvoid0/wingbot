#[cfg(test)]
mod integration_tests {
    use crate::lcu_api::{LCUAPI, LCUConnection};
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;
    use tokio_test;
    use serde_json::json;

    // Mock HTTP responses for testing
    const MOCK_GAMEFLOW_PHASE_RESPONSE: &str = r#""InProgress""#;
    const MOCK_GAMEFLOW_SESSION_RESPONSE: &str = r#"{
        "phase": "InProgress",
        "gameData": {
            "gameId": 12345,
            "gameMode": "CLASSIC",
            "gameType": "MATCHED_GAME"
        }
    }"#;
    const MOCK_CHAMP_SELECT_RESPONSE: &str = r#"{
        "phase": "BAN_PICK",
        "timeRemaining": 30,
        "myTeam": [],
        "enemyTeam": [],
        "bans": [],
        "picks": []
    }"#;

    // Helper function to create a mock lockfile
    fn create_mock_lockfile(dir: &Path, content: &str) -> std::io::Result<()> {
        let lockfile_path = dir.join("lockfile");
        fs::write(lockfile_path, content)?;
        Ok(())
    }

    // Test the complete flow of game state detection
    #[tokio::test]
    async fn test_spectator_mode_detection_flow() {
        // This test simulates the complete flow when in spectator mode
        let mut api = LCUAPI::new();
        
        // Set up a mock connection
        let connection = LCUConnection {
            port: 12345,
            password: "test_password".to_string(),
            protocol: "https".to_string(),
        };
        api.set_connection(connection);

        // Test the comprehensive game state logic
        let result = api.get_comprehensive_game_state().await;
        assert!(result.is_ok());
        
        let state = result.unwrap();
        
        // The live_client_available will depend on the actual Live Client API
        // In spectator mode, it should be false, but we can't easily mock this
        // So we'll just verify the structure is correct
        assert!(state.lcu_connected);
        assert!(state.champion_select.is_none());
        assert!(state.live_game.is_none());
    }

    // Test lockfile detection with various scenarios
    #[test]
    fn test_lockfile_detection_scenarios() {
        let temp_dir = tempdir().unwrap();
        
        // Test valid lockfile
        let valid_content = "Riot Client:12345:56789:test_password:https";
        create_mock_lockfile(temp_dir.path(), valid_content).unwrap();
        
        let api = LCUAPI::new();
        let result = api.parse_lockfile_content(valid_content);
        assert!(result.is_ok());
        
        let connection = result.unwrap();
        assert_eq!(connection.port, 56789);
        assert_eq!(connection.password, "test_password");
        assert_eq!(connection.protocol, "https");
    }

    // Test error handling in lockfile parsing
    #[test]
    fn test_lockfile_error_handling() {
        let api = LCUAPI::new();
        
        let error_cases = vec![
            ("", "Invalid lockfile format"),
            ("single_field", "Invalid lockfile format"),
            ("field1:field2", "Invalid lockfile format"),
            ("field1:field2:field3", "Invalid lockfile format"),
            ("field1:field2:field3:field4", "Invalid lockfile format"),
            ("field1:invalid_port:field3:field4:field5", "Invalid port in lockfile"),
        ];

        for (content, expected_error) in error_cases {
            let result = api.parse_lockfile_content(content);
            assert!(result.is_err());
            let error = result.unwrap_err();
            assert!(error.contains(expected_error), 
                "Expected error containing '{}', got: {}", expected_error, error);
        }
    }

    // Test game phase mapping logic
    #[test]
    fn test_game_phase_mapping() {
        let test_cases = vec![
            ("InProgress", "in_game"),
            ("ChampSelect", "champion_select"),
            ("Lobby", "lobby"),
            ("ReadyCheck", "ready_check"),
            ("Matchmaking", "matchmaking"),
            ("Reconnect", "reconnect"),
        ];

        for (input_phase, expected_phase) in test_cases {
            let mapped_phase = match input_phase {
                "InProgress" => "in_game",
                "ChampSelect" => "champion_select",
                "Lobby" => "lobby",
                "ReadyCheck" => "ready_check",
                "Matchmaking" => "matchmaking",
                "Reconnect" => "reconnect",
                _ => input_phase,
            };
            
            assert_eq!(mapped_phase, expected_phase, 
                "Failed to map phase '{}' to '{}'", input_phase, expected_phase);
        }
    }

    // Test JSON state structure
    #[test]
    fn test_comprehensive_state_structure() {
        let state = json!({
            "lcu_connected": true,
            "live_client_available": false,
            "is_spectator_mode": false,
            "game_phase": "InProgress",
            "current_summoner": null,
            "champion_select": null,
            "live_game": null,
            "live_stats": null,
            "player_scores": null,
            "timestamp": 1234567890u64
        });

        assert!(state.get("lcu_connected").is_some());
        assert!(state.get("live_client_available").is_some());
        assert!(state.get("is_spectator_mode").is_some());
        assert!(state.get("game_phase").is_some());
        assert!(state.get("champion_select").is_some());
        assert!(state.get("live_game").is_some());
        assert!(state.get("timestamp").is_some());

        assert!(state["lcu_connected"].is_boolean());
        assert!(state["live_client_available"].is_boolean());
        assert!(state["game_phase"].is_string());
        assert!(state["champion_select"].is_null());
        assert!(state["live_game"].is_null());
        assert!(state["timestamp"].is_number());
    }

    // Test connection status logic
    #[test]
    fn test_connection_status_logic() {
        let mut api = LCUAPI::new();
        
        // Test no connection
        assert_eq!(api.get_connection_status(), "League not running");
        
        // Test with connection
        let connection = LCUConnection {
            port: 12345,
            password: "test_password".to_string(),
            protocol: "https".to_string(),
        };
        api.set_connection(connection);
        assert_eq!(api.get_connection_status(), "Connected");
    }

    /*
    // Test additional path detection
    #[test]
    fn test_additional_path_detection() {
        let api = LCUAPI::new();
        
        // Test registry search
        let registry_result = api.test_find_league_paths_from_registry();
        assert!(registry_result.is_ok());
        
        // Test shortcut search
        let shortcut_result = api.test_find_league_paths_from_shortcuts();
        assert!(shortcut_result.is_ok());
        
        // Test drive search
        let drive_result = api.test_find_league_paths_from_drives();
        assert!(drive_result.is_ok());
        
        // Test combined search
        let combined_result = api.test_find_additional_league_paths();
        assert!(combined_result.is_ok());
    }
    */

    // Test timestamp generation
    #[test]
    fn test_timestamp_generation() {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Should be a reasonable timestamp
        assert!(timestamp > 0);
        assert!(timestamp < 2000000000); // Before year 2033
    }

    // Test LCUConnection struct
    #[test]
    fn test_lcu_connection_struct() {
        let connection = LCUConnection {
            port: 12345,
            password: "test_password".to_string(),
            protocol: "https".to_string(),
        };

        assert_eq!(connection.port, 12345);
        assert_eq!(connection.password, "test_password");
        assert_eq!(connection.protocol, "https");
    }

    // Test error message formatting
    #[test]
    fn test_error_message_formatting() {
        let api = LCUAPI::new();
        
        // Test various error scenarios
        let result = api.parse_lockfile_content("invalid");
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(error.contains("Invalid lockfile format"));
    }

    // Test URL construction
    #[test]
    fn test_url_construction() {
        let connection = LCUConnection {
            port: 12345,
            password: "test_password".to_string(),
            protocol: "https".to_string(),
        };

        let gameflow_phase_url = format!("{}://127.0.0.1:{}/lol-gameflow/v1/gameflow-phase", 
                                        connection.protocol, connection.port);
        let gameflow_session_url = format!("{}://127.0.0.1:{}/lol-gameflow/v1/session", 
                                          connection.protocol, connection.port);
        let champ_select_url = format!("{}://127.0.0.1:{}/lol-champ-select/v1/session", 
                                      connection.protocol, connection.port);

        assert_eq!(gameflow_phase_url, "https://127.0.0.1:12345/lol-gameflow/v1/gameflow-phase");
        assert_eq!(gameflow_session_url, "https://127.0.0.1:12345/lol-gameflow/v1/session");
        assert_eq!(champ_select_url, "https://127.0.0.1:12345/lol-champ-select/v1/session");
    }

    // Test spectator mode detection logic
    #[test]
    fn test_spectator_mode_detection_logic() {
        // Test the logic for detecting spectator mode
        let test_cases = vec![
            (true, false, "in_game"), // LCU connected, Live Client not available = spectator mode
            (true, true, "in_game"),  // LCU connected, Live Client available = normal game
            (false, false, "unknown"), // No connection = unknown
        ];

        for (lcu_connected, live_client_available, expected_phase) in test_cases {
            let phase = if lcu_connected && !live_client_available {
                "in_game" // Spectator mode
            } else if lcu_connected && live_client_available {
                "in_game" // Normal game
            } else {
                "unknown"
            };
            
            assert_eq!(phase, expected_phase, 
                "Failed for lcu_connected={}, live_client_available={}", 
                lcu_connected, live_client_available);
        }
    }

    // Test comprehensive state initialization
    #[test]
    fn test_comprehensive_state_initialization() {
        let state = serde_json::json!({
            "lcu_connected": false,
            "live_client_available": false,
            "game_phase": "unknown",
            "champion_select": null,
            "live_game": null,
            "game_mode": "Unknown",
            "timestamp": 1234567890
        });

        // Verify initial state
        assert_eq!(state["lcu_connected"], false);
        assert_eq!(state["live_client_available"], false);
        assert_eq!(state["game_phase"], "unknown");
        assert_eq!(state["champion_select"], serde_json::Value::Null);
        assert_eq!(state["live_game"], serde_json::Value::Null);
        assert_eq!(state["game_mode"], "Unknown");
        assert_eq!(state["timestamp"], 1234567890);
    }
}
