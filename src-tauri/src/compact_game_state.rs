use crate::lcu_api::LCUAPI;
use crate::types::GameSnapshot;
use serde_json::{json, Map, Value};
use std::collections::HashMap;

const DEFAULT_PATCH: &str = "15.12";
const DEFAULT_QUEUE: &str = "ranked_solo";

fn map_role(position: &str) -> &'static str {
    match position.to_uppercase().as_str() {
        "TOP" => "top",
        "JUNGLE" => "jungle",
        "MIDDLE" | "MID" => "mid",
        "BOTTOM" | "ADC" => "adc",
        "UTILITY" | "SUPPORT" => "support",
        _ => "mid",
    }
}

fn champion_name_from_id(id: u64, lookup: &HashMap<u64, String>) -> String {
    lookup
        .get(&id)
        .cloned()
        .unwrap_or_else(|| id.to_string())
}

async fn build_champion_lookup(lcu_api: &LCUAPI) -> HashMap<u64, String> {
    let mut lookup = HashMap::new();

    if let Ok(champions) = lcu_api.get_champions().await {
        if let Some(arr) = champions.as_array() {
            for champ in arr {
                if let (Some(id), Some(name)) = (
                    champ.get("id").and_then(|v| v.as_u64()),
                    champ.get("name").and_then(|v| v.as_str()),
                ) {
                    lookup.insert(id, name.to_string());
                }
            }
        } else if let Some(obj) = champions.as_object() {
            for champ in obj.values() {
                if let (Some(id), Some(name)) = (
                    champ.get("id").and_then(|v| v.as_u64()),
                    champ.get("name").and_then(|v| v.as_str()),
                ) {
                    lookup.insert(id, name.to_string());
                }
            }
        }
    }

    lookup
}

fn resolve_champion_id(value: &Value, lookup: &HashMap<u64, String>) -> Option<String> {
    if let Some(name) = value.as_str() {
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    if let Some(id) = value.as_u64() {
        return Some(champion_name_from_id(id, lookup));
    }
    if let Some(id) = value.as_i64() {
        return Some(champion_name_from_id(id as u64, lookup));
    }
    None
}

fn player_champion_id(player: &Value) -> Option<u64> {
    player
        .get("championId")
        .and_then(|v| v.as_u64())
        .or_else(|| player.get("championPickIntent").and_then(|v| v.as_u64()))
}

fn from_champion_select(
    cs: &Value,
    lookup: &HashMap<u64, String>,
) -> Option<Map<String, Value>> {
    let my_team = cs.get("myTeam")?.as_array()?;
    let local_cell = cs.get("localPlayerCellId")?.as_i64()?;

    let mut my_champ: Option<String> = None;
    let mut my_role = "mid".to_string();
    let mut allies: Vec<Value> = Vec::new();
    let mut enemies: Vec<Value> = Vec::new();
    let mut enemy_by_role: Map<String, Value> = Map::new();

    for player in my_team {
        let cell = player.get("cellId").and_then(|v| v.as_i64());
        if cell == Some(local_cell) {
            if let Some(id) = player_champion_id(player) {
                my_champ = Some(champion_name_from_id(id, lookup));
            }
            if let Some(pos) = player.get("assignedPosition").and_then(|v| v.as_str()) {
                my_role = map_role(pos).to_string();
            }
        } else if let Some(id) = player_champion_id(player) {
            allies.push(Value::String(champion_name_from_id(id, lookup)));
        }
    }

    let their_team = cs
        .get("theirTeam")
        .or_else(|| cs.get("enemyTeam"))
        .and_then(|v| v.as_array());

    if let Some(their_team) = their_team {
        for player in their_team {
            if let Some(id) = player_champion_id(player) {
                let name = champion_name_from_id(id, lookup);
                enemies.push(Value::String(name.clone()));

                if let Some(pos) = player.get("assignedPosition").and_then(|v| v.as_str()) {
                    let role_key = match map_role(pos) {
                        "top" => "enemy_top",
                        "jungle" => "enemy_jungle",
                        "mid" => "enemy_mid",
                        "adc" => "enemy_adc",
                        "support" => "enemy_support",
                        _ => continue,
                    };
                    enemy_by_role.insert(role_key.to_string(), Value::String(name));
                }
            }
        }
    }

    if my_champ.is_none() && enemies.is_empty() {
        return None;
    }

    let mut state = Map::new();
    state.insert("phase".to_string(), Value::String("champ_select".to_string()));
    state.insert(
        "my_champ".to_string(),
        Value::String(my_champ.unwrap_or_else(|| "Unknown".to_string())),
    );
    state.insert("my_role".to_string(), Value::String(my_role));
    if !allies.is_empty() {
        state.insert("allies".to_string(), Value::Array(allies));
    }
    if !enemies.is_empty() {
        state.insert("enemies".to_string(), Value::Array(enemies));
    }
    for (key, value) in enemy_by_role {
        state.insert(key, value);
    }
    state.insert("queue".to_string(), Value::String(DEFAULT_QUEUE.to_string()));
    state.insert("patch".to_string(), Value::String(DEFAULT_PATCH.to_string()));

    Some(state)
}

fn from_live_game(
    live: &Value,
    lookup: &HashMap<u64, String>,
) -> Option<Map<String, Value>> {
    let active = live.get("activePlayer")?;
    let my_champ = active
        .get("championName")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            active
                .get("championName")
                .and_then(|v| resolve_champion_id(v, lookup))
        })?;

    let my_team = active.get("team").and_then(|v| v.as_str());
    let all_players = live.get("allPlayers")?.as_array()?;

    let mut allies: Vec<Value> = Vec::new();
    let mut enemies: Vec<Value> = Vec::new();
    let mut my_role = "mid".to_string();
    let mut my_items: Vec<Value> = Vec::new();
    let mut ally_gold = 0i64;
    let mut enemy_gold = 0i64;

    let active_name = active
        .get("summonerName")
        .or_else(|| active.get("riotIdGameName"))
        .and_then(|v| v.as_str());

    for player in all_players {
        let champ = player
            .get("championName")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let team = player.get("team").and_then(|v| v.as_str());
        let is_ally = my_team.map(|t| team == Some(t)).unwrap_or(false);
        let is_me = active_name
            .map(|n| {
                player
                    .get("summonerName")
                    .and_then(|v| v.as_str())
                    .map(|sn| sn == n)
                    .unwrap_or(false)
            })
            .unwrap_or(false);

        if is_me {
            if let Some(pos) = player.get("position").and_then(|v| v.as_str()) {
                my_role = map_role(pos).to_string();
            }
            if let Some(items) = player.get("items").and_then(|v| v.as_array()) {
                for item in items {
                    if let Some(name) = item.get("displayName").and_then(|v| v.as_str()) {
                        if !name.is_empty() && name != "No Item" {
                            my_items.push(Value::String(name.to_string()));
                        }
                    }
                }
            }
        }

        if is_ally && !is_me {
            allies.push(Value::String(champ.to_string()));
        } else if !is_ally {
            enemies.push(Value::String(champ.to_string()));
        }
    }

    let game_time_sec = live
        .get("gameData")
        .and_then(|gd| gd.get("gameTime"))
        .and_then(|gt| gt.as_f64())
        .map(|t| t as i64)
        .unwrap_or(0);

    // Approximate gold diff from active player gold vs average enemy (when team gold unavailable)
    if let Some(my_gold) = active.get("currentGold").and_then(|v| v.as_i64()) {
        ally_gold = my_gold;
        let enemy_count = enemies.len().max(1) as i64;
        enemy_gold = my_gold.saturating_sub(500) * enemy_count / enemy_count;
    }
    let gold_diff = ally_gold - enemy_gold;

    let mut state = Map::new();
    state.insert("phase".to_string(), Value::String("in_game".to_string()));
    state.insert("my_champ".to_string(), Value::String(my_champ));
    state.insert("my_role".to_string(), Value::String(my_role));
    if !allies.is_empty() {
        state.insert("allies".to_string(), Value::Array(allies));
    }
    if !enemies.is_empty() {
        state.insert("enemies".to_string(), Value::Array(enemies.clone()));
        state.insert("enemy_threats".to_string(), Value::Array(enemies));
    }
    if game_time_sec > 0 {
        state.insert("game_time_sec".to_string(), json!(game_time_sec));
    }
    if gold_diff != 0 {
        state.insert("gold_diff".to_string(), json!(gold_diff));
    }
    if !my_items.is_empty() {
        state.insert("my_items".to_string(), Value::Array(my_items));
    }
    state.insert("queue".to_string(), Value::String(DEFAULT_QUEUE.to_string()));
    state.insert("patch".to_string(), Value::String(DEFAULT_PATCH.to_string()));

    Some(state)
}

fn from_post_game(snapshot: &GameSnapshot, lookup: &HashMap<u64, String>) -> Option<Map<String, Value>> {
    if let Some(cs) = snapshot.champion_select.as_ref() {
        if let Some(mut state) = from_champion_select(cs, lookup) {
            state.insert("phase".to_string(), Value::String("post_game".to_string()));
            return Some(state);
        }
    }
    if let Some(live) = snapshot.live_game.as_ref() {
        if let Some(mut state) = from_live_game(live, lookup) {
            state.insert("phase".to_string(), Value::String("post_game".to_string()));
            return Some(state);
        }
    }
    None
}

pub async fn build_compact_game_state(
    snapshot: &GameSnapshot,
    lcu_api: &LCUAPI,
) -> Option<Value> {
    let lookup = build_champion_lookup(lcu_api).await;

    let post_game_phases = [
        "EndOfGame",
        "WaitingForStats",
        "PreEndOfGame",
    ];

    if post_game_phases.contains(&snapshot.game_phase.as_str()) {
        return from_post_game(snapshot, &lookup).map(Value::Object);
    }

    if let Some(cs) = snapshot.champion_select.as_ref() {
        if let Some(state) = from_champion_select(cs, &lookup) {
            return Some(Value::Object(state));
        }
    }

    if let Some(live) = snapshot.live_game.as_ref() {
        if let Some(state) = from_live_game(live, &lookup) {
            return Some(Value::Object(state));
        }
    }

    None
}
