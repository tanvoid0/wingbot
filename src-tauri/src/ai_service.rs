use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequest {
    pub session_id: String,
    pub message: String,
    pub context: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub response: String,
    pub confidence: Option<f64>,
    pub suggestions: Option<Vec<String>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionSelectRequest {
    pub session_id: String,
    pub message: String,
    pub champ_select_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionSelectResponse {
    pub best_pick: Vec<String>,
    pub counter_reasoning: String,
    pub synergy_notes: String,
    pub danger_notes: String,
    pub priority_order: Vec<String>,
    pub team_comp_analysis: String,
    pub tts_summary: String,
}

pub struct AIService {
    pub base_url: String,
    pub timeout: Duration,
}

impl Default for AIService {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8000".to_string(),
            timeout: Duration::from_secs(60),
        }
    }
}

impl AIService {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            timeout: Duration::from_secs(60),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub async fn generate_response(&self, request: AIRequest) -> Result<AIResponse, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/ai/chat", self.base_url);
        
        let response = timeout(
            self.timeout,
            client
                .post(&url)
                .json(&request)
                .send()
        ).await
        .map_err(|_| "Request timeout".to_string())?
        .map_err(|e| format!("Request failed: {}", e))?;

        if response.status().is_success() {
            let ai_response: AIResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(ai_response)
        } else {
            Err(format!("AI service error: {}", response.status()))
        }
    }

    pub async fn get_champion_suggestions(
        &self,
        request: ChampionSelectRequest,
    ) -> Result<ChampionSelectResponse, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/ai/champ-select", self.base_url);
        
        let response = timeout(
            self.timeout,
            client
                .post(&url)
                .json(&request)
                .send()
        ).await
        .map_err(|_| "Request timeout".to_string())?
        .map_err(|e| format!("Request failed: {}", e))?;

        if response.status().is_success() {
            let champ_response: ChampionSelectResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(champ_response)
        } else {
            Err(format!("Champion select AI service error: {}", response.status()))
        }
    }

    pub async fn get_live_game_suggestions(
        &self,
        session_id: String,
        message: String,
        game_data: serde_json::Value,
    ) -> Result<AIResponse, String> {
        let request = AIRequest {
            session_id,
            message,
            context: Some(HashMap::from([
                ("game_data".to_string(), game_data),
            ])),
        };
        
        self.generate_response(request).await
    }

    pub async fn is_server_available(&self) -> bool {
        let client = reqwest::Client::new();
        let url = format!("{}/health", self.base_url);
        
        match timeout(
            Duration::from_secs(5), // Shorter timeout for health check
            client.get(&url).send()
        ).await {
            Ok(Ok(response)) => response.status().is_success(),
            _ => false
        }
    }

    pub async fn get_server_status(&self) -> String {
        if self.is_server_available().await {
            "Connected".to_string()
        } else {
            "Disconnected".to_string()
        }
    }
}
