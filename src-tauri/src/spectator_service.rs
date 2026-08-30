use serde_json::{json, Value};

use crate::config::{resolve_riot_config, RiotApiConfig};
use crate::lcu_api::LCUAPI;
use crate::riot_api::RiotApiClient;

static SUMMONER_SPELL_NAMES: &[(u64, &str)] = &[
    (1, "Summoner Boost"),
    (3, "Summoner Exhaust"),
    (4, "Summoner Flash"),
    (6, "Summoner Haste"),
    (7, "Summoner Heal"),
    (11, "Summoner Smite"),
    (12, "Summoner Teleport"),
    (13, "Summoner Mana"),
    (14, "Summoner Ignite"),
    (21, "Summoner Barrier"),
    (32, "Summoner Snowball"),
];

/// Fetch active spectator game via Riot API and shape it like Live Client `allPlayers`.
pub async fn fetch_spectator_game(puuid: &str, lcu_api: &LCUAPI) -> Option<Value> {
    let config = current_riot_config();
    let client = RiotApiClient::new(config);
    if !client.is_configured() {
        return None;
    }

    let raw = match client.get_active_spectator_game(puuid).await {
        Ok(v) => v,
        Err(e) => {
            if e.contains("404") {
                return None;
            }
            eprintln!("[Spectator] API error: {}", e);
            return None;
        }
    };

    let lookup = build_champion_lookup(lcu_api).await;
    Some(transform_spectator_game(&raw, &lookup))
}

fn transform_spectator_game(raw: &Value, lookup: &std::collections::HashMap<u64, String>) -> Value {
    let participants = raw
        .get("participants")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let all_players: Vec<Value> = participants
        .iter()
        .filter_map(|p| participant_to_player(p, lookup))
        .collect();

    json!({
        "source": "spectator-v5",
        "gameId": raw.get("gameId"),
        "gameMode": raw.get("gameMode"),
        "gameType": raw.get("gameType"),
        "mapId": raw.get("mapId"),
        "gameQueueConfigId": raw.get("gameQueueConfigId"),
        "gameLength": raw.get("gameLength"),
        "platformId": raw.get("platformId"),
        "allPlayers": all_players,
        "gameData": {
            "gameTime": raw.get("gameLength").and_then(|v| v.as_f64()).unwrap_or(0.0),
            "gameMode": raw.get("gameMode").and_then(|v| v.as_str()).unwrap_or(""),
        },
    })
}

fn participant_to_player(
    p: &Value,
    lookup: &std::collections::HashMap<u64, String>,
) -> Option<Value> {
    let champion_id = p.get("championId").and_then(|v| v.as_u64())?;
    let champion_name = lookup
        .get(&champion_id)
        .cloned()
        .unwrap_or_else(|| champion_id.to_string());

    let team_id = p.get("teamId").and_then(|v| v.as_u64()).unwrap_or(100);
    let team = if team_id == 100 { "ORDER" } else { "CHAOS" };

    let (game_name, tag_line, riot_id) = parse_riot_id(p);

    let spell1 = p.get("spell1Id").and_then(|v| v.as_u64());
    let spell2 = p.get("spell2Id").and_then(|v| v.as_u64());

    Some(json!({
        "championName": champion_name,
        "championId": champion_id,
        "team": team,
        "isBot": p.get("bot").and_then(|v| v.as_bool()).unwrap_or(false),
        "riotId": riot_id,
        "riotIdGameName": game_name,
        "riotIdTagLine": tag_line,
        "summonerName": riot_id.clone().unwrap_or_else(|| "Unknown".to_string()),
        "puuid": p.get("puuid").and_then(|v| v.as_str()),
        "position": "",
        "level": 1,
        "scores": { "kills": 0, "deaths": 0, "assists": 0, "creepScore": 0, "wardScore": 0 },
        "items": [],
        "summonerSpells": {
            "summonerSpellOne": spell_json(spell1),
            "summonerSpellTwo": spell_json(spell2),
        },
        "runes": perks_to_runes(p.get("perks")),
    }))
}

fn parse_riot_id(p: &Value) -> (Option<String>, Option<String>, Option<String>) {
    if let Some(id) = p.get("riotId").and_then(|v| v.as_str()) {
        if let Some((name, tag)) = id.split_once('#') {
            return (
                Some(name.to_string()),
                Some(tag.to_string()),
                Some(id.to_string()),
            );
        }
    }
    (None, None, None)
}

fn spell_json(spell_id: Option<u64>) -> Value {
    let name = spell_id
        .and_then(|id| {
            SUMMONER_SPELL_NAMES
                .iter()
                .find(|(sid, _)| *sid == id)
                .map(|(_, n)| *n)
        })
        .unwrap_or("Unknown Spell");
    json!({
        "displayName": name,
        "rawDisplayName": name,
    })
}

fn perks_to_runes(perks: Option<&Value>) -> Value {
    let perks = match perks {
        Some(v) => v,
        None => return Value::Null,
    };
    let style = perks
        .get("perkStyle")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let sub = perks
        .get("perkSubStyle")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let ids = perks
        .get("perkIds")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let keystone_id = ids.first().and_then(|v| v.as_u64()).unwrap_or(0);

    json!({
        "keystone": {
            "id": keystone_id,
            "displayName": format!("Perk {}", keystone_id),
        },
        "primaryRuneTree": { "id": style, "displayName": format!("Style {}", style) },
        "secondaryRuneTree": { "id": sub, "displayName": format!("Style {}", sub) },
    })
}

async fn build_champion_lookup(lcu_api: &LCUAPI) -> std::collections::HashMap<u64, String> {
    let mut lookup = std::collections::HashMap::new();
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

fn current_riot_config() -> RiotApiConfig {
    crate::get_config_manager()
        .lock()
        .map(|m| resolve_riot_config(&m.get_config().riot_api))
        .unwrap_or_else(|_| resolve_riot_config(&RiotApiConfig::default()))
}
