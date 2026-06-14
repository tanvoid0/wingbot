use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use tokio::sync::{broadcast, RwLock};
use std::sync::Arc;
use base64::{Engine as _, engine::general_purpose};
use serde_json::Value;

use crate::lcu_api::LCUConnection;

#[derive(Debug, Clone)]
pub struct LcuWebSocket {
    connection: Option<LCUConnection>,
    event_sender: broadcast::Sender<Value>,
    is_connected: Arc<RwLock<bool>>,
}

impl LcuWebSocket {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            connection: None,
            event_sender: tx,
            is_connected: Arc::new(RwLock::new(false)),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Value> {
        self.event_sender.subscribe()
    }

    pub async fn is_connected(&self) -> bool {
        *self.is_connected.read().await
    }

    pub async fn connect(&mut self, connection: LCUConnection) -> Result<(), String> {
        println!("[LCU WebSocket] Connecting to wss://127.0.0.1:{}...", connection.port);
        
        self.connection = Some(connection.clone());
        
        // Build WebSocket URL
        let ws_url = format!("wss://127.0.0.1:{}", connection.port);
        
        // Create authorization header
        let auth_string = format!("riot:{}", connection.password);
        let auth_encoded = general_purpose::STANDARD.encode(auth_string.as_bytes());
        
        // Build request with headers
        use tokio_tungstenite::tungstenite::http::Request;
        let request = Request::builder()
            .uri(&ws_url)
            .header("Authorization", format!("Basic {}", auth_encoded))
            .header("Host", format!("127.0.0.1:{}", connection.port))
            .body(())
            .map_err(|e| format!("Failed to create WebSocket request: {}", e))?;
        
        // Connect to WebSocket with TLS
        let (ws_stream, _) = connect_async(request)
            .await
            .map_err(|e| format!("Failed to connect to LCU WebSocket: {}", e))?;
        
        println!("[LCU WebSocket] Connected successfully!");
        
        let (mut write, mut read) = ws_stream.split();
        
        // Subscribe to all events
        let subscribe_msg = "[5, \"OnJsonApiEvent\"]";
        write
            .send(Message::Binary(subscribe_msg.as_bytes().to_vec()))
            .await
            .map_err(|e| format!("Failed to subscribe to events: {}", e))?;
        
        println!("[LCU WebSocket] Subscribed to OnJsonApiEvent");
        
        // Update connection status
        *self.is_connected.write().await = true;
        
        // Spawn task to handle incoming messages
        let event_sender = self.event_sender.clone();
        let is_connected = self.is_connected.clone();
        
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        // Parse the WebSocket message
                        // Format is typically: [8, "OnJsonApiEvent", {...}]
                        if let Ok(parsed) = serde_json::from_str::<Value>(&text) {
                            Self::handle_websocket_message(parsed, &event_sender).await;
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        // Handle binary messages (LCU might send binary)
                        if let Ok(text) = String::from_utf8(data) {
                            if let Ok(parsed) = serde_json::from_str::<Value>(&text) {
                                Self::handle_websocket_message(parsed, &event_sender).await;
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        println!("[LCU WebSocket] Connection closed by server");
                        *is_connected.write().await = false;
                        break;
                    }
                    Err(e) => {
                        println!("[LCU WebSocket] Error receiving message: {}", e);
                        *is_connected.write().await = false;
                        break;
                    }
                    _ => {}
                }
            }
            
            println!("[LCU WebSocket] Event listener stopped");
        });
        
        Ok(())
    }

    pub async fn disconnect(&mut self) {
        println!("[LCU WebSocket] Disconnecting...");
        *self.is_connected.write().await = false;
        self.connection = None;
    }

    async fn handle_websocket_message(parsed: Value, event_sender: &broadcast::Sender<Value>) {
        if let Some(arr) = parsed.as_array() {
            if arr.len() >= 3 {
                // arr[0] is message type (8 for events)
                // arr[1] is event name ("OnJsonApiEvent")
                // arr[2] is the actual event data
                if let Some(event_data) = arr.get(2) {
                    // Send event to all subscribers
                    let _ = event_sender.send(event_data.clone());
                    
                    // Log event for debugging
                    if let Some(event_obj) = event_data.as_object() {
                        if let (Some(event_type), Some(uri)) = (
                            event_obj.get("eventType").and_then(|v| v.as_str()),
                            event_obj.get("uri").and_then(|v| v.as_str()),
                        ) {
                            println!("[LCU WebSocket] Event: {} {}", event_type, uri);
                        }
                    }
                }
            }
        }
    }
}


