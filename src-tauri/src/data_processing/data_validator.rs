use serde_json::Value;

/// Handles validation of data structures and provides error handling
pub struct DataValidator;

impl DataValidator {
    /// Validate Live Client Data API response structure
    pub fn validate_live_client_data(data: &Value) -> Result<(), String> {
        // Check for required top-level fields
        let required_fields = ["gameData", "activePlayer"];
        
        for field in &required_fields {
            if !data.get(field).is_some() {
                return Err(format!("Missing required field: {}", field));
            }
        }
        
        // Validate game data structure
        if let Some(game_data) = data.get("gameData") {
            Self::validate_game_data(game_data)?;
        }
        
        // Validate active player structure
        if let Some(active_player) = data.get("activePlayer") {
            Self::validate_active_player(active_player)?;
        }
        
        // Validate all players if present
        if let Some(all_players) = data.get("allPlayers") {
            Self::validate_all_players(all_players)?;
        }
        
        Ok(())
    }
    
    /// Validate game data structure
    fn validate_game_data(game_data: &Value) -> Result<(), String> {
        if !game_data.is_object() {
            return Err("gameData must be an object".to_string());
        }
        
        // Check for game time (required for live games)
        if let Some(game_time) = game_data.get("gameTime") {
            if !game_time.is_number() {
                return Err("gameTime must be a number".to_string());
            }
        }
        
        Ok(())
    }
    
    /// Validate active player structure
    fn validate_active_player(active_player: &Value) -> Result<(), String> {
        if !active_player.is_object() {
            return Err("activePlayer must be an object".to_string());
        }
        
        // Check for champion name
        if let Some(champion_name) = active_player.get("championName") {
            if !champion_name.is_string() {
                return Err("championName must be a string".to_string());
            }
        }
        
        // Check for level
        if let Some(level) = active_player.get("level") {
            if !level.is_number() {
                return Err("level must be a number".to_string());
            }
        }
        
        Ok(())
    }
    
    /// Validate all players structure
    fn validate_all_players(all_players: &Value) -> Result<(), String> {
        if !all_players.is_array() {
            return Err("allPlayers must be an array".to_string());
        }
        
        if let Some(players_array) = all_players.as_array() {
            for (index, player) in players_array.iter().enumerate() {
                if !player.is_object() {
                    return Err(format!("Player at index {} must be an object", index));
                }
                
                // Check for required player fields
                if player.get("championName").is_none() {
                    return Err(format!("Player at index {} missing championName", index));
                }
                
                if player.get("team").is_none() {
                    return Err(format!("Player at index {} missing team", index));
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate events structure
    pub fn validate_events(events: &Value) -> Result<(), String> {
        if !events.is_array() {
            return Err("events must be an array".to_string());
        }
        
        if let Some(events_array) = events.as_array() {
            for (index, event) in events_array.iter().enumerate() {
                if !event.is_object() {
                    return Err(format!("Event at index {} must be an object", index));
                }
                
                // Check for required event fields
                if event.get("EventName").is_none() {
                    return Err(format!("Event at index {} missing EventName", index));
                }
                
                if event.get("EventTime").is_none() {
                    return Err(format!("Event at index {} missing EventTime", index));
                }
            }
        }
        
        Ok(())
    }
    
    /// Sanitize and clean data
    pub fn sanitize_data(data: &mut Value) -> Result<(), String> {
        match data {
            Value::Object(obj) => {
                // Remove null values
                obj.retain(|_, v| !v.is_null());
                
                // Recursively sanitize nested objects
                for (_, value) in obj.iter_mut() {
                    Self::sanitize_data(value)?;
                }
            }
            Value::Array(arr) => {
                // Remove null values from arrays
                arr.retain(|v| !v.is_null());
                
                // Recursively sanitize array elements
                for value in arr.iter_mut() {
                    Self::sanitize_data(value)?;
                }
            }
            _ => {} // Primitive values don't need sanitization
        }
        
        Ok(())
    }
    
    /// Check if data is valid for spectator mode
    pub fn is_spectator_mode_data(data: &Value) -> bool {
        // In spectator mode, we might have limited data
        // Check if we have at least game data and some player info
        data.get("gameData").is_some() && 
        (data.get("activePlayer").is_some() || data.get("allPlayers").is_some())
    }
    
    /// Get data quality score (0.0 to 1.0)
    pub fn get_data_quality_score(data: &Value) -> f64 {
        let mut score = 0.0;
        let mut total_checks = 0.0;
        
        // Check for game data
        if data.get("gameData").is_some() {
            score += 1.0;
        }
        total_checks += 1.0;
        
        // Check for active player
        if data.get("activePlayer").is_some() {
            score += 1.0;
        }
        total_checks += 1.0;
        
        // Check for all players
        if data.get("allPlayers").is_some() {
            score += 1.0;
        }
        total_checks += 1.0;
        
        // Check for events
        if data.get("events").is_some() {
            score += 1.0;
        }
        total_checks += 1.0;
        
        if total_checks > 0.0 {
            score / total_checks
        } else {
            0.0
        }
    }
}
