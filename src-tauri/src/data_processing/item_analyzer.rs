use std::collections::HashMap;

/// Represents a player for item analysis
#[derive(Debug, Clone)]
pub struct Player {
    pub champion_name: String,
    pub team: String,
    pub role: String,
    pub items: Vec<u32>,
    pub level: u8,
}

/// Handles item analysis and counter item suggestions
pub struct ItemAnalyzer;

impl ItemAnalyzer {
    /// Suggest counter items based on enemy team composition
    pub fn suggest_counter_items(enemy_team: &[Player]) -> HashMap<String, Vec<serde_json::Value>> {
        let mut suggestions = HashMap::new();
        
        // Analyze enemy team composition
        let enemy_comp = Self::analyze_team_composition(enemy_team);
        
        // Generate counter suggestions for each enemy
        for enemy in enemy_team {
            let counter_items = Self::get_counter_items_for_champion(&enemy.champion_name, &enemy_comp);
            suggestions.insert(enemy.champion_name.clone(), counter_items);
        }
        
        // Add team-wide counter suggestions
        suggestions.insert("team_counters".to_string(), Self::get_team_counter_items(&enemy_comp));
        
        suggestions
    }
    
    /// Analyze team composition
    fn analyze_team_composition(team: &[Player]) -> TeamComposition {
        let mut comp = TeamComposition::new();
        
        for player in team {
            comp.add_champion(&player.champion_name, &player.role);
        }
        
        comp
    }
    
    /// Get counter items for a specific champion
    fn get_counter_items_for_champion(champion: &str, team_comp: &TeamComposition) -> Vec<serde_json::Value> {
        let mut counters = Vec::new();
        
        // Champion-specific counter items
        let champion_counters = Self::get_champion_specific_counters(champion);
        counters.extend(champion_counters);
        
        // Role-based counter items
        let role_counters = Self::get_role_based_counters(champion, team_comp);
        counters.extend(role_counters);
        
        // Team composition counters
        let team_counters = Self::get_team_comp_counters(team_comp);
        counters.extend(team_counters);
        
        counters
    }
    
    /// Get champion-specific counter items
    fn get_champion_specific_counters(champion: &str) -> Vec<serde_json::Value> {
        let mut counters = Vec::new();
        
        match champion.to_lowercase().as_str() {
            "yasuo" | "yone" => {
                counters.push(serde_json::json!({
                    "item": "Randuin's Omen",
                    "reason": "Reduces critical strike damage",
                    "priority": "high"
                }));
                counters.push(serde_json::json!({
                    "item": "Thornmail",
                    "reason": "Reflects damage and reduces healing",
                    "priority": "medium"
                }));
            },
            "fiora" | "camille" => {
                counters.push(serde_json::json!({
                    "item": "Bramble Vest",
                    "reason": "Reduces healing and reflects damage",
                    "priority": "high"
                }));
                counters.push(serde_json::json!({
                    "item": "Ninja Tabi",
                    "reason": "Reduces basic attack damage",
                    "priority": "medium"
                }));
            },
            "katarina" | "akali" => {
                counters.push(serde_json::json!({
                    "item": "Spirit Visage",
                    "reason": "Magic resist and healing",
                    "priority": "high"
                }));
                counters.push(serde_json::json!({
                    "item": "Banshee's Veil",
                    "reason": "Magic resist and spell shield",
                    "priority": "medium"
                }));
            },
            "zed" | "talon" => {
                counters.push(serde_json::json!({
                    "item": "Zhonya's Hourglass",
                    "reason": "Invulnerability against burst",
                    "priority": "high"
                }));
                counters.push(serde_json::json!({
                    "item": "Seeker's Armguard",
                    "reason": "Armor and AP for early game",
                    "priority": "medium"
                }));
            },
            _ => {
                // Generic counters for unknown champions
                counters.push(serde_json::json!({
                    "item": "Adaptive Helm",
                    "reason": "Reduces repeated magic damage",
                    "priority": "low"
                }));
            }
        }
        
        counters
    }
    
    /// Get role-based counter items
    fn get_role_based_counters(champion: &str, _team_comp: &TeamComposition) -> Vec<serde_json::Value> {
        let mut counters = Vec::new();
        
        // Determine champion role (simplified)
        let role = Self::determine_champion_role(champion);
        
        match role.as_str() {
            "tank" => {
                counters.push(serde_json::json!({
                    "item": "Lord Dominik's Regards",
                    "reason": "Armor penetration against tanks",
                    "priority": "high"
                }));
                counters.push(serde_json::json!({
                    "item": "Void Staff",
                    "reason": "Magic penetration against tanks",
                    "priority": "high"
                }));
            },
            "assassin" => {
                counters.push(serde_json::json!({
                    "item": "Guardian Angel",
                    "reason": "Revival against assassins",
                    "priority": "high"
                }));
                counters.push(serde_json::json!({
                    "item": "Stopwatch",
                    "reason": "Temporary invulnerability",
                    "priority": "medium"
                }));
            },
            "mage" => {
                counters.push(serde_json::json!({
                    "item": "Mercury's Treads",
                    "reason": "Magic resist and tenacity",
                    "priority": "medium"
                }));
                counters.push(serde_json::json!({
                    "item": "Maw of Malmortius",
                    "reason": "Magic resist and shield",
                    "priority": "medium"
                }));
            },
            _ => {}
        }
        
        counters
    }
    
    /// Get team composition counter items
    fn get_team_comp_counters(team_comp: &TeamComposition) -> Vec<serde_json::Value> {
        let mut counters = Vec::new();
        
        // High AD team
        if team_comp.ad_champions >= 3 {
            counters.push(serde_json::json!({
                "item": "Thornmail",
                "reason": "Reflects damage against AD-heavy teams",
                "priority": "high"
            }));
            counters.push(serde_json::json!({
                "item": "Randuin's Omen",
                "reason": "Reduces critical strike damage",
                "priority": "high"
            }));
        }
        
        // High AP team
        if team_comp.ap_champions >= 3 {
            counters.push(serde_json::json!({
                "item": "Spirit Visage",
                "reason": "Magic resist and healing",
                "priority": "high"
            }));
            counters.push(serde_json::json!({
                "item": "Adaptive Helm",
                "reason": "Reduces repeated magic damage",
                "priority": "medium"
            }));
        }
        
        // High mobility team
        if team_comp.mobile_champions >= 3 {
            counters.push(serde_json::json!({
                "item": "Frozen Heart",
                "reason": "Reduces attack speed",
                "priority": "medium"
            }));
        }
        
        counters
    }
    
    /// Get team-wide counter items
    fn get_team_counter_items(team_comp: &TeamComposition) -> Vec<serde_json::Value> {
        let mut counters = Vec::new();
        
        // Team-wide items based on composition
        if team_comp.ad_champions >= 4 {
            counters.push(serde_json::json!({
                "item": "Thornmail",
                "reason": "Essential against AD-heavy teams",
                "priority": "critical"
            }));
        }
        
        if team_comp.ap_champions >= 4 {
            counters.push(serde_json::json!({
                "item": "Spirit Visage",
                "reason": "Essential against AP-heavy teams",
                "priority": "critical"
            }));
        }
        
        if team_comp.assassin_champions >= 2 {
            counters.push(serde_json::json!({
                "item": "Guardian Angel",
                "reason": "Protection against multiple assassins",
                "priority": "high"
            }));
        }
        
        counters
    }
    
    /// Determine champion role (simplified)
    fn determine_champion_role(champion: &str) -> String {
        match champion.to_lowercase().as_str() {
            "malphite" | "maokai" | "ornn" | "sion" | "cho'gath" => "tank".to_string(),
            "zed" | "talon" | "kha'zix" | "rengar" | "akali" => "assassin".to_string(),
            "orianna" | "syndra" | "azir" | "cassiopeia" | "ryze" => "mage".to_string(),
            "yasuo" | "yone" | "fiora" | "camille" | "riven" => "fighter".to_string(),
            "jinx" | "caitlyn" | "tristana" | "vayne" | "kog'maw" => "marksman".to_string(),
            "thresh" | "blitzcrank" | "leona" | "nautilus" | "braum" => "support".to_string(),
            _ => "unknown".to_string(),
        }
    }
}

/// Represents team composition analysis
#[derive(Debug, Clone)]
struct TeamComposition {
    pub ad_champions: u8,
    pub ap_champions: u8,
    pub tank_champions: u8,
    pub assassin_champions: u8,
    pub mobile_champions: u8,
}

impl TeamComposition {
    fn new() -> Self {
        Self {
            ad_champions: 0,
            ap_champions: 0,
            tank_champions: 0,
            assassin_champions: 0,
            mobile_champions: 0,
        }
    }
    
    fn add_champion(&mut self, champion: &str, role: &str) {
        let role_lower = role.to_lowercase();
        
        match role_lower.as_str() {
            "top" | "jungle" | "mid" | "adc" | "support" => {
                // Determine if champion is AD or AP based on common patterns
                if Self::is_ad_champion(champion) {
                    self.ad_champions += 1;
                } else if Self::is_ap_champion(champion) {
                    self.ap_champions += 1;
                }
                
                // Determine other characteristics
                if Self::is_tank_champion(champion) {
                    self.tank_champions += 1;
                }
                if Self::is_assassin_champion(champion) {
                    self.assassin_champions += 1;
                }
                if Self::is_mobile_champion(champion) {
                    self.mobile_champions += 1;
                }
            }
            _ => {}
        }
    }
    
    fn is_ad_champion(champion: &str) -> bool {
        match champion.to_lowercase().as_str() {
            "yasuo" | "yone" | "fiora" | "camille" | "riven" | "darius" | "garen" | "renekton" => true,
            "jinx" | "caitlyn" | "tristana" | "vayne" | "kog'maw" | "ashe" | "sivir" => true,
            "zed" | "talon" | "kha'zix" | "rengar" | "graves" | "kindred" => true,
            _ => false,
        }
    }
    
    fn is_ap_champion(champion: &str) -> bool {
        match champion.to_lowercase().as_str() {
            "orianna" | "syndra" | "azir" | "cassiopeia" | "ryze" | "anivia" | "brand" => true,
            "katarina" | "akali" | "leblanc" | "fizz" | "ekko" | "diana" => true,
            "thresh" | "blitzcrank" | "leona" | "nautilus" | "braum" | "lulu" => true,
            _ => false,
        }
    }
    
    fn is_tank_champion(champion: &str) -> bool {
        match champion.to_lowercase().as_str() {
            "malphite" | "maokai" | "ornn" | "sion" | "cho'gath" | "nautilus" | "leona" => true,
            _ => false,
        }
    }
    
    fn is_assassin_champion(champion: &str) -> bool {
        match champion.to_lowercase().as_str() {
            "zed" | "talon" | "kha'zix" | "rengar" | "akali" | "katarina" | "leblanc" => true,
            _ => false,
        }
    }
    
    fn is_mobile_champion(champion: &str) -> bool {
        match champion.to_lowercase().as_str() {
            "yasuo" | "yone" | "riven" | "fiora" | "camille" | "zed" | "akali" => true,
            "katarina" | "leblanc" | "fizz" | "ekko" | "diana" | "kha'zix" => true,
            _ => false,
        }
    }
}