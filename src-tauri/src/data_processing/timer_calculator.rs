use serde_json::Value;
use std::collections::HashMap;

/// Handles timer calculations for jungle camps, objectives, and other game timers
pub struct TimerCalculator;

impl TimerCalculator {
    /// Extract jungle camp events and calculate respawn times
    pub fn extract_jungle_camp_events(events: &[Value], game_time: f64) -> HashMap<String, serde_json::Value> {
        let mut jungle_kills = HashMap::new();
        
        // Jungle camp respawn times (in seconds)
        let camp_respawn_times = [
            ("Baron", 360.0),      // 6 minutes
            ("Dragon", 300.0),     // 5 minutes
            ("Herald", 360.0),     // 6 minutes
            ("Blue", 300.0),       // 5 minutes
            ("Red", 300.0),        // 5 minutes
            ("Gromp", 150.0),      // 2.5 minutes
            ("Wolves", 120.0),     // 2 minutes
            ("Raptors", 100.0),    // 1.67 minutes
            ("Krugs", 100.0),      // 1.67 minutes
        ];
        
        // Parse events for jungle camp kills
        for event in events {
            if let Some(event_name) = event.get("EventName").and_then(|n| n.as_str()) {
                if let Some(event_time) = event.get("EventTime").and_then(|t| t.as_f64()) {
                    // Check for jungle camp kills
                    for (camp_name, respawn_time) in &camp_respawn_times {
                        if event_name.contains(camp_name) && event_name.contains("Kill") {
                            let respawn_time_seconds = event_time + respawn_time;
                            let time_until_respawn = respawn_time_seconds - game_time;
                            
                            jungle_kills.insert(camp_name.to_string(), serde_json::json!({
                                "killed_at": event_time,
                                "respawn_at": respawn_time_seconds,
                                "time_until_respawn": time_until_respawn,
                                "is_respawned": time_until_respawn <= 0.0,
                                "formatted_time": Self::format_time(time_until_respawn)
                            }));
                        }
                    }
                }
            }
        }
        
        jungle_kills
    }
    
    /// Calculate objective timers (Baron, Dragon, etc.)
    pub fn calculate_objective_timers(game_time: f64, events: &[Value]) -> HashMap<String, serde_json::Value> {
        let mut timers = HashMap::new();
        
        // Objective spawn times
        let objective_times = [
            ("Dragon", 300.0),     // 5 minutes
            ("Baron", 1200.0),      // 20 minutes
            ("Herald", 600.0),      // 10 minutes
        ];
        
        // Check for objective kills
        for event in events {
            if let Some(event_name) = event.get("EventName").and_then(|n| n.as_str()) {
                if let Some(event_time) = event.get("EventTime").and_then(|t| t.as_f64()) {
                    for (objective, spawn_time) in &objective_times {
                        if event_name.contains(objective) && event_name.contains("Kill") {
                            let next_spawn = event_time + spawn_time;
                            let time_until_spawn = next_spawn - game_time;
                            
                            timers.insert(objective.to_string(), serde_json::json!({
                                "killed_at": event_time,
                                "next_spawn_at": next_spawn,
                                "time_until_spawn": time_until_spawn,
                                "is_available": time_until_spawn <= 0.0,
                                "formatted_time": Self::format_time(time_until_spawn)
                            }));
                        }
                    }
                }
            }
        }
        
        // Add default timers for objectives that haven't been killed yet
        for (objective, spawn_time) in &objective_times {
            if !timers.contains_key(*objective) {
                let time_until_spawn = *spawn_time - game_time;
                if time_until_spawn > 0.0 {
                    timers.insert(objective.to_string(), serde_json::json!({
                        "killed_at": null,
                        "next_spawn_at": *spawn_time,
                        "time_until_spawn": time_until_spawn,
                        "is_available": time_until_spawn <= 0.0,
                        "formatted_time": Self::format_time(time_until_spawn)
                    }));
                }
            }
        }
        
        timers
    }
    
    /// Calculate ward timers
    pub fn calculate_ward_timers(events: &[Value], game_time: f64) -> HashMap<String, serde_json::Value> {
        let mut ward_timers = HashMap::new();
        
        // Ward durations (in seconds)
        let ward_durations = [
            ("Control Ward", 999999.0), // Control wards last until destroyed
            ("Stealth Ward", 150.0),    // 2.5 minutes
            ("Pink Ward", 999999.0),    // Pink wards last until destroyed
        ];
        
        for event in events {
            if let Some(event_name) = event.get("EventName").and_then(|n| n.as_str()) {
                if let Some(event_time) = event.get("EventTime").and_then(|t| t.as_f64()) {
                    for (ward_type, duration) in &ward_durations {
                        if event_name.contains(ward_type) && event_name.contains("Ward") {
                            let expires_at = event_time + duration;
                            let time_until_expiry = expires_at - game_time;
                            
                            ward_timers.insert(ward_type.to_string(), serde_json::json!({
                                "placed_at": event_time,
                                "expires_at": expires_at,
                                "time_until_expiry": time_until_expiry,
                                "is_expired": time_until_expiry <= 0.0,
                                "formatted_time": Self::format_time(time_until_expiry)
                            }));
                        }
                    }
                }
            }
        }
        
        ward_timers
    }
    
    /// Calculate summoner spell cooldowns
    pub fn calculate_summoner_cooldowns(events: &[Value], game_time: f64) -> HashMap<String, serde_json::Value> {
        let mut cooldowns = HashMap::new();
        
        // Summoner spell cooldowns (in seconds)
        let spell_cooldowns = [
            ("Flash", 300.0),      // 5 minutes
            ("Ignite", 180.0),      // 3 minutes
            ("Heal", 240.0),        // 4 minutes
            ("Barrier", 180.0),     // 3 minutes
            ("Exhaust", 210.0),     // 3.5 minutes
            ("Ghost", 180.0),       // 3 minutes
            ("Teleport", 360.0),    // 6 minutes
            ("Cleanse", 210.0),     // 3.5 minutes
        ];
        
        for event in events {
            if let Some(event_name) = event.get("EventName").and_then(|n| n.as_str()) {
                if let Some(event_time) = event.get("EventTime").and_then(|t| t.as_f64()) {
                    for (spell, cooldown) in &spell_cooldowns {
                        if event_name.contains(spell) && event_name.contains("Cast") {
                            let available_at = event_time + cooldown;
                            let time_until_available = available_at - game_time;
                            
                            cooldowns.insert(spell.to_string(), serde_json::json!({
                                "cast_at": event_time,
                                "available_at": available_at,
                                "time_until_available": time_until_available,
                                "is_available": time_until_available <= 0.0,
                                "formatted_time": Self::format_time(time_until_available)
                            }));
                        }
                    }
                }
            }
        }
        
        cooldowns
    }
    
    /// Format time in MM:SS format
    fn format_time(seconds: f64) -> String {
        if seconds <= 0.0 {
            "00:00".to_string()
        } else {
            let minutes = (seconds / 60.0) as u32;
            let secs = (seconds % 60.0) as u32;
            format!("{:02}:{:02}", minutes, secs)
        }
    }
    
    /// Get time until next important objective
    pub fn get_next_important_objective(game_time: f64, events: &[Value]) -> Option<serde_json::Value> {
        let objective_timers = Self::calculate_objective_timers(game_time, events);
        
        let mut next_objective = None;
        let mut min_time = f64::INFINITY;
        
        for (objective, timer_data) in objective_timers {
            if let Some(time_until) = timer_data.get("time_until_spawn").and_then(|t| t.as_f64()) {
                if time_until > 0.0 && time_until < min_time {
                    min_time = time_until;
                    next_objective = Some(serde_json::json!({
                        "objective": objective,
                        "time_until": time_until,
                        "formatted_time": Self::format_time(time_until)
                    }));
                }
            }
        }
        
        next_objective
    }
}