use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

pub struct GameManager {
    current_state: GameState,
    champion_data: HashMap<String, Champion>,
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
        }
    }

    pub fn get_current_state(&self) -> &GameState {
        &self.current_state
    }

    pub fn update_state(&mut self, state: GameState) {
        self.current_state = state;
    }

    pub fn get_champion_suggestions(
        &self,
        _enemy_team: &[Player],
        _ally_team: &[Player],
        _role: &Role,
    ) -> Vec<ChampionSuggestion> {
        // TODO: Implement AI-powered champion suggestions
        // For now, return mock data
        vec![
            ChampionSuggestion {
                champion: Champion {
                    id: "103".to_string(),
                    name: "Ahri".to_string(),
                    role: Role::Mid,
                    abilities: vec![],
                    difficulty: 3,
                    counters: vec!["Zed".to_string(), "Fizz".to_string()],
                    synergies: vec!["Jarvan".to_string(), "Sejuani".to_string()],
                },
                confidence: 0.85,
                reason: "Strong against current enemy composition".to_string(),
                counters: vec!["Zed".to_string(), "Fizz".to_string()],
                synergies: vec!["Jarvan".to_string(), "Sejuani".to_string()],
                priority: 1,
            }
        ]
    }

    pub fn get_ability_recommendations(
        &self,
        _champion: &str,
        _game_time: u64,
        _enemy_champions: &[String],
    ) -> Vec<AbilityRecommendation> {
        // TODO: Implement AI-powered ability recommendations
        // For now, return mock data
        vec![
            AbilityRecommendation {
                ability: Ability {
                    key: "Q".to_string(),
                    name: "Orb of Deception".to_string(),
                    description: "Send out and pull back her orb".to_string(),
                    cooldown: 7.0,
                    range: 550,
                    damage: Some(40),
                },
                priority: 1,
                reason: "Primary damage ability, max first".to_string(),
                situational: false,
                timing: None,
            }
        ]
    }

    pub fn is_in_champion_select(&self) -> bool {
        matches!(self.current_state.status, GameStatus::InChampionSelect)
    }

    pub fn is_in_game(&self) -> bool {
        matches!(self.current_state.status, GameStatus::InGame)
    }
}
