use serde_json::{Value, Map};

/// Handles detection of changes between data states for efficient delta updates
pub struct DeltaDetector;

impl DeltaDetector {
    /// Detect changes in game state with priority field handling
    pub fn detect_game_state_changes(current: &Value, previous: &Option<Value>) -> Option<Value> {
        if let Some(prev) = previous {
            if current == prev {
                return None; // No changes
            }
            
            let changes = Self::extract_changes(current, prev);
            
            if changes.is_empty() {
                None
            } else {
                println!("[DeltaDetector] Game state changes detected: {} fields changed", changes.len());
                Some(Value::Object(changes))
            }
        } else {
            // First time, send full state
            println!("[DeltaDetector] Sending initial game state");
            Some(current.clone())
        }
    }
    
    /// Detect changes in champion select session data.
    pub fn detect_champion_select_changes(current: &Value, previous: &Option<Value>) -> Option<Value> {
        if let Some(prev) = previous {
            if current == prev {
                return None;
            }

            let changes = Self::extract_champion_select_changes(current, prev);

            if changes.is_empty() {
                None
            } else {
                Some(Value::Object(changes))
            }
        } else {
            Some(current.clone())
        }
    }

    /// Detect changes in live game data with focus on key fields
    pub fn detect_live_game_data_changes(current: &Value, previous: &Option<Value>) -> Option<Value> {
        if let Some(prev) = previous {
            if current == prev {
                return None; // No changes
            }
            
            let changes = Self::extract_priority_changes(current, prev);
            
            if changes.is_empty() {
                None
            } else {
                println!("[DeltaDetector] Live game data changes detected: {} fields changed", changes.len());
                Some(Value::Object(changes))
            }
        } else {
            // First time, send full state
            println!("[DeltaDetector] Sending initial live game data");
            Some(current.clone())
        }
    }
    
    /// Extract all changes between current and previous state
    fn extract_changes(current: &Value, previous: &Value) -> Map<String, Value> {
        let mut changes = Map::new();
        
        if let (Some(current_obj), Some(prev_obj)) = (current.as_object(), previous.as_object()) {
            // Priority fields that should always be checked for changes
            let priority_fields = ["gameData", "activePlayer", "allPlayers", "events", "timestamp"];
            
            // Check priority fields first
            for field in &priority_fields {
                if let (Some(current_value), Some(prev_value)) = (current_obj.get(*field), prev_obj.get(*field)) {
                    if current_value != prev_value {
                        changes.insert(field.to_string(), current_value.clone());
                    }
                } else if let Some(current_value) = current_obj.get(*field) {
                    // New field
                    changes.insert(field.to_string(), current_value.clone());
                }
            }
            
            // Check other fields for changes
            for (key, current_value) in current_obj {
                if !priority_fields.contains(&key.as_str()) {
                    if let Some(prev_value) = prev_obj.get(key) {
                        if current_value != prev_value {
                            changes.insert(key.clone(), current_value.clone());
                        }
                    } else {
                        // New field
                        changes.insert(key.clone(), current_value.clone());
                    }
                }
            }
        }
        
        changes
    }
    
    fn extract_champion_select_changes(current: &Value, previous: &Value) -> Map<String, Value> {
        let mut changes = Map::new();

        if let (Some(current_obj), Some(prev_obj)) = (current.as_object(), previous.as_object()) {
            let key_fields = ["timer", "actions", "myTeam", "theirTeam", "bans", "localPlayerCellId"];

            for field in &key_fields {
                if let (Some(current_value), Some(prev_value)) =
                    (current_obj.get(*field), prev_obj.get(*field))
                {
                    if current_value != prev_value {
                        changes.insert(field.to_string(), current_value.clone());
                    }
                } else if let Some(current_value) = current_obj.get(*field) {
                    changes.insert(field.to_string(), current_value.clone());
                }
            }

            for (key, current_value) in current_obj {
                if key_fields.contains(&key.as_str()) {
                    continue;
                }
                if let Some(prev_value) = prev_obj.get(key) {
                    if current_value != prev_value {
                        changes.insert(key.clone(), current_value.clone());
                    }
                } else {
                    changes.insert(key.clone(), current_value.clone());
                }
            }
        }

        changes
    }

    /// Extract priority changes for live game data
    fn extract_priority_changes(current: &Value, previous: &Value) -> Map<String, Value> {
        let mut changes = Map::new();
        
        if let (Some(current_obj), Some(prev_obj)) = (current.as_object(), previous.as_object()) {
            // Key fields that change frequently in live game data
            let key_fields = ["gameTime", "activePlayer", "allPlayers", "events"];
            
            for field in &key_fields {
                if let (Some(current_value), Some(prev_value)) = (current_obj.get(*field), prev_obj.get(*field)) {
                    if current_value != prev_value {
                        changes.insert(field.to_string(), current_value.clone());
                    }
                } else if let Some(current_value) = current_obj.get(*field) {
                    // New field
                    changes.insert(field.to_string(), current_value.clone());
                }
            }
            
            // Special handling for game time - always include if it exists
            if let Some(game_time) = current_obj.get("gameTime") {
                if let Some(prev_time) = prev_obj.get("gameTime") {
                    if game_time != prev_time {
                        changes.insert("gameTime".to_string(), game_time.clone());
                    }
                } else {
                    changes.insert("gameTime".to_string(), game_time.clone());
                }
            }
        }
        
        changes
    }
    
    /// Check if two values are significantly different (for performance optimization)
    pub fn is_significantly_different(current: &Value, previous: &Value, threshold: f64) -> bool {
        match (current, previous) {
            (Value::Number(current_num), Value::Number(prev_num)) => {
                if let (Some(current_f64), Some(prev_f64)) = (current_num.as_f64(), prev_num.as_f64()) {
                    (current_f64 - prev_f64).abs() > threshold
                } else {
                    current != previous
                }
            }
            _ => current != previous
        }
    }
}
