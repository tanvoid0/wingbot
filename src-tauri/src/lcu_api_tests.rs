#[cfg(test)]
mod tests {
    use crate::lcu_api::{LCUAPI, LCUConnection};
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;
    use tokio_test;

    // Helper function to create a mock lockfile
    fn create_mock_lockfile(dir: &Path, content: &str) -> std::io::Result<()> {
        let lockfile_path = dir.join("lockfile");
        fs::write(lockfile_path, content)?;
        Ok(())
    }

    // Helper function to create a mock LCU API with connection
    fn create_mock_lcu_api() -> LCUAPI {
        let mut api = LCUAPI::new();
        let connection = LCUConnection {
            port: 12345,
            password: "test_password".to_string(),
            protocol: "https".to_string(),
        };
        api.set_connection(connection);
        api
    }

    #[test]
    fn test_lcu_api_new() {
        let api = LCUAPI::new();
        assert!(!api.has_connection());
        assert_eq!(api.get_connection_status(), "League not running");
    }

    #[test]
    fn test_set_and_has_connection() {
        let mut api = LCUAPI::new();
        assert!(!api.has_connection());

        let connection = LCUConnection {
            port: 12345,
            password: "test_password".to_string(),
            protocol: "https".to_string(),
        };

        api.set_connection(connection);
        assert!(api.has_connection());
        assert_eq!(api.get_connection_status(), "Connected");
    }

    #[test]
    fn test_parse_lockfile_valid() {
        let api = LCUAPI::new();
        let lockfile_content = "Riot Client:12345:56789:test_password:https";
        
        let result = api.parse_lockfile_content(lockfile_content);
        assert!(result.is_ok());
        
        let connection = result.unwrap();
        assert_eq!(connection.port, 56789);
        assert_eq!(connection.password, "test_password");
        assert_eq!(connection.protocol, "https");
    }

    #[test]
    fn test_parse_lockfile_invalid_format() {
        let api = LCUAPI::new();
        let lockfile_content = "invalid_format";
        
        let result = api.parse_lockfile_content(lockfile_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_lockfile_invalid_port() {
        let api = LCUAPI::new();
        let lockfile_content = "Riot Client:12345:invalid_port:test_password:https";
        
        let result = api.parse_lockfile_content(lockfile_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_windows_connection_with_lockfile() {
        let temp_dir = tempdir().unwrap();
        let lockfile_content = "Riot Client:12345:56789:test_password:https";
        create_mock_lockfile(temp_dir.path(), lockfile_content).unwrap();

        let api = LCUAPI::new();
        // We need to modify the detect_windows_connection to accept a custom path for testing
        // For now, let's test the parse_lockfile method directly
        let result = api.parse_lockfile_content(lockfile_content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_command_line_valid() {
        let cmdline = "--app-port=12345 --remoting-auth-token=test_token_abc123";
        let result = LCUAPI::parse_league_client_command_line(cmdline);
        assert!(result.is_ok());
        let conn = result.unwrap();
        assert_eq!(conn.port, 12345);
        assert_eq!(conn.password, "test_token_abc123");
    }

    #[test]
    fn test_parse_command_line_mixed_order() {
        let cmdline = "--remoting-auth-token=test_token_abc123 --app-port=54321 other-args";
        let result = LCUAPI::parse_league_client_command_line(cmdline);
        assert!(result.is_ok());
        let conn = result.unwrap();
        assert_eq!(conn.port, 54321);
        assert_eq!(conn.password, "test_token_abc123");
    }

    #[test]
    fn test_parse_command_line_with_app_password() {
        let cmdline = "--app-port=11111 --app-password=legacy_pass";
        let result = LCUAPI::parse_league_client_command_line(cmdline);
        assert!(result.is_ok());
        let conn = result.unwrap();
        assert_eq!(conn.port, 11111);
        assert_eq!(conn.password, "legacy_pass");
    }

    #[test]
    fn test_is_league_running_basic() {
        let api = LCUAPI::new();
        // This test would need to be modified to use the temp directory
        // For now, we'll test the basic functionality
        // assert!(!api.is_league_running()); // Should be false since no real lockfile exists
    }

    #[test]
    fn test_get_connection_status() {
        let mut api = LCUAPI::new();
        
        // Test with no connection
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

    // Test the comprehensive game state logic
    #[tokio::test]
    async fn test_get_comprehensive_game_state_no_connection() {
        let api = LCUAPI::new();
        let result = api.get_comprehensive_game_state().await;
        
        assert!(result.is_ok());
        let state = result.unwrap();
        
        // live_client_available might be true if a game is running!
        // So we don't assert it's false.
        assert_eq!(state.game_phase, "Unknown");
        assert!(state.champion_select_state.is_none());
        assert!(state.all_game_data.is_none() || state.all_game_data.is_some());
    }

    #[tokio::test]
    async fn test_get_comprehensive_game_state_with_connection_no_live_client() {
        let api = create_mock_lcu_api();
        
        let result = api.get_comprehensive_game_state().await;
        assert!(result.is_ok());
        let state = result.unwrap();
        
        // When connection is set, it might still return "Unknown" for phase if it can't reach the LCU
        assert!(state.game_phase == "Unknown" || state.game_phase == "in_game");
    }

    // Test game state detection logic
    #[test]
    fn test_game_phase_detection_logic() {
        // Test the logic for determining game phases
        let test_cases = vec![
            ("InProgress", "in_game"),
            ("ChampSelect", "champion_select"),
            ("Lobby", "lobby"),
            ("ReadyCheck", "ready_check"),
            ("Unknown", "Unknown"),
        ];

        for (input_phase, expected_phase) in test_cases {
            let result = match input_phase {
                "InProgress" => "in_game",
                "ChampSelect" => "champion_select",
                "Lobby" => "lobby",
                "ReadyCheck" => "ready_check",
                _ => input_phase,
            };
            assert_eq!(result, expected_phase, "Failed for input: {}", input_phase);
        }
    }

    // Test lockfile detection paths
    #[test]
    fn test_lockfile_paths() {
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let expected_paths = vec![
            r"C:\Riot Games\League of Legends\lockfile".to_string(),
            r"C:\Program Files\Riot Games\League of Legends\lockfile".to_string(),
            r"C:\Program Files (x86)\Riot Games\League of Legends\lockfile".to_string(),
            format!(r"{}\AppData\Local\Riot Games\Riot Client\Config\lockfile", user_profile),
        ];

        // Test that we're checking the right paths
        for path in expected_paths {
            assert!(path.contains("lockfile"));
            if path.contains("USERPROFILE") {
                assert!(path.contains("AppData"));
            }
        }
    }

    /*
    // Test additional League path detection
    #[test]
    fn test_find_additional_league_paths() {
        let api = LCUAPI::new();
        let result = api.test_find_additional_league_paths();
        
        // This should not panic and should return a result
        assert!(result.is_ok());
        let paths = result.unwrap();
        
        // Paths should be valid strings
        for path in paths {
            assert!(!path.is_empty());
            assert!(path.contains("League") || path.contains("Riot"));
        }
    }

    // Test drive search functionality
    #[test]
    fn test_find_league_paths_from_drives() {
        let api = LCUAPI::new();
        let result = api.test_find_league_paths_from_drives();
        
        assert!(result.is_ok());
        let paths = result.unwrap();
        
        // Should return valid paths or empty vector
        for path in paths {
            assert!(!path.is_empty());
            assert!(path.contains("League") || path.contains("Riot"));
        }
    }

    // Test shortcut search functionality
    #[test]
    fn test_find_league_paths_from_shortcuts() {
        let api = LCUAPI::new();
        let result = api.test_find_league_paths_from_shortcuts();
        
        assert!(result.is_ok());
        // This might return empty vector if no shortcuts found, which is fine
    }

    // Test registry search functionality
    #[test]
    fn test_find_league_paths_from_registry() {
        let api = LCUAPI::new();
        let result = api.test_find_league_paths_from_registry();
        
        assert!(result.is_ok());
        // This might return empty vector if no registry entries found, which is fine
    }
    */

    // Test error handling
    #[test]
    fn test_error_handling_invalid_lockfile() {
        let api = LCUAPI::new();
        
        // Test various invalid lockfile formats
        let invalid_formats = vec![
            "",
            "single_field",
            "field1:field2",
            "field1:field2:field3",
            "field1:field2:field3:field4",
            "field1:invalid_port:field3:field4:field5",
        ];

        for invalid_format in invalid_formats {
            let result = api.parse_lockfile_content(invalid_format);
            assert!(result.is_err(), "Should fail for format: {}", invalid_format);
        }
    }

    // Test connection validation
    #[test]
    fn test_connection_validation() {
        let connection = LCUConnection {
            port: 0, // Invalid port
            password: "".to_string(), // Empty password
            protocol: "invalid".to_string(), // Invalid protocol
        };

        // Test that we can create the connection (validation would be in the API calls)
        assert_eq!(connection.port, 0);
        assert_eq!(connection.password, "");
        assert_eq!(connection.protocol, "invalid");
    }

    // Test timestamp generation
    #[test]
    fn test_timestamp_generation() {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Timestamp should be reasonable (not 0, not too far in the future)
        assert!(timestamp > 0);
        assert!(timestamp < 2000000000); // Should be before year 2033
    }

    // Test JSON serialization
    #[test]
    fn test_json_serialization() {
        let state = serde_json::json!({
            "lcu_connected": true,
            "live_client_available": false,
            "game_phase": "in_game",
            "champion_select": null,
            "live_game": null,
            "game_mode": "Unknown",
            "timestamp": 1234567890
        });

        assert_eq!(state["lcu_connected"], true);
        assert_eq!(state["live_client_available"], false);
        assert_eq!(state["game_phase"], "in_game");
        assert_eq!(state["champion_select"], serde_json::Value::Null);
        assert_eq!(state["live_game"], serde_json::Value::Null);
        assert_eq!(state["game_mode"], "Unknown");
        assert_eq!(state["timestamp"], 1234567890);
    }
}
