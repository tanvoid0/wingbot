use serde_json::{Value, Map, Number, json};
use chrono::Utc;

/// Handles shaping and transforming raw Live Client Data API responses
/// into optimized structures for frontend consumption
pub struct GameDataShaper;

impl GameDataShaper {
    /// Shape raw Live Client Data API response for better frontend processing
    pub fn shape_live_client_data(raw_data: &Value) -> Result<Value, String> {
        let mut shaped = Map::new();
        
        // Extract and validate game data
        if let Some(game_data) = Self::extract_game_data(raw_data)? {
            shaped.insert("gameData".to_string(), game_data);
        }
        
        // Extract and validate active player data
        if let Some(active_player) = Self::extract_active_player(raw_data)? {
            shaped.insert("activePlayer".to_string(), active_player);
        }
        
        // Extract and validate all players data
        if let Some(all_players) = Self::extract_all_players(raw_data)? {
            shaped.insert("allPlayers".to_string(), all_players);
        }
        
        // Extract events (simplified)
        if let Some(events) = Self::extract_events(raw_data)? {
            shaped.insert("events".to_string(), events);
        }
        
        // Add metadata
        Self::add_metadata(&mut shaped);
        
        Ok(Value::Object(shaped))
    }

    /// Extract CS tracker data from active player
    pub fn extract_cs_data(active_player: &Value, game_time: f64) -> Value {
        let cs = active_player.get("scores")
            .and_then(|s| s.get("creepScore"))
            .and_then(|cs| cs.as_u64())
            .unwrap_or(0);
        
        let cs_per_min = if game_time > 0.0 {
            (cs as f64) / (game_time / 60.0)
        } else {
            0.0
        };
        
        let target = Self::calculate_target_cs(game_time);
        let gold_from_cs = cs * 20; // Approximate gold per CS
        let benchmark = Self::get_cs_benchmark(cs_per_min);
        
        json!({
            "current": cs,
            "perMinute": cs_per_min,
            "target": target,
            "goldFromCS": gold_from_cs,
            "benchmark": benchmark
        })
    }

    /// Extract jungle camp events and calculate respawn times
    pub fn extract_jungle_camp_events(events: &[Value], game_time: f64) -> Value {
        use crate::data_processing::TimerCalculator;
        
        let jungle_kills = TimerCalculator::extract_jungle_camp_events(events, game_time);
        json!(jungle_kills)
    }

    /// Extract objective timers
    pub fn extract_objective_timers(game_time: f64, events: &[Value]) -> Value {
        use crate::data_processing::TimerCalculator;
        
        let timers = TimerCalculator::calculate_objective_timers(game_time, events);
        json!(timers)
    }

    /// Extract counter item suggestions
    pub fn extract_counter_item_suggestions(all_players: &[Value]) -> Value {
        use crate::data_processing::ItemAnalyzer;
        
        // Parse enemy team
        let enemy_team: Vec<_> = all_players.iter()
            .filter(|p| p.get("team").and_then(|t| t.as_str()) == Some("CHAOS"))
            .filter_map(|p| Self::parse_player_for_analysis(p).ok())
            .collect();
        
        let suggestions = ItemAnalyzer::suggest_counter_items(&enemy_team);
        json!(suggestions)
    }

    /// Parse player data for item analysis
    fn parse_player_for_analysis(player: &Value) -> Result<crate::data_processing::item_analyzer::Player, String> {
        Ok(crate::data_processing::item_analyzer::Player {
            champion_name: player.get("championName")
                .and_then(|n| n.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            team: player.get("team")
                .and_then(|t| t.as_str())
                .unwrap_or("UNKNOWN")
                .to_string(),
            role: player.get("role")
                .and_then(|r| r.as_str())
                .unwrap_or("UNKNOWN")
                .to_string(),
            items: player.get("items")
                .and_then(|i| i.as_array())
                .map(|arr| arr.iter().filter_map(|item| item.as_u64().map(|id| id as u32)).collect())
                .unwrap_or_default(),
            level: player.get("level")
                .and_then(|l| l.as_u64())
                .unwrap_or(1) as u8,
        })
    }

    /// Calculate target CS based on game time
    fn calculate_target_cs(game_time: f64) -> u64 {
        let minutes = game_time / 60.0;
        if minutes <= 10.0 {
            (minutes * 7.0) as u64 // 7 CS/min for first 10 minutes
        } else if minutes <= 20.0 {
            (70.0 + (minutes - 10.0) * 8.0) as u64 // 8 CS/min for next 10 minutes
        } else {
            (150.0 + (minutes - 20.0) * 6.0) as u64 // 6 CS/min after 20 minutes
        }
    }

    /// Get CS benchmark based on CS per minute
    fn get_cs_benchmark(cs_per_min: f64) -> String {
        if cs_per_min >= 8.0 {
            "excellent".to_string()
        } else if cs_per_min >= 6.0 {
            "good".to_string()
        } else if cs_per_min >= 4.0 {
            "average".to_string()
        } else {
            "poor".to_string()
        }
    }
    
    /// Extract and shape game data
    fn extract_game_data(raw_data: &Value) -> Result<Option<Value>, String> {
        // In spectator mode, gameData might not exist
        let game_data = match raw_data.get("gameData") {
            Some(data) => data,
            None => {
                // Return None for spectator mode when no game data
                return Ok(None);
            }
        };
        let mut game_info = Map::new();
        
        // Game time with formatted version
        if let Some(game_time) = game_data.get("gameTime").and_then(|t| t.as_f64()) {
            game_info.insert("gameTime".to_string(), Value::Number(
                Number::from_f64(game_time).unwrap_or(Number::from(0))
            ));
            game_info.insert("formattedTime".to_string(), Value::String(
                Self::format_game_time(game_time)
            ));
        }
        
        // Game mode
        if let Some(game_mode) = game_data.get("gameMode").and_then(|m| m.as_str()) {
            game_info.insert("gameMode".to_string(), Value::String(game_mode.to_string()));
        }
        
        // Map name
        if let Some(map_name) = game_data.get("mapName").and_then(|m| m.as_str()) {
            game_info.insert("mapName".to_string(), Value::String(map_name.to_string()));
        }
        
        Ok(Some(Value::Object(game_info)))
    }
    
    /// Extract and shape active player data
    fn extract_active_player(raw_data: &Value) -> Result<Option<Value>, String> {
        // In spectator mode, activePlayer might not exist
        let active_player = match raw_data.get("activePlayer") {
            Some(player) => player,
            None => {
                // Return None for spectator mode when no active player
                return Ok(None);
            }
        };
        let mut player_info = Map::new();
        
        // Basic player info
        Self::add_string_field(active_player, "championName", &mut player_info);
        Self::add_number_field(active_player, "level", &mut player_info);
        Self::add_number_field(active_player, "currentGold", &mut player_info);
        
        // Champion stats (simplified)
        if let Some(stats) = Self::extract_champion_stats(active_player)? {
            player_info.insert("championStats".to_string(), stats);
        }
        
        // Items (simplified)
        if let Some(items) = Self::extract_items(active_player)? {
            player_info.insert("items".to_string(), items);
        }
        
        Ok(Some(Value::Object(player_info)))
    }
    
    /// Extract and shape all players data
    fn extract_all_players(raw_data: &Value) -> Result<Option<Value>, String> {
        // In spectator mode, allPlayers might not exist
        let all_players = match raw_data.get("allPlayers").and_then(|p| p.as_array()) {
            Some(players) => players,
            None => {
                // Return empty array for spectator mode
                return Ok(Some(Value::Array(vec![])));
            }
        };
        
        let simplified_players: Vec<Value> = all_players.iter()
            .filter_map(|player| Self::simplify_player(player).ok())
            .collect();
        
        Ok(Some(Value::Array(simplified_players)))
    }
    
    /// Extract and shape events
    fn extract_events(raw_data: &Value) -> Result<Option<Value>, String> {
        // In spectator mode, events might not exist
        let events = match raw_data.get("events")
            .and_then(|e| e.get("Events"))
            .and_then(|e| e.as_array()) {
            Some(events) => events,
            None => {
                // Return empty array for spectator mode
                return Ok(Some(Value::Array(vec![])));
            }
        };
        
        let recent_events: Vec<Value> = events.iter()
            .rev()
            .take(10) // Only keep last 10 events
            .filter_map(|event| Self::simplify_event(event).ok())
            .collect();
        
        Ok(Some(Value::Array(recent_events)))
    }
    
    /// Extract champion stats (simplified for UI)
    fn extract_champion_stats(active_player: &Value) -> Result<Option<Value>, String> {
        // In spectator mode, championStats might not be available
        let stats = match active_player.get("championStats") {
            Some(stats) => stats,
            None => {
                // Return empty stats for spectator mode
                return Ok(Some(Value::Object(Map::new())));
            }
        };
        
        let mut simplified_stats = Map::new();
        
        let stat_fields = ["currentHealth", "maxHealth", "attackDamage", "abilityPower"];
        for field in &stat_fields {
            Self::add_number_field(stats, field, &mut simplified_stats);
        }
        
        Ok(Some(Value::Object(simplified_stats)))
    }
    
    /// Extract items (simplified)
    fn extract_items(active_player: &Value) -> Result<Option<Value>, String> {
        // In spectator mode, items might not be available
        let items = match active_player.get("items").and_then(|i| i.as_array()) {
            Some(items) => items,
            None => {
                // Return empty items array for spectator mode
                return Ok(Some(Value::Array(vec![])));
            }
        };
        
        let simplified_items: Vec<Value> = items.iter()
            .filter_map(|item| Self::simplify_item(item).ok())
            .collect();
        
        Ok(Some(Value::Array(simplified_items)))
    }
    
    /// Simplify a player object
    fn simplify_player(player: &Value) -> Result<Value, String> {
        let mut player_info = Map::new();
        
        Self::add_string_field(player, "championName", &mut player_info);
        Self::add_string_field(player, "team", &mut player_info);
        Self::add_number_field(player, "level", &mut player_info);
        Self::add_bool_field(player, "isDead", &mut player_info);
        
        // Scores
        if let Some(scores) = player.get("scores") {
            let mut score_info = Map::new();
            let score_fields = ["kills", "deaths", "assists"];
            for field in &score_fields {
                Self::add_number_field(scores, field, &mut score_info);
            }
            player_info.insert("scores".to_string(), Value::Object(score_info));
        }
        
        Ok(Value::Object(player_info))
    }
    
    /// Simplify an event object
    fn simplify_event(event: &Value) -> Result<Value, String> {
        let mut event_info = Map::new();
        
        Self::add_string_field(event, "EventName", &mut event_info);
        Self::add_number_field(event, "EventTime", &mut event_info);
        
        Ok(Value::Object(event_info))
    }
    
    /// Simplify an item object
    fn simplify_item(item: &Value) -> Result<Value, String> {
        let mut item_info = Map::new();
        
        Self::add_string_field(item, "displayName", &mut item_info);
        Self::add_number_field(item, "itemID", &mut item_info);
        Self::add_number_field(item, "slot", &mut item_info);
        
        Ok(Value::Object(item_info))
    }
    
    /// Add string field to map if it exists
    fn add_string_field(source: &Value, field: &str, target: &mut Map<String, Value>) {
        if let Some(value) = source.get(field).and_then(|v| v.as_str()) {
            target.insert(field.to_string(), Value::String(value.to_string()));
        }
    }
    
    /// Add number field to map if it exists
    fn add_number_field(source: &Value, field: &str, target: &mut Map<String, Value>) {
        if let Some(value) = source.get(field).and_then(|v| v.as_f64()) {
            target.insert(field.to_string(), Value::Number(
                Number::from_f64(value).unwrap_or(Number::from(0))
            ));
        }
    }
    
    /// Add boolean field to map if it exists
    fn add_bool_field(source: &Value, field: &str, target: &mut Map<String, Value>) {
        if let Some(value) = source.get(field).and_then(|v| v.as_bool()) {
            target.insert(field.to_string(), Value::Bool(value));
        }
    }
    
    /// Format game time as MM:SS
    fn format_game_time(game_time: f64) -> String {
        format!("{:02}:{:02}", (game_time / 60.0) as u32, (game_time % 60.0) as u32)
    }
    
    /// Add metadata to shaped data
    fn add_metadata(shaped: &mut Map<String, Value>) {
        shaped.insert("timestamp".to_string(), Value::Number(
            Number::from_f64(Utc::now().timestamp_millis() as f64)
                .unwrap_or(Number::from(0))
        ));
        shaped.insert("dataSource".to_string(), Value::String("live_client_api".to_string()));
    }
}
