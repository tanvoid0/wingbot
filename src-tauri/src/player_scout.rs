use serde::Deserialize;
use serde_json::{json, Value};

use crate::config::{resolve_riot_config, RiotApiConfig};
use crate::riot_api::RiotApiClient;
use crate::riot_api_cache;
use crate::scout_cache;

const CHAMP_FORM_SCAN: u32 = 15;
const CHAMP_FORM_TARGET: u32 = 5;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoutPlayerInput {
    pub puuid: Option<String>,
    pub game_name: Option<String>,
    pub tag_line: Option<String>,
    pub champion_id: Option<u64>,
    pub champion_name: Option<String>,
    pub position: Option<String>,
    /// When true, fetch recent match-v5 form on the locked champion.
    #[serde(default)]
    pub include_champion_form: bool,
}

pub async fn scout_players(players: Vec<ScoutPlayerInput>) -> Result<Value, String> {
    let config = current_riot_config();
    let client = RiotApiClient::new(config.clone());

    if !client.is_configured() {
        return Err(
            "Riot API key required for enemy scouting — add your key in Settings".to_string(),
        );
    }

    let mut results = Vec::with_capacity(players.len());

    for player in players {
        match scout_single(&client, &config, &player).await {
            Ok(profile) => results.push(profile),
            Err(e) => results.push(scout_error_profile(&player, e)),
        }
    }

    update_session_enemy_intel(&results).await;

    Ok(json!({
        "players": results,
        "platform": config.platform,
        "routing": if config.routing_region.is_empty() {
            crate::riot_api::platform_to_routing(&config.platform).to_string()
        } else {
            config.routing_region.clone()
        },
    }))
}

async fn scout_single(
    client: &RiotApiClient,
    config: &RiotApiConfig,
    input: &ScoutPlayerInput,
) -> Result<Value, String> {
    let mut input = input.clone();
    if input.champion_id.is_none() {
        if let Some(ref name) = input.champion_name {
            input.champion_id = resolve_champion_id_by_name(name).await;
        }
    }

    let puuid = resolve_puuid(client, &input).await?;

    let mut profile = if let Some(cached) = scout_cache::get_cached_profile(&puuid).await {
        enrich_with_champion(cached, input.champion_id)
    } else {
        build_fresh_profile(client, config, &input, &puuid).await?
    };

    if input.include_champion_form {
        if let Some(champ_id) = input.champion_id {
            if profile.get("champion_form").is_none()
                || profile
                    .get("champion_form")
                    .and_then(|v| v.get("champion_id"))
                    .and_then(|v| v.as_u64())
                    != Some(champ_id)
            {
                if let Some(form) = fetch_champion_form(client, &puuid, champ_id).await {
                    profile["champion_form"] = form;
                }
            }
        }
    }

    if let Some(champ_id) = input.champion_id {
        profile = enrich_with_champion(profile, Some(champ_id));
    }

    Ok(profile)
}

async fn build_fresh_profile(
    client: &RiotApiClient,
    config: &RiotApiConfig,
    input: &ScoutPlayerInput,
    puuid: &str,
) -> Result<Value, String> {
    let riot_id = riot_id_label(input);
    let league = client.get_league_entries_by_puuid(puuid).await.ok();
    let masteries = client
        .get_champion_masteries_by_puuid(puuid)
        .await
        .unwrap_or_else(|_| json!([]));

    let solo = parse_solo_rank(&league);
    let flex = parse_flex_rank(&league);
    let top_masteries = parse_top_masteries(&masteries, 3);
    let champ_mastery = input
        .champion_id
        .and_then(|id| find_champion_mastery(&masteries, id));

    let mastery_score = masteries
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("championPoints").and_then(|v| v.as_u64()))
                .sum::<u64>()
        })
        .unwrap_or(0);

    let is_one_trick = detect_one_trick(&masteries, input.champion_id);

    let mut profile = json!({
        "puuid": puuid,
        "riot_id": riot_id,
        "position": input.position,
        "champion_id": input.champion_id,
        "champion_name": input.champion_name,
        "solo_rank": solo,
        "flex_rank": flex,
        "mastery_score": mastery_score,
        "top_masteries": top_masteries,
        "champion_mastery": champ_mastery,
        "is_one_trick": is_one_trick,
        "platform": config.platform,
        "error": null,
    });

    if let Some(solo) = profile.get("solo_rank").cloned() {
        if solo.is_null() {
            if let Some(flex) = profile.get("flex_rank").cloned() {
                if !flex.is_null() {
                    profile["primary_rank"] = flex;
                }
            }
        } else {
            profile["primary_rank"] = solo;
        }
    }

    if input.include_champion_form {
        if let Some(champ_id) = input.champion_id {
            if let Some(form) = fetch_champion_form(client, puuid, champ_id).await {
                profile["champion_form"] = form;
            }
        }
    }

    scout_cache::store_profile(puuid, profile.clone()).await;
    Ok(profile)
}

async fn fetch_champion_form(
    client: &RiotApiClient,
    puuid: &str,
    champion_id: u64,
) -> Option<Value> {
    let ids = client.get_match_ids(puuid, 0, CHAMP_FORM_SCAN).await.ok()?;
    if ids.is_empty() {
        return None;
    }

    let mut wins = 0u32;
    let mut losses = 0u32;
    let mut kills = 0u64;
    let mut deaths = 0u64;
    let mut assists = 0u64;
    let mut games = 0u32;

    for id in ids {
        if games >= CHAMP_FORM_TARGET {
            break;
        }

        let match_data = if let Some(cached) = riot_api_cache::get_cached_match(&id).await {
            cached
        } else {
            match client.get_match(&id).await {
                Ok(m) => {
                    riot_api_cache::store_match(&id, m.clone()).await;
                    m
                }
                Err(_) => continue,
            }
        };

        let Some(player) = find_participant(&match_data, puuid) else {
            continue;
        };
        if player.get("championId").and_then(|v| v.as_u64()) != Some(champion_id) {
            continue;
        }

        games += 1;
        if player.get("win").and_then(|v| v.as_bool()).unwrap_or(false) {
            wins += 1;
        } else {
            losses += 1;
        }
        kills += player.get("kills").and_then(|v| v.as_u64()).unwrap_or(0);
        deaths += player.get("deaths").and_then(|v| v.as_u64()).unwrap_or(0);
        assists += player.get("assists").and_then(|v| v.as_u64()).unwrap_or(0);
    }

    if games == 0 {
        return None;
    }

    let win_rate = ((wins as f64 / games as f64) * 100.0).round();
    let avg_kda = if deaths > 0 {
        (kills + assists) as f64 / deaths as f64
    } else {
        (kills + assists) as f64
    };

    Some(json!({
        "champion_id": champion_id,
        "games": games,
        "wins": wins,
        "losses": losses,
        "win_rate": win_rate,
        "avg_kda": (avg_kda * 100.0).round() / 100.0,
        "summary": format!("{}W {}L ({}%) on champ", wins, losses, win_rate as u32),
    }))
}

fn find_participant(match_data: &Value, puuid: &str) -> Option<Value> {
    let metadata = match_data.get("metadata")?;
    let info = match_data.get("info")?;
    let participants_meta = metadata.get("participants")?.as_array()?;
    let player_index = participants_meta
        .iter()
        .position(|p| p.as_str() == Some(puuid))?;
    info.get("participants")?
        .as_array()?
        .get(player_index)
        .cloned()
}

async fn resolve_champion_id_by_name(name: &str) -> Option<u64> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lcu_api = crate::get_lcu_api().lock().await;
    if let Ok(champions) = lcu_api.get_champions().await {
        if let Some(id) = lookup_champion_id(&champions, trimmed) {
            return Some(id);
        }
    }

    None
}

fn lookup_champion_id(champions: &Value, name: &str) -> Option<u64> {
    let target = name.to_lowercase();

    let try_match = |champ: &Value| -> Option<u64> {
        let id = champ.get("id").and_then(|v| v.as_u64())?;
        let champ_name = champ.get("name").and_then(|v| v.as_str())?;
        if champ_name.to_lowercase() == target {
            return Some(id);
        }
        champ
            .get("alias")
            .and_then(|v| v.as_str())
            .filter(|a| a.to_lowercase() == target)
            .map(|_| id)
    };

    if let Some(arr) = champions.as_array() {
        for champ in arr {
            if let Some(id) = try_match(champ) {
                return Some(id);
            }
        }
    } else if let Some(obj) = champions.as_object() {
        for champ in obj.values() {
            if let Some(id) = try_match(champ) {
                return Some(id);
            }
        }
    }

    None
}

async fn update_session_enemy_intel(profiles: &[Value]) {
    let intel: Vec<Value> = profiles
        .iter()
        .filter_map(compact_intel_entry)
        .collect();

    if intel.is_empty() {
        return;
    }

    scout_cache::set_session_enemy_intel(Value::Array(intel)).await;
}

fn compact_intel_entry(profile: &Value) -> Option<Value> {
    if profile.get("error").and_then(|v| v.as_str()).is_some() {
        return None;
    }

    let rank = profile
        .get("primary_rank")
        .or_else(|| profile.get("solo_rank"))
        .and_then(|r| r.get("summary").and_then(|v| v.as_str()))
        .unwrap_or("Unranked");

    let mut entry = json!({
        "riot_id": profile.get("riot_id").cloned().unwrap_or(Value::Null),
        "position": profile.get("position").cloned().unwrap_or(Value::Null),
        "rank": rank,
        "is_one_trick": profile.get("is_one_trick").and_then(|v| v.as_bool()).unwrap_or(false),
    });

    if let Some(m) = profile.get("champion_mastery") {
        if let (Some(level), Some(points)) = (
            m.get("level").and_then(|v| v.as_u64()),
            m.get("points").and_then(|v| v.as_u64()),
        ) {
            entry["mastery"] = json!(format!("M{} · {} pts", level, points));
        }
    }

    if let Some(form) = profile.get("champion_form") {
        if let Some(summary) = form.get("summary").and_then(|v| v.as_str()) {
            entry["recent_form"] = json!(summary);
        }
    }

    if let Some(name) = profile.get("champion_name").and_then(|v| v.as_str()) {
        entry["champion"] = json!(name);
    } else if let Some(id) = profile.get("champion_id").and_then(|v| v.as_u64()) {
        entry["champion_id"] = json!(id);
    }

    Some(entry)
}

fn scout_error_profile(input: &ScoutPlayerInput, error: String) -> Value {
    json!({
        "puuid": input.puuid,
        "riot_id": riot_id_label(input),
        "position": input.position,
        "champion_id": input.champion_id,
        "champion_name": input.champion_name,
        "solo_rank": null,
        "flex_rank": null,
        "primary_rank": null,
        "mastery_score": 0,
        "top_masteries": [],
        "champion_mastery": null,
        "champion_form": null,
        "is_one_trick": false,
        "error": error,
    })
}

async fn resolve_puuid(client: &RiotApiClient, input: &ScoutPlayerInput) -> Result<String, String> {
    if let Some(ref id) = input.puuid {
        if !id.is_empty() {
            return Ok(id.clone());
        }
    }

    let game_name = input
        .game_name
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());
    let tag_line = input
        .tag_line
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    match (game_name, tag_line) {
        (Some(name), Some(tag)) => {
            let account = client.get_account_by_riot_id(name, tag).await?;
            account
                .get("puuid")
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .ok_or_else(|| format!("No PUUID returned for {}#{}", name, tag))
        }
        _ => Err("Player identity missing — need PUUID or Riot ID (gameName#tagLine)".to_string()),
    }
}

fn riot_id_label(input: &ScoutPlayerInput) -> Value {
    match (&input.game_name, &input.tag_line) {
        (Some(name), Some(tag)) if !name.is_empty() && !tag.is_empty() => {
            json!(format!("{}#{}", name.trim(), tag.trim()))
        }
        _ => Value::Null,
    }
}

fn enrich_with_champion(mut profile: Value, champion_id: Option<u64>) -> Value {
    if let Some(id) = champion_id {
        if let Some(obj) = profile.as_object_mut() {
            obj.insert("champion_id".into(), json!(id));
            if let Some(masteries) = obj.get("top_masteries").and_then(|v| v.as_array()) {
                let on_champ = masteries.iter().any(|m| {
                    m.get("champion_id")
                        .and_then(|v| v.as_u64())
                        .map(|cid| cid == id)
                        .unwrap_or(false)
                });
                obj.insert("champion_in_top_masteries".into(), json!(on_champ));
            }
        }
    }
    profile
}

fn parse_solo_rank(league: &Option<Value>) -> Value {
    parse_queue_rank(league, "RANKED_SOLO_5x5")
}

fn parse_flex_rank(league: &Option<Value>) -> Value {
    parse_queue_rank(league, "RANKED_FLEX_SR")
}

fn parse_queue_rank(league: &Option<Value>, queue_type: &str) -> Value {
    let Some(entries) = league.as_ref().and_then(|v| v.as_array()) else {
        return Value::Null;
    };

    let entry = entries
        .iter()
        .find(|e| e.get("queueType").and_then(|v| v.as_str()) == Some(queue_type));

    let Some(entry) = entry else {
        return Value::Null;
    };

    let tier = entry
        .get("tier")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if tier.is_empty() {
        return Value::Null;
    }

    let division = entry
        .get("rank")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let lp = entry
        .get("leaguePoints")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let wins = entry.get("wins").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let losses = entry.get("losses").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let total = wins + losses;
    let win_rate = if total > 0 {
        ((wins as f64 / total as f64) * 100.0).round()
    } else {
        0.0
    };

    let summary = if lp > 0 || division.is_empty() {
        format!("{} {} · {} LP", tier, division, lp).trim().to_string()
    } else {
        format!("{} {}", tier, division)
    };

    json!({
        "queue_type": queue_type,
        "tier": tier,
        "division": division,
        "lp": lp,
        "wins": wins,
        "losses": losses,
        "win_rate": win_rate,
        "summary": summary,
    })
}

fn parse_top_masteries(masteries: &Value, limit: usize) -> Value {
    let Some(arr) = masteries.as_array() else {
        return json!([]);
    };

    let mut items: Vec<Value> = arr
        .iter()
        .filter_map(|m| {
            let champion_id = m.get("championId").and_then(|v| v.as_u64())?;
            let level = m.get("championLevel").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let points = m
                .get("championPoints")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            Some(json!({
                "champion_id": champion_id,
                "level": level,
                "points": points,
            }))
        })
        .collect();

    items.sort_by(|a, b| {
        let pa = a.get("points").and_then(|v| v.as_u64()).unwrap_or(0);
        let pb = b.get("points").and_then(|v| v.as_u64()).unwrap_or(0);
        pb.cmp(&pa)
    });
    items.truncate(limit);
    json!(items)
}

fn find_champion_mastery(masteries: &Value, champion_id: u64) -> Option<Value> {
    let arr = masteries.as_array()?;
    let entry = arr
        .iter()
        .find(|m| m.get("championId").and_then(|v| v.as_u64()) == Some(champion_id))?;

    Some(json!({
        "champion_id": champion_id,
        "level": entry.get("championLevel").and_then(|v| v.as_u64()).unwrap_or(0),
        "points": entry.get("championPoints").and_then(|v| v.as_u64()).unwrap_or(0),
        "tokens_earned": entry.get("tokensEarned").and_then(|v| v.as_u64()).unwrap_or(0),
    }))
}

fn detect_one_trick(masteries: &Value, locked_champion: Option<u64>) -> bool {
    let Some(arr) = masteries.as_array() else {
        return false;
    };
    if arr.is_empty() {
        return false;
    }

    let top = arr
        .iter()
        .max_by_key(|m| m.get("championPoints").and_then(|v| v.as_u64()).unwrap_or(0));
    let Some(top) = top else {
        return false;
    };

    let top_id = top.get("championId").and_then(|v| v.as_u64());
    let top_points = top.get("championPoints").and_then(|v| v.as_u64()).unwrap_or(0);
    let top_level = top.get("championLevel").and_then(|v| v.as_u64()).unwrap_or(0);

    if top_level < 6 || top_points < 150_000 {
        return false;
    }

    if let Some(locked) = locked_champion {
        return top_id == Some(locked);
    }

    let second_points = arr
        .iter()
        .filter(|m| {
            m.get("championId")
                .and_then(|v| v.as_u64())
                .map(|id| Some(id) != top_id)
                .unwrap_or(false)
        })
        .filter_map(|m| m.get("championPoints").and_then(|v| v.as_u64()))
        .max()
        .unwrap_or(0);

    second_points * 3 < top_points
}

fn current_riot_config() -> RiotApiConfig {
    crate::get_config_manager()
        .lock()
        .map(|m| resolve_riot_config(&m.get_config().riot_api))
        .unwrap_or_else(|_| resolve_riot_config(&RiotApiConfig::default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_solo_rank_from_entries() {
        let league = json!([
            {
                "queueType": "RANKED_SOLO_5x5",
                "tier": "GOLD",
                "rank": "II",
                "leaguePoints": 45,
                "wins": 50,
                "losses": 48
            }
        ]);
        let solo = parse_solo_rank(&Some(league));
        assert_eq!(solo["tier"], "GOLD");
        assert_eq!(solo["division"], "II");
        assert_eq!(solo["lp"], 45);
    }

    #[test]
    fn detect_one_trick_on_locked_champion() {
        let masteries = json!([
            { "championId": 157, "championLevel": 7, "championPoints": 500000 },
            { "championId": 64, "championLevel": 5, "championPoints": 80000 }
        ]);
        assert!(detect_one_trick(&masteries, Some(157)));
        assert!(!detect_one_trick(&masteries, Some(64)));
    }

    #[test]
    fn compact_intel_entry_builds_summary() {
        let profile = json!({
            "riot_id": "Player#NA1",
            "position": "MIDDLE",
            "primary_rank": { "summary": "Gold II · 45 LP" },
            "champion_mastery": { "level": 7, "points": 400000 },
            "champion_form": { "summary": "6W 4L (60%) on champ" },
            "is_one_trick": true,
            "champion_name": "Yasuo"
        });
        let entry = compact_intel_entry(&profile).unwrap();
        assert_eq!(entry["rank"], "Gold II · 45 LP");
        assert_eq!(entry["recent_form"], "6W 4L (60%) on champ");
    }
}
