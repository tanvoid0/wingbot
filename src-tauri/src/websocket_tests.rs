use super::websocket_server::{WebSocketServer, WebSocketMessage};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use serde_json;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_websocket_server_startup() {
    let server = WebSocketServer::new(Arc::new(AtomicBool::new(true)));
    
    // Test that server can be created
    assert_eq!(server.get_client_count().await, 0);
}

#[tokio::test]
async fn test_websocket_message_types() {
    // Test Subscribe message
    let subscribe_msg = WebSocketMessage::Subscribe {
        topics: vec!["game_state".to_string(), "live_game".to_string()],
    };
    let serialized = serde_json::to_string(&subscribe_msg).unwrap();
    let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
    
    match deserialized {
        WebSocketMessage::Subscribe { topics } => {
            assert_eq!(topics.len(), 2);
            assert!(topics.contains(&"game_state".to_string()));
            assert!(topics.contains(&"live_game".to_string()));
        }
        _ => panic!("Wrong message type"),
    }
    
    // Test GameStateUpdate message
    let game_state_msg = WebSocketMessage::GameStateUpdate {
        data: serde_json::json!({
            "game_phase": "in_game",
            "lcu_connected": true,
            "live_client_available": true
        }),
    };
    let serialized = serde_json::to_string(&game_state_msg).unwrap();
    let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
    
    match deserialized {
        WebSocketMessage::GameStateUpdate { data } => {
            assert_eq!(data["game_phase"], "in_game");
            assert_eq!(data["lcu_connected"], true);
        }
        _ => panic!("Wrong message type"),
    }
    
    // Test LCUConnectionUpdate message
    let lcu_msg = WebSocketMessage::LCUConnectionUpdate {
        connected: true,
        status: "Connected".to_string(),
    };
    let serialized = serde_json::to_string(&lcu_msg).unwrap();
    let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
    
    match deserialized {
        WebSocketMessage::LCUConnectionUpdate { connected, status } => {
            assert_eq!(connected, true);
            assert_eq!(status, "Connected");
        }
        _ => panic!("Wrong message type"),
    }
    
    // Test Error message
    let error_msg = WebSocketMessage::Error {
        message: "Test error".to_string(),
    };
    let serialized = serde_json::to_string(&error_msg).unwrap();
    let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
    
    match deserialized {
        WebSocketMessage::Error { message } => {
            assert_eq!(message, "Test error");
        }
        _ => panic!("Wrong message type"),
    }
    
    // Test Ping/Pong messages
    let ping_msg = WebSocketMessage::Ping {};
    let serialized = serde_json::to_string(&ping_msg).unwrap();
    let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
    
    match deserialized {
        WebSocketMessage::Ping {} => {
            // Success
        }
        _ => panic!("Wrong message type"),
    }
    
    let pong_msg = WebSocketMessage::Pong {};
    let serialized = serde_json::to_string(&pong_msg).unwrap();
    let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
    
    match deserialized {
        WebSocketMessage::Pong {} => {
            // Success
        }
        _ => panic!("Wrong message type"),
    }
}

/// Pins the wire format against the literal bytes the frontend sends and reads,
/// rather than round-tripping through our own serializer — a round-trip passes
/// whatever shape we happen to emit, which is how the heartbeat broke unnoticed.
/// websocketStore.ts sends `{"Ping":{}}` and dispatches on `message.Pong`.
#[test]
fn test_heartbeat_matches_frontend_wire_format() {
    // The client's heartbeat must deserialize. As a unit variant this was
    // `Err("invalid type: map, expected unit")`, so last_ping never refreshed
    // and ping_clients evicted every client 60s after it connected.
    let parsed: WebSocketMessage = serde_json::from_str(r#"{"Ping":{}}"#)
        .expect("frontend heartbeat {\"Ping\":{}} must deserialize");
    assert!(matches!(parsed, WebSocketMessage::Ping {}));

    // The reply must be an object the client can dispatch on, not the bare
    // string "Pong" — `message.Pong` is undefined on a string.
    let pong = serde_json::to_string(&WebSocketMessage::Pong {}).unwrap();
    assert_eq!(pong, r#"{"Pong":{}}"#);

    // Every other variant is already object-shaped; keep it that way.
    let update = serde_json::to_string(&WebSocketMessage::GameTimeUpdate { game_time: 1.0 }).unwrap();
    assert!(update.starts_with(r#"{"GameTimeUpdate":"#));
}

#[tokio::test]
async fn test_websocket_client_connection() {
    // This test requires a running server, so we'll test the connection logic
    // without actually starting a server to avoid port conflicts
    
    let server = WebSocketServer::new(Arc::new(AtomicBool::new(true)));
    
    // Test that we can create a server instance
    assert_eq!(server.get_client_count().await, 0);
    
    // Test that the server has the required components
    // (This is more of a smoke test since we can't easily test the full connection)
}

#[tokio::test]
async fn test_websocket_subscription_management() {
    let server = WebSocketServer::new(Arc::new(AtomicBool::new(true)));
    
    // Test subscription message handling
    let subscribe_msg = WebSocketMessage::Subscribe {
        topics: vec!["game_state".to_string(), "live_game".to_string()],
    };
    
    // Test unsubscribe message handling
    let unsubscribe_msg = WebSocketMessage::Unsubscribe {
        topics: vec!["game_state".to_string()],
    };
    
    // Test that messages can be serialized/deserialized
    let subscribe_serialized = serde_json::to_string(&subscribe_msg).unwrap();
    let unsubscribe_serialized = serde_json::to_string(&unsubscribe_msg).unwrap();
    
    let subscribe_deserialized: WebSocketMessage = serde_json::from_str(&subscribe_serialized).unwrap();
    let unsubscribe_deserialized: WebSocketMessage = serde_json::from_str(&unsubscribe_serialized).unwrap();
    
    match subscribe_deserialized {
        WebSocketMessage::Subscribe { topics } => {
            assert_eq!(topics.len(), 2);
        }
        _ => panic!("Wrong message type"),
    }
    
    match unsubscribe_deserialized {
        WebSocketMessage::Unsubscribe { topics } => {
            assert_eq!(topics.len(), 1);
            assert_eq!(topics[0], "game_state");
        }
        _ => panic!("Wrong message type"),
    }
}

#[tokio::test]
async fn test_websocket_broadcast_message_types() {
    // Test that all broadcast message types can be serialized
    let messages = vec![
        WebSocketMessage::GameStateUpdate {
            data: serde_json::json!({
                "game_phase": "champion_select",
                "lcu_connected": true,
                "live_client_available": false
            }),
        },
        WebSocketMessage::LCUConnectionUpdate {
            connected: true,
            status: "Connected".to_string(),
        },
        WebSocketMessage::LiveGameDataUpdate {
            data: serde_json::json!({
                "gameData": {
                    "gameTime": 120.5,
                    "gameMode": "Classic"
                },
                "allPlayers": []
            }),
        },
        WebSocketMessage::GameEvent {
            event_type: "ChampionKill".to_string(),
            data: serde_json::json!({
                "killerName": "TestPlayer",
                "victimName": "EnemyPlayer",
                "timestamp": 120.5
            }),
        },
        WebSocketMessage::Error {
            message: "Test error message".to_string(),
        },
    ];
    
    for msg in messages {
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
        
        // Verify that the message can be round-tripped
        match (&msg, &deserialized) {
            (WebSocketMessage::GameStateUpdate { data: data1 }, WebSocketMessage::GameStateUpdate { data: data2 }) => {
                assert_eq!(data1, data2);
            }
            (WebSocketMessage::LCUConnectionUpdate { connected: c1, status: s1 }, WebSocketMessage::LCUConnectionUpdate { connected: c2, status: s2 }) => {
                assert_eq!(c1, c2);
                assert_eq!(s1, s2);
            }
            (WebSocketMessage::LiveGameDataUpdate { data: data1 }, WebSocketMessage::LiveGameDataUpdate { data: data2 }) => {
                assert_eq!(data1, data2);
            }
            (WebSocketMessage::GameEvent { event_type: et1, data: data1 }, WebSocketMessage::GameEvent { event_type: et2, data: data2 }) => {
                assert_eq!(et1, et2);
                assert_eq!(data1, data2);
            }
            (WebSocketMessage::Error { message: m1 }, WebSocketMessage::Error { message: m2 }) => {
                assert_eq!(m1, m2);
            }
            _ => panic!("Message type mismatch"),
        }
    }
}

#[tokio::test]
async fn test_websocket_message_validation() {
    // Test that invalid JSON is handled gracefully
    let invalid_json = r#"{"type": "InvalidMessage", "data": "test"}"#;
    let result: Result<WebSocketMessage, _> = serde_json::from_str(invalid_json);
    
    // Should fail to deserialize
    assert!(result.is_err());
    
    // Test that valid but unknown message types are handled
    let unknown_message = r#"{"UnknownMessage": {"data": "test"}}"#;
    let result: Result<WebSocketMessage, _> = serde_json::from_str(unknown_message);
    
    // Should fail to deserialize
    assert!(result.is_err());
}

#[tokio::test]
async fn test_websocket_performance() {
    // Test that we can create and serialize many messages quickly
    let start = std::time::Instant::now();
    
    for _ in 0..1000 {
        let msg = WebSocketMessage::GameStateUpdate {
            data: serde_json::json!({
                "game_phase": "in_game",
                "lcu_connected": true,
                "live_client_available": true,
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            }),
        };
        
        let serialized = serde_json::to_string(&msg).unwrap();
        let _deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
    }
    
    let duration = start.elapsed();
    println!("Serialized/deserialized 1000 messages in {:?}", duration);
    
    // Should complete in reasonable time (less than 1 second)
    assert!(duration < Duration::from_secs(1));
}

// Integration test that would require a running server
#[tokio::test]
#[ignore] // Ignore by default since it requires a running server
async fn test_websocket_integration() {
    // This test would connect to a running WebSocket server and test
    // the full communication flow, but it's marked as ignored since
    // it requires a running server instance
    
    let server_url = "ws://127.0.0.1:8765";
    
    match timeout(Duration::from_secs(5), connect_async(server_url)).await {
        Ok(Ok((ws_stream, _))) => {
            let (mut ws_sender, mut ws_receiver) = ws_stream.split();
            
            // Send a ping message
            let ping_msg = WebSocketMessage::Ping {};
            let ping_json = serde_json::to_string(&ping_msg).unwrap();
            ws_sender.send(Message::Text(ping_json)).await.unwrap();
            
            // Wait for pong response
            if let Some(msg) = ws_receiver.next().await {
                match msg.unwrap() {
                    Message::Text(text) => {
                        let response: WebSocketMessage = serde_json::from_str(&text).unwrap();
                        match response {
                            WebSocketMessage::Pong {} => {
                                println!("Received pong response");
                            }
                            _ => panic!("Expected pong, got {:?}", response),
                        }
                    }
                    _ => panic!("Expected text message"),
                }
            }
        }
        Ok(Err(e)) => {
            panic!("Failed to connect to WebSocket server: {}", e);
        }
        Err(_) => {
            panic!("Connection timeout - server may not be running");
        }
    }
}

#[tokio::test]
async fn test_websocket_error_handling() {
    let server = WebSocketServer::new(Arc::new(AtomicBool::new(true)));
    
    // Test error message creation and serialization
    let error_msg = WebSocketMessage::Error {
        message: "Connection failed".to_string(),
    };
    
    let serialized = serde_json::to_string(&error_msg).unwrap();
    let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
    
    match deserialized {
        WebSocketMessage::Error { message } => {
            assert_eq!(message, "Connection failed");
        }
        _ => panic!("Wrong message type"),
    }
}

#[tokio::test]
async fn test_websocket_live_game_data_format() {
    // Test that live game data messages have the expected format
    let live_game_data = serde_json::json!({
        "gameData": {
            "gameTime": 300.5,
            "gameMode": "Classic",
            "mapName": "Summoner's Rift"
        },
        "allPlayers": [
            {
                "summonerName": "TestPlayer",
                "championName": "Ahri",
                "team": "ORDER",
                "level": 6,
                "items": [1001, 2003, 3001]
            }
        ],
        "events": {
            "Events": [
                {
                    "EventName": "ChampionKill",
                    "EventTime": 120.5,
                    "KillerName": "TestPlayer",
                    "VictimName": "EnemyPlayer"
                }
            ]
        }
    });
    
    let msg = WebSocketMessage::LiveGameDataUpdate {
        data: live_game_data.clone(),
    };
    
    let serialized = serde_json::to_string(&msg).unwrap();
    let deserialized: WebSocketMessage = serde_json::from_str(&serialized).unwrap();
    
    match deserialized {
        WebSocketMessage::LiveGameDataUpdate { data } => {
            assert_eq!(data["gameData"]["gameTime"], 300.5);
            assert_eq!(data["gameData"]["gameMode"], "Classic");
            assert!(data["allPlayers"].is_array());
            assert!(data["events"]["Events"].is_array());
        }
        _ => panic!("Wrong message type"),
    }
}
