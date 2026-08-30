use crate::lcu_api::LCUAPI;
use crate::types::GameSnapshot;
use serde_json::{json, Map, Value};
use std::collections::HashMap;

/// Fallbacks only — the live values come from the client (see
/// `LCUAPI::get_patch_version` / `get_current_queue`). A hardcoded patch
/// silently feeds the model stale meta on every request.
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

/// Identity fields the Live Client API uses for a player, most specific first.
const PLAYER_NAME_KEYS: [&str; 3] = ["riotId", "summonerName", "riotIdGameName"];

fn from_live_game(
    live: &Value,
    lookup: &HashMap<u64, String>,
) -> Option<Map<String, Value>> {
    let active = live.get("activePlayer")?;
    let all_players = live.get("allPlayers")?.as_array()?;

    // `activePlayer` carries no `championName` and no `team` — those exist only
    // on the per-player entries in `allPlayers`, keyed by Riot ID. Reading them
    // off `activePlayer` yields None for every live game.
    let active_name = active
        .get("riotId")
        .or_else(|| active.get("summonerName"))
        .or_else(|| active.get("riotIdGameName"))
        .and_then(|v| v.as_str());

    let me = active_name.and_then(|name| {
        all_players.iter().find(|player| {
            PLAYER_NAME_KEYS
                .iter()
                .any(|key| player.get(*key).and_then(|v| v.as_str()) == Some(name))
        })
    })?;

    let my_champ = me
        .get("championName")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            me.get("championId")
                .and_then(|v| resolve_champion_id(v, lookup))
        })?;

    let my_team = me.get("team").and_then(|v| v.as_str());

    let mut allies: Vec<Value> = Vec::new();
    let mut enemies: Vec<Value> = Vec::new();
    let mut my_role = "mid".to_string();
    let mut my_items: Vec<Value> = Vec::new();

    for player in all_players {
        let champ = player
            .get("championName")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        let team = player.get("team").and_then(|v| v.as_str());
        let is_ally = my_team.map(|t| team == Some(t)).unwrap_or(false);
        let is_me = active_name
            .map(|n| {
                PLAYER_NAME_KEYS
                    .iter()
                    .any(|key| player.get(*key).and_then(|v| v.as_str()) == Some(n))
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

    // No gold_diff: the Live Client payload exposes only the active player's
    // unspent gold, which cannot express a team gold lead. The field is
    // optional in game_state.schema.json — omitting it beats guessing.

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
    if !my_items.is_empty() {
        state.insert("my_items".to_string(), Value::Array(my_items));
    }
    state.insert("queue".to_string(), Value::String(DEFAULT_QUEUE.to_string()));
    state.insert("patch".to_string(), Value::String(DEFAULT_PATCH.to_string()));

    Some(state)
}

async fn attach_scout_intel(mut state: Map<String, Value>) -> Map<String, Value> {
    if let Some(intel) = crate::scout_cache::get_session_enemy_intel().await {
        if intel.as_array().is_some_and(|a| !a.is_empty()) {
            state.insert("enemy_intel".to_string(), intel);
        }
    }
    state
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

    let mut state = if post_game_phases.contains(&snapshot.game_phase.as_str()) {
        match from_post_game(snapshot, &lookup) {
            Some(state) => state,
            None => return None,
        }
    } else {
        let from_cs = snapshot
            .champion_select
            .as_ref()
            .and_then(|cs| from_champion_select(cs, &lookup));
        let from_live = || {
            snapshot
                .live_game
                .as_ref()
                .and_then(|live| from_live_game(live, &lookup))
        };
        match from_cs.or_else(from_live) {
            Some(state) => state,
            None => return None,
        }
    };

    apply_live_context(&mut state, lcu_api).await;

    Some(Value::Object(attach_scout_intel(state).await))
}

/// Replaces the compiled-in patch and queue with what the client reports.
async fn apply_live_context(state: &mut Map<String, Value>, lcu_api: &LCUAPI) {
    if let Some(patch) = lcu_api.get_patch_version().await {
        state.insert("patch".to_string(), Value::String(patch));
    }
    if let Some((id, slug)) = lcu_api.get_current_queue().await {
        if slug != "unknown" {
            state.insert("queue".to_string(), Value::String(slug));
        }
        state.insert("queue_id".to_string(), json!(id));
    }
}
