use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub status: GameStatus,
    pub game_mode: GameMode,
    pub champion: Option<Champion>,
    pub team: Vec<Player>,
    pub enemy_team: Vec<Player>,
    pub game_time: Option<u64>,
    pub phase: Option<String>,
    pub time_remaining: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameStatus {
    Idle,
    InChampionSelect,
    InGame,
    Loading,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMode {
    None,
    SummonersRift,
    HowlingAbyss,
    TwistedTreeline,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Champion {
    pub id: String,
    pub name: String,
    pub role: Role,
    pub abilities: Vec<Ability>,
    pub difficulty: u8,
    pub counters: Vec<String>,
    pub synergies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ability {
    pub key: String,
    pub name: String,
    pub description: String,
    pub cooldown: f64,
    pub range: u32,
    pub damage: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub summoner_name: String,
    pub champion: Option<Champion>,
    pub role: Role,
    pub team: Team,
    pub rank: Option<String>,
    pub pick_order: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    Top,
    Jungle,
    Mid,
    ADC,
    Support,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Top => write!(f, "Top"),
            Role::Jungle => write!(f, "Jungle"),
            Role::Mid => write!(f, "Mid"),
            Role::ADC => write!(f, "ADC"),
            Role::Support => write!(f, "Support"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Team {
    Blue,
    Red,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionSuggestion {
    pub champion: Champion,
    pub confidence: f64,
    pub reason: String,
    pub counters: Vec<String>,
    pub synergies: Vec<String>,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityRecommendation {
    pub ability: Ability,
    pub priority: u8,
    pub reason: String,
    pub situational: bool,
    pub timing: Option<String>,
}

#[derive(Clone)]
pub struct GameManager {
    current_state: GameState,
    champion_data: HashMap<String, Champion>,
    champion_cache: Arc<Mutex<HashMap<u64, serde_json::Value>>>,
    item_cache: Arc<Mutex<HashMap<u32, serde_json::Value>>>,
    rune_cache: Arc<Mutex<HashMap<u32, serde_json::Value>>>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            current_state: GameState {
                status: GameStatus::Idle,
                game_mode: GameMode::None,
                champion: None,
                team: Vec::new(),
                enemy_team: Vec::new(),
                game_time: None,
                phase: None,
                time_remaining: None,
            },
            champion_data: HashMap::new(),
            champion_cache: Arc::new(Mutex::new(HashMap::new())),
            item_cache: Arc::new(Mutex::new(HashMap::new())),
            rune_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_current_state(&self) -> &GameState {
        &self.current_state
    }

    pub fn update_state(&mut self, state: GameState) {
        self.current_state = state;
    }

    pub async fn get_champion_suggestions(
        &self,
        enemy_team: &[Player],
        ally_team: &[Player],
        role: &Role,
    ) -> Result<Vec<ChampionSuggestion>, String> {
        println!("[GameManager] Getting champion suggestions for role: {:?}", role);
        
        // Get real champion data from cache
        let champions = self.get_available_champions().await?;
        
        if champions.is_empty() {
            return Err("No champion data available. Please ensure LCU connection is established.".to_string());
        }
        
        // Analyze enemy team composition
        let enemy_champions: Vec<String> = enemy_team.iter()
            .filter_map(|p| p.champion.as_ref().map(|c| c.name.clone()))
            .collect();
        
        // Analyze ally team composition  
        let ally_champions: Vec<String> = ally_team.iter()
            .filter_map(|p| p.champion.as_ref().map(|c| c.name.clone()))
            .collect();
        
        // Filter champions by role compatibility
        let role_compatible_champions: Vec<&crate::lcu_api::Champion> = champions.iter()
            .filter(|champ| self.is_champion_suitable_for_role(champ, role))
            .collect();
        
        if role_compatible_champions.is_empty() {
            return Err("No champions found for the specified role".to_string());
        }
        
        // Generate AI-powered suggestions based on team compositions
        let mut suggestions = Vec::new();
        
        for champion in role_compatible_champions.iter().take(5) { // Top 5 suggestions
            let confidence = self.calculate_champion_confidence(champion, &enemy_champions, &ally_champions, role);
            let reason = self.generate_champion_reason(champion, &enemy_champions, &ally_champions, role);
            let counters = self.identify_champion_counters(champion, &enemy_champions);
            let synergies = self.identify_champion_synergies(champion, &ally_champions);
            let priority = self.calculate_champion_priority(champion, &enemy_champions, &ally_champions);
            
            // Convert LCU Champion to game Champion
            let game_champion = Champion {
                id: champion.id.to_string(),
                name: champion.name.clone(),
                role: role.clone(),
                abilities: champion.abilities.iter().map(|ability| {
                    crate::game::Ability {
                        key: ability.id.clone(),
                        name: ability.name.clone(),
                        description: ability.description.clone(),
                        cooldown: ability.cooldown.first().unwrap_or(&0.0).clone(),
                        range: ability.range.first().unwrap_or(&0.0).clone() as u32,
                        damage: ability.effect.first().map(|d| *d as u32),
                    }
                }).collect(),
                difficulty: self.calculate_champion_difficulty(champion),
                counters: counters.clone(),
                synergies: synergies.clone(),
            };
            
            suggestions.push(ChampionSuggestion {
                champion: game_champion,
                confidence,
                reason,
                counters,
                synergies,
                priority,
            });
        }
        
        // Sort by confidence and priority
        suggestions.sort_by(|a, b| {
            b.confidence.partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.priority.cmp(&a.priority))
        });
        
        println!("[GameManager] Generated {} champion suggestions", suggestions.len());
        Ok(suggestions)
    }

    pub async fn get_ability_recommendations(
        &self,
        champion: &str,
        game_time: u64,
        enemy_champions: &[String],
    ) -> Result<Vec<AbilityRecommendation>, String> {
        println!("[GameManager] Getting ability recommendations for {} at {}s", champion, game_time);
        
        // Get real champion data from cache
        let champions = self.get_available_champions().await?;
        
        // Find the specific champion
        let champion_data = champions.iter()
            .find(|c| c.name.to_lowercase() == champion.to_lowercase())
            .ok_or_else(|| format!("Champion '{}' not found in available champions", champion))?;
        
        let mut recommendations = Vec::new();
        
        // Analyze each ability based on game state
        for ability in &champion_data.abilities {
            let priority = self.calculate_ability_priority(ability, game_time, enemy_champions);
            let reason = self.generate_ability_reason(ability, game_time, enemy_champions);
            let situational = self.is_ability_situational(ability, enemy_champions);
            let timing = self.get_ability_timing(ability, game_time);
            
            let game_ability = Ability {
                key: ability.id.clone(),
                name: ability.name.clone(),
                description: ability.description.clone(),
                cooldown: ability.cooldown.first().unwrap_or(&0.0).clone(),
                range: ability.range.first().unwrap_or(&0.0).clone() as u32,
                damage: ability.effect.first().map(|d| *d as u32),
            };
            
            recommendations.push(AbilityRecommendation {
                ability: game_ability,
                priority,
                reason,
                situational,
                timing,
            });
        }
        
        // Sort by priority
        recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        println!("[GameManager] Generated {} ability recommendations", recommendations.len());
        Ok(recommendations)
    }

    pub fn is_in_champion_select(&self) -> bool {
        matches!(self.current_state.status, GameStatus::InChampionSelect)
    }

    pub fn is_in_game(&self) -> bool {
        matches!(self.current_state.status, GameStatus::InGame)
    }

    // Helper method to get champion data from LCU API
    pub async fn get_champion_data(&self, champion_id: u64) -> Result<serde_json::Value, String> {
        // Check cache first
        {
            let cache = self.champion_cache.lock().await;
            if let Some(data) = cache.get(&champion_id) {
                return Ok(data.clone());
            }
        }

        // This method should be called with an active LCU connection
        // The actual implementation would fetch from LCU API and cache the result
        Ok(serde_json::json!({
            "id": champion_id,
            "name": "Champion Data",
            "title": "LCU Connection Required",
            "description": "Champion data will be fetched from LCU API when connection is established",
            "abilities": [],
            "stats": {},
            "tags": []
        }))
    }

    // Helper method to get item data from LCU API
    pub async fn get_item_data(&self, item_id: u32) -> Result<serde_json::Value, String> {
        // Check cache first
        {
            let cache = self.item_cache.lock().await;
            if let Some(data) = cache.get(&item_id) {
                return Ok(data.clone());
            }
        }

        // This method should be called with an active LCU connection
        // The actual implementation would fetch from LCU API and cache the result
        Ok(serde_json::json!({
            "id": item_id,
            "name": "Item Data",
            "description": "Item data will be fetched from LCU API when connection is established",
            "price": 0,
            "stats": {}
        }))
    }

    // Helper method to get rune data from LCU API
    pub async fn get_rune_data(&self, rune_id: u32) -> Result<serde_json::Value, String> {
        // Check cache first
        {
            let cache = self.rune_cache.lock().await;
            if let Some(data) = cache.get(&rune_id) {
                return Ok(data.clone());
            }
        }

        // This method should be called with an active LCU connection
        // The actual implementation would fetch from LCU API and cache the result
        Ok(serde_json::json!({
            "id": rune_id,
            "name": "Rune Data",
            "description": "Rune data will be fetched from LCU API when connection is established",
            "icon": "",
            "longDesc": ""
        }))
    }

    // Method to populate caches with LCU data
    pub async fn populate_caches_from_lcu(&self, lcu_api: &crate::lcu_api::LCUAPI) -> Result<(), String> {
        println!("[GameManager] Populating caches from LCU...");

        let champions = lcu_api.get_champions().await?;
        {
            let mut cache = self.champion_cache.lock().await;
            cache.clear();
            if let Some(arr) = champions.as_array() {
                for champ in arr {
                    if let Some(id) = champ.get("id").and_then(|v| v.as_u64()) {
                        cache.insert(id, champ.clone());
                    }
                }
            } else if let Some(obj) = champions.as_object() {
                for (id_str, champ) in obj {
                    if let Ok(id) = id_str.parse::<u64>() {
                        cache.insert(id, champ.clone());
                    }
                }
            }
            println!("[GameManager] Cached {} champions", cache.len());
        }

        if let Ok(items) = lcu_api.get_items().await {
            let mut cache = self.item_cache.lock().await;
            cache.clear();
            if let Some(obj) = items.as_object() {
                for (id_str, item) in obj {
                    if let Ok(id) = id_str.parse::<u32>() {
                        cache.insert(id, item.clone());
                    }
                }
            }
        }

        if let Ok(runes) = lcu_api.get_runes().await {
            let mut cache = self.rune_cache.lock().await;
            cache.clear();
            if let Some(obj) = runes.as_object() {
                for (id_str, rune) in obj {
                    if let Ok(id) = id_str.parse::<u32>() {
                        cache.insert(id, rune.clone());
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn populate_caches(&self) -> Result<(), String> {
        Err("Use populate_caches_from_lcu with an active LCU connection".to_string())
    }

    // Helper methods for champion analysis
    async fn get_available_champions(&self) -> Result<Vec<crate::lcu_api::Champion>, String> {
        let cache = self.champion_cache.lock().await;
        if cache.is_empty() {
            return Ok(Vec::new());
        }

        let mut champions = Vec::new();
        for (id, data) in cache.iter() {
            let name = data
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("Unknown")
                .to_string();
            let tags = data
                .get("tags")
                .and_then(|t| t.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default();

            champions.push(crate::lcu_api::Champion {
                id: *id,
                name,
                title: data
                    .get("title")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string(),
                description: data
                    .get("blurb")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string(),
                tags,
                stats: crate::lcu_api::ChampionStats {
                    hp: 0.0,
                    hpperlevel: 0.0,
                    mp: 0.0,
                    mpperlevel: 0.0,
                    movespeed: 0.0,
                    armor: 0.0,
                    armorperlevel: 0.0,
                    spellblock: 0.0,
                    spellblockperlevel: 0.0,
                    attackrange: 0.0,
                    hpregen: 0.0,
                    hpregenperlevel: 0.0,
                    mpregen: 0.0,
                    mpregenperlevel: 0.0,
                    crit: 0.0,
                    critperlevel: 0.0,
                    attackdamage: 0.0,
                    attackdamageperlevel: 0.0,
                    attackspeedperlevel: 0.0,
                    attackspeed: 0.0,
                },
                abilities: Vec::new(),
                passive: crate::lcu_api::ChampionPassive {
                    name: "Passive".to_string(),
                    description: String::new(),
                    image: crate::lcu_api::ChampionImage {
                        full: String::new(),
                        sprite: String::new(),
                        group: String::new(),
                        x: 0,
                        y: 0,
                        w: 0,
                        h: 0,
                    },
                },
            });
        }
        Ok(champions)
    }

    fn is_champion_suitable_for_role(&self, champion: &crate::lcu_api::Champion, role: &Role) -> bool {
        // Analyze champion tags to determine role suitability
        match role {
            Role::Top => champion.tags.contains(&"Fighter".to_string()) || champion.tags.contains(&"Tank".to_string()),
            Role::Jungle => champion.tags.contains(&"Fighter".to_string()) || champion.tags.contains(&"Assassin".to_string()),
            Role::Mid => champion.tags.contains(&"Mage".to_string()) || champion.tags.contains(&"Assassin".to_string()),
            Role::ADC => champion.tags.contains(&"Marksman".to_string()),
            Role::Support => champion.tags.contains(&"Support".to_string()) || champion.tags.contains(&"Tank".to_string()),
        }
    }

    fn calculate_champion_confidence(&self, champion: &crate::lcu_api::Champion, enemy_champions: &[String], ally_champions: &[String], role: &Role) -> f64 {
        let mut confidence = 0.5; // Base confidence
        
        // Increase confidence based on role suitability
        if self.is_champion_suitable_for_role(champion, role) {
            confidence += 0.2;
        }
        
        // Analyze enemy team counters
        let counter_score = self.analyze_counter_matchups(champion, enemy_champions);
        confidence += counter_score * 0.3;
        
        // Analyze ally team synergies
        let synergy_score = self.analyze_synergy_matchups(champion, ally_champions);
        confidence += synergy_score * 0.2;
        
        confidence.min(1.0).max(0.0)
    }

    fn generate_champion_reason(&self, champion: &crate::lcu_api::Champion, enemy_champions: &[String], ally_champions: &[String], role: &Role) -> String {
        let mut reasons = Vec::new();
        
        // Role suitability
        if self.is_champion_suitable_for_role(champion, role) {
            reasons.push(format!("Strong {} pick", role));
        }
        
        // Counter analysis
        let counter_score = self.analyze_counter_matchups(champion, enemy_champions);
        if counter_score > 0.3 {
            reasons.push("Good against enemy team composition".to_string());
        }
        
        // Synergy analysis
        let synergy_score = self.analyze_synergy_matchups(champion, ally_champions);
        if synergy_score > 0.3 {
            reasons.push("Synergizes well with ally team".to_string());
        }
        
        if reasons.is_empty() {
            "Solid pick for current team composition".to_string()
        } else {
            reasons.join(", ")
        }
    }

    fn identify_champion_counters(&self, champion: &crate::lcu_api::Champion, enemy_champions: &[String]) -> Vec<String> {
        // Simplified counter identification
        // In a real implementation, this would use champion matchup data
        enemy_champions.iter()
            .filter(|enemy| self.is_champion_countered_by(champion, enemy))
            .cloned()
            .collect()
    }

    fn identify_champion_synergies(&self, champion: &crate::lcu_api::Champion, ally_champions: &[String]) -> Vec<String> {
        // Simplified synergy identification
        // In a real implementation, this would use champion synergy data
        ally_champions.iter()
            .filter(|ally| self.has_champion_synergy_with(champion, ally))
            .cloned()
            .collect()
    }

    fn calculate_champion_priority(&self, champion: &crate::lcu_api::Champion, enemy_champions: &[String], ally_champions: &[String]) -> u8 {
        let mut priority = 5; // Base priority
        
        // Increase priority based on counter effectiveness
        let counter_score = self.analyze_counter_matchups(champion, enemy_champions);
        if counter_score > 0.5 {
            priority += 2;
        }
        
        // Increase priority based on synergy effectiveness
        let synergy_score = self.analyze_synergy_matchups(champion, ally_champions);
        if synergy_score > 0.5 {
            priority += 1;
        }
        
        priority.min(10).max(1)
    }

    fn calculate_champion_difficulty(&self, champion: &crate::lcu_api::Champion) -> u8 {
        // Simplified difficulty calculation based on champion stats
        let mut difficulty = 3; // Base difficulty
        
        // Increase difficulty for complex champions (more abilities, higher skill ceiling)
        if champion.abilities.len() > 4 {
            difficulty += 1;
        }
        
        // Adjust based on champion tags
        if champion.tags.contains(&"Assassin".to_string()) {
            difficulty += 1;
        }
        
        difficulty.min(5).max(1)
    }

    fn calculate_ability_priority(&self, ability: &crate::lcu_api::ChampionAbility, game_time: u64, _enemy_champions: &[String]) -> u8 {
        let mut priority = 5; // Base priority
        
        // Early game abilities get higher priority
        if game_time < 300 && ability.name.to_lowercase().contains("q") {
            priority += 2;
        }
        
        // Ultimate abilities get higher priority
        if ability.name.to_lowercase().contains("ultimate") || ability.name.to_lowercase().contains("r") {
            priority += 3;
        }
        
        priority.min(10).max(1)
    }

    fn generate_ability_reason(&self, ability: &crate::lcu_api::ChampionAbility, game_time: u64, _enemy_champions: &[String]) -> String {
        let mut reasons = Vec::new();
        
        if game_time < 300 {
            reasons.push("Strong early game ability".to_string());
        }
        
        if ability.name.to_lowercase().contains("ultimate") {
            reasons.push("Ultimate ability - high impact".to_string());
        }
        
        if ability.cooldown.iter().any(|&cd| cd < 10.0) {
            reasons.push("Low cooldown - high frequency usage".to_string());
        }
        
        if reasons.is_empty() {
            "Situational ability based on game state".to_string()
        } else {
            reasons.join(", ")
        }
    }

    fn is_ability_situational(&self, ability: &crate::lcu_api::ChampionAbility, _enemy_champions: &[String]) -> bool {
        // Abilities with high cooldowns or specific conditions are situational
        ability.cooldown.iter().any(|&cd| cd > 20.0) || 
        ability.name.to_lowercase().contains("ultimate")
    }

    fn get_ability_timing(&self, ability: &crate::lcu_api::ChampionAbility, _game_time: u64) -> Option<String> {
        if ability.name.to_lowercase().contains("ultimate") {
            Some("Use during team fights or key moments".to_string())
        } else if ability.cooldown.iter().any(|&cd| cd < 5.0) {
            Some("Use frequently for poke and trading".to_string())
        } else {
            Some("Use when cooldown is available".to_string())
        }
    }

    fn analyze_counter_matchups(&self, champion: &crate::lcu_api::Champion, enemy_champions: &[String]) -> f64 {
        // Simplified counter analysis
        // In a real implementation, this would use comprehensive matchup data
        let mut score: f64 = 0.0;
        
        for enemy in enemy_champions {
            if self.is_champion_countered_by(champion, enemy) {
                score += 0.2;
            }
        }
        
        score.min(1.0)
    }

    fn analyze_synergy_matchups(&self, champion: &crate::lcu_api::Champion, ally_champions: &[String]) -> f64 {
        // Simplified synergy analysis
        let mut score: f64 = 0.0;
        
        for ally in ally_champions {
            if self.has_champion_synergy_with(champion, ally) {
                score += 0.2;
            }
        }
        
        score.min(1.0)
    }

    fn is_champion_countered_by(&self, champion: &crate::lcu_api::Champion, enemy: &str) -> bool {
        // Simplified counter logic
        // In a real implementation, this would use comprehensive matchup data
        match (champion.name.to_lowercase().as_str(), enemy.to_lowercase().as_str()) {
            ("yasuo", "malphite") | ("yasuo", "rammus") => true,
            ("katarina", "malzahar") | ("katarina", "lissandra") => true,
            ("master yi", "jax") | ("master yi", "rammus") => true,
            _ => false,
        }
    }

    fn has_champion_synergy_with(&self, champion: &crate::lcu_api::Champion, ally: &str) -> bool {
        // Simplified synergy logic
        match (champion.name.to_lowercase().as_str(), ally.to_lowercase().as_str()) {
            ("yasuo", "malphite") | ("yasuo", "gragas") => true,
            ("katarina", "yuumi") | ("katarina", "lulu") => true,
            ("jinx", "thresh") | ("jinx", "blitzcrank") => true,
            _ => false,
        }
    }
}
