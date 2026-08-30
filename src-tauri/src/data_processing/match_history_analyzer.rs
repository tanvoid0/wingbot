use serde_json::{json, Value};

/// Aggregated match-history stats consumed by the frontend (`matchHistoryStore`).
#[derive(Debug, Clone)]
pub struct MatchHistoryAnalysis {
    pub source: &'static str,
    pub puuid: Option<String>,
    pub total_matches: u32,
    pub wins: u32,
    pub losses: u32,
    pub win_rate: f64,
    pub avg_kills: f64,
    pub avg_deaths: f64,
    pub avg_assists: f64,
    pub avg_cs: f64,
    pub avg_damage: f64,
    pub avg_gold: f64,
    pub avg_vision: f64,
    pub avg_impact: f64,
    pub champion_performance: Value,
    pub role_stats: Value,
    pub insights: Value,
    pub raw_matches: Value,
    pub matches: Value,
}

impl MatchHistoryAnalysis {
    pub fn to_json(&self) -> Value {
        json!({
            "source": self.source,
            "puuid": self.puuid,
            "total_matches": self.total_matches,
            "wins": self.wins,
            "losses": self.losses,
            "win_rate": self.win_rate,
            "avg_kills": self.avg_kills,
            "avg_deaths": self.avg_deaths,
            "avg_assists": self.avg_assists,
            "avg_cs": self.avg_cs,
            "avg_damage": self.avg_damage,
            "avg_gold": self.avg_gold,
            "avg_vision": self.avg_vision,
            "avg_impact": self.avg_impact,
            "champion_performance": self.champion_performance,
            "role_stats": self.role_stats,
            "insights": self.insights,
            "raw_matches": self.raw_matches,
            "matches": self.matches,
        })
    }
}

pub struct MatchHistoryAnalyzer;

struct NormalizedStats {
    win: bool,
    kills: u64,
    deaths: u64,
    assists: u64,
    cs: u64,
    damage: u64,
    gold: u64,
    vision: u64,
}

impl MatchHistoryAnalyzer {
    pub fn from_lcu(data: &Value, puuid: Option<&str>) -> Option<MatchHistoryAnalysis> {
        let games = data
            .get("games")
            .and_then(|g| g.get("games"))
            .and_then(|g| g.as_array())?;

        if games.is_empty() {
            return None;
        }

        let normalized: Vec<Value> = games
            .iter()
            .filter_map(|game| Self::normalize_lcu_game(game))
            .collect();

        if normalized.is_empty() {
            return None;
        }

        Some(Self::aggregate("lcu", puuid.map(str::to_string), normalized))
    }

    pub fn from_riot_matches(matches: &[Value], puuid: &str) -> Option<MatchHistoryAnalysis> {
        if matches.is_empty() {
            return None;
        }

        let normalized: Vec<Value> = matches
            .iter()
            .filter_map(|m| Self::normalize_riot_match(m, puuid))
            .collect();

        if normalized.is_empty() {
            return None;
        }

        Some(Self::aggregate("riot", Some(puuid.to_string()), normalized))
    }

    fn normalize_lcu_game(game: &Value) -> Option<Value> {
        let participant = game.get("participants")?.as_array()?.first()?;
        let stats = participant.get("stats")?;
        let champion_id = participant
            .get("championId")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let game_creation = game
            .get("gameCreationDate")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp_millis())
            .or_else(|| game.get("gameCreation").and_then(|v| v.as_i64()))
            .unwrap_or(0);

        let duration = game.get("gameDuration").and_then(|v| v.as_u64()).unwrap_or(0);
        let queue_id = game.get("queueId").and_then(|v| v.as_u64()).unwrap_or(0);

        let normalized_stats = Self::parse_stats(stats);
        let impact = Self::compute_impact(&normalized_stats, duration);

        Some(json!({
            "gameId": game.get("gameId").cloned().unwrap_or(Value::Null),
            "gameCreation": game_creation,
            "gameDuration": duration,
            "gameMode": game.get("gameMode").and_then(|v| v.as_str()).unwrap_or(""),
            "queueId": queue_id,
            "queueName": queue_label(queue_id),
            "championId": champion_id,
            "championName": participant.get("championName")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            "role": participant.get("teamPosition")
                .or_else(|| participant.get("lane"))
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            "kda": kda_ratio(normalized_stats.kills, normalized_stats.deaths, normalized_stats.assists),
            "impactScore": impact,
            "impactLabel": impact_label(impact),
            "stats": Self::stats_json(&normalized_stats),
        }))
    }

    fn normalize_riot_match(match_data: &Value, puuid: &str) -> Option<Value> {
        let metadata = match_data.get("metadata")?;
        let info = match_data.get("info")?;
        let participants_meta = metadata.get("participants")?.as_array()?;
        let player_index = participants_meta
            .iter()
            .position(|p| p.as_str() == Some(puuid))?;

        let player = info.get("participants")?.as_array()?.get(player_index)?;

        let game_creation = info.get("gameCreation").and_then(|v| v.as_i64()).unwrap_or(0);
        let duration = info.get("gameDuration").and_then(|v| v.as_u64()).unwrap_or(0);
        let queue_id = info.get("queueId").and_then(|v| v.as_u64()).unwrap_or(0);
        let champion_id = player.get("championId").and_then(|v| v.as_u64()).unwrap_or(0);

        let normalized_stats = Self::parse_riot_stats(player);
        let impact = Self::compute_impact(&normalized_stats, duration);

        let role = player
            .get("teamPosition")
            .or_else(|| player.get("individualPosition"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        Some(json!({
            "gameId": metadata.get("matchId").cloned().unwrap_or(Value::Null),
            "gameCreation": game_creation,
            "gameDuration": duration,
            "gameMode": info.get("gameMode").and_then(|v| v.as_str()).unwrap_or(""),
            "queueId": queue_id,
            "queueName": queue_label(queue_id),
            "championId": champion_id,
            "championName": player.get("championName").and_then(|v| v.as_str()).unwrap_or(""),
            "role": role,
            "kda": kda_ratio(normalized_stats.kills, normalized_stats.deaths, normalized_stats.assists),
            "impactScore": impact,
            "impactLabel": impact_label(impact),
            "stats": Self::stats_json(&normalized_stats),
        }))
    }

    fn parse_stats(stats: &Value) -> NormalizedStats {
        NormalizedStats {
            win: stats.get("win").and_then(|v| v.as_bool()).unwrap_or(false),
            kills: stats.get("kills").and_then(|v| v.as_u64()).unwrap_or(0),
            deaths: stats.get("deaths").and_then(|v| v.as_u64()).unwrap_or(0),
            assists: stats.get("assists").and_then(|v| v.as_u64()).unwrap_or(0),
            cs: stats.get("totalMinionsKilled").and_then(|v| v.as_u64()).unwrap_or(0),
            damage: stats
                .get("totalDamageDealtToChampions")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            gold: stats.get("goldEarned").and_then(|v| v.as_u64()).unwrap_or(0),
            vision: stats
                .get("visionScore")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        }
    }

    fn parse_riot_stats(player: &Value) -> NormalizedStats {
        NormalizedStats {
            win: player.get("win").and_then(|v| v.as_bool()).unwrap_or(false),
            kills: player.get("kills").and_then(|v| v.as_u64()).unwrap_or(0),
            deaths: player.get("deaths").and_then(|v| v.as_u64()).unwrap_or(0),
            assists: player.get("assists").and_then(|v| v.as_u64()).unwrap_or(0),
            cs: player.get("totalMinionsKilled").and_then(|v| v.as_u64()).unwrap_or(0)
                + player
                    .get("neutralMinionsKilled")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
            damage: player
                .get("totalDamageDealtToChampions")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            gold: player.get("goldEarned").and_then(|v| v.as_u64()).unwrap_or(0),
            vision: player.get("visionScore").and_then(|v| v.as_u64()).unwrap_or(0),
        }
    }

    fn stats_json(s: &NormalizedStats) -> Value {
        json!({
            "win": s.win,
            "kills": s.kills,
            "deaths": s.deaths,
            "assists": s.assists,
            "cs": s.cs,
            "damage": s.damage,
            "gold": s.gold,
            "vision": s.vision,
        })
    }

    fn compute_impact(s: &NormalizedStats, duration_secs: u64) -> f64 {
        let mins = (duration_secs as f64 / 60.0).max(1.0);
        let kda = kda_ratio(s.kills, s.deaths, s.assists);
        let kda_score = (kda / 5.0).min(1.0) * 30.0;
        let win_bonus = if s.win { 25.0 } else { 0.0 };
        let cs_per_min = s.cs as f64 / mins;
        let cs_score = (cs_per_min / 10.0).min(1.0) * 20.0;
        let dmg_per_min = s.damage as f64 / mins;
        let dmg_score = (dmg_per_min / 800.0).min(1.0) * 25.0;
        (kda_score + win_bonus + cs_score + dmg_score).min(100.0)
    }

    fn aggregate(
        source: &'static str,
        puuid: Option<String>,
        matches: Vec<Value>,
    ) -> MatchHistoryAnalysis {
        let total = matches.len() as u32;
        let mut wins = 0u32;
        let mut total_kills = 0u64;
        let mut total_deaths = 0u64;
        let mut total_assists = 0u64;
        let mut total_cs = 0u64;
        let mut total_damage = 0u64;
        let mut total_gold = 0u64;
        let mut total_vision = 0u64;
        let mut total_impact = 0f64;
        let mut champion_perf = serde_json::Map::new();
        let mut role_stats = serde_json::Map::new();

        for m in &matches {
            let stats = m.get("stats").cloned().unwrap_or(Value::Null);
            let win = stats.get("win").and_then(|v| v.as_bool()).unwrap_or(false);
            if win {
                wins += 1;
            }

            let kills = stats.get("kills").and_then(|v| v.as_u64()).unwrap_or(0);
            let deaths = stats.get("deaths").and_then(|v| v.as_u64()).unwrap_or(0);
            let assists = stats.get("assists").and_then(|v| v.as_u64()).unwrap_or(0);
            let cs = stats.get("cs").and_then(|v| v.as_u64()).unwrap_or(0);
            let damage = stats.get("damage").and_then(|v| v.as_u64()).unwrap_or(0);
            let gold = stats.get("gold").and_then(|v| v.as_u64()).unwrap_or(0);
            let vision = stats.get("vision").and_then(|v| v.as_u64()).unwrap_or(0);
            let impact = m.get("impactScore").and_then(|v| v.as_f64()).unwrap_or(0.0);

            total_kills += kills;
            total_deaths += deaths;
            total_assists += assists;
            total_cs += cs;
            total_damage += damage;
            total_gold += gold;
            total_vision += vision;
            total_impact += impact;

            let champ_key = m
                .get("championId")
                .and_then(|v| v.as_u64())
                .map(|id| id.to_string())
                .unwrap_or_else(|| "unknown".to_string());

            Self::accumulate_bucket(
                &mut champion_perf,
                &champ_key,
                m.get("championName").and_then(|v| v.as_str()).unwrap_or(""),
                win,
                kills,
                deaths,
                assists,
                cs,
                damage,
                gold,
                vision,
                impact,
            );

            let role = m.get("role").and_then(|v| v.as_str()).unwrap_or("");
            if !role.is_empty() && role != "Invalid" {
                Self::accumulate_bucket(
                    &mut role_stats,
                    role,
                    role,
                    win,
                    kills,
                    deaths,
                    assists,
                    cs,
                    damage,
                    gold,
                    vision,
                    impact,
                );
            }
        }

        let losses = total.saturating_sub(wins);
        let n = total as f64;
        let win_rate = if total > 0 {
            (wins as f64 / n) * 100.0
        } else {
            0.0
        };

        let avg_impact = if total > 0 {
            total_impact / n
        } else {
            0.0
        };

        let insights = build_insights(
            win_rate,
            total_kills as f64 / n,
            total_deaths as f64 / n,
            total_assists as f64 / n,
            &matches,
        );

        MatchHistoryAnalysis {
            source,
            puuid,
            total_matches: total,
            wins,
            losses,
            win_rate,
            avg_kills: total_kills as f64 / n,
            avg_deaths: total_deaths as f64 / n,
            avg_assists: total_assists as f64 / n,
            avg_cs: total_cs as f64 / n,
            avg_damage: total_damage as f64 / n,
            avg_gold: total_gold as f64 / n,
            avg_vision: total_vision as f64 / n,
            avg_impact,
            champion_performance: finalize_buckets(champion_perf),
            role_stats: finalize_buckets(role_stats),
            insights,
            raw_matches: Value::Array(matches.clone()),
            matches: Value::Array(matches),
        }
    }

    fn accumulate_bucket(
        map: &mut serde_json::Map<String, Value>,
        key: &str,
        name: &str,
        win: bool,
        kills: u64,
        deaths: u64,
        assists: u64,
        cs: u64,
        damage: u64,
        gold: u64,
        vision: u64,
        impact: f64,
    ) {
        let entry = map.entry(key.to_string()).or_insert_with(|| {
            json!({
                "name": name,
                "champion_name": name,
                "games": 0,
                "wins": 0,
                "kills": 0,
                "deaths": 0,
                "assists": 0,
                "cs": 0,
                "damage": 0,
                "gold": 0,
                "vision": 0,
                "impact_total": 0.0,
            })
        });

        if let Some(obj) = entry.as_object_mut() {
            obj.insert("games".into(), json!(obj["games"].as_u64().unwrap_or(0) + 1));
            if win {
                obj.insert("wins".into(), json!(obj["wins"].as_u64().unwrap_or(0) + 1));
            }
            obj.insert("kills".into(), json!(obj["kills"].as_u64().unwrap_or(0) + kills));
            obj.insert("deaths".into(), json!(obj["deaths"].as_u64().unwrap_or(0) + deaths));
            obj.insert("assists".into(), json!(obj["assists"].as_u64().unwrap_or(0) + assists));
            obj.insert("cs".into(), json!(obj["cs"].as_u64().unwrap_or(0) + cs));
            obj.insert("damage".into(), json!(obj["damage"].as_u64().unwrap_or(0) + damage));
            obj.insert("gold".into(), json!(obj["gold"].as_u64().unwrap_or(0) + gold));
            obj.insert("vision".into(), json!(obj["vision"].as_u64().unwrap_or(0) + vision));
            obj.insert(
                "impact_total".into(),
                json!(obj["impact_total"].as_f64().unwrap_or(0.0) + impact),
            );
        }
    }
}

fn finalize_buckets(mut map: serde_json::Map<String, Value>) -> Value {
    for (_, entry) in map.iter_mut() {
        if let Some(obj) = entry.as_object_mut() {
            let games = obj.get("games").and_then(|v| v.as_u64()).unwrap_or(1).max(1) as f64;
            let impact_total = obj.get("impact_total").and_then(|v| v.as_f64()).unwrap_or(0.0);
            obj.insert("avg_impact".into(), json!(impact_total / games));
            let wins = obj.get("wins").and_then(|v| v.as_u64()).unwrap_or(0);
            obj.insert("win_rate".into(), json!((wins as f64 / games) * 100.0));
        }
    }
    Value::Object(map)
}

fn kda_ratio(kills: u64, deaths: u64, assists: u64) -> f64 {
    if deaths == 0 {
        return (kills + assists) as f64;
    }
    (kills + assists) as f64 / deaths as f64
}

fn impact_label(score: f64) -> &'static str {
    if score >= 85.0 {
        "Hard Carry"
    } else if score >= 70.0 {
        "Strong"
    } else if score >= 55.0 {
        "Solid"
    } else if score >= 40.0 {
        "Average"
    } else {
        "Weak"
    }
}

fn queue_label(queue_id: u64) -> &'static str {
    match queue_id {
        420 => "Ranked Solo",
        440 => "Ranked Flex",
        450 => "ARAM",
        400 => "Normal Draft",
        430 => "Normal Blind",
        490 => "Quickplay",
        1700 => "Arena",
        900 | 1900 => "URF",
        1020 => "One for All",
        1300 => "Nexus Blitz",
        1400 => "Ultimate Spellbook",
        _ => "Other",
    }
}

fn build_insights(
    win_rate: f64,
    avg_kills: f64,
    avg_deaths: f64,
    avg_assists: f64,
    matches: &[Value],
) -> Value {
    let mut insights = Vec::new();
    let avg_kda = kda_ratio(avg_kills as u64, avg_deaths as u64, avg_assists as u64);

    if win_rate >= 60.0 {
        insights.push(json!({
            "type": "positive",
            "title": "Strong Win Rate",
            "message": format!("{:.1}% win rate over recent games.", win_rate)
        }));
    } else if win_rate < 40.0 {
        insights.push(json!({
            "type": "negative",
            "title": "Low Win Rate",
            "message": format!("{:.1}% win rate — focus on consistency.", win_rate)
        }));
    }

    if avg_kda >= 3.0 {
        insights.push(json!({
            "type": "positive",
            "title": "High Impact Fighting",
            "message": format!("{:.2} KDA suggests strong teamfight value.", avg_kda)
        }));
    } else if avg_kda < 1.5 {
        insights.push(json!({
            "type": "negative",
            "title": "Deaths Costing Games",
            "message": format!("{:.2} KDA — safer positioning may help.", avg_kda)
        }));
    }

    if matches.len() >= 3 {
        let recent: Vec<bool> = matches
            .iter()
            .take(5)
            .filter_map(|m| m.get("stats")?.get("win")?.as_bool())
            .collect();
        if recent.len() >= 3 {
            let streak_wins = recent.iter().take_while(|&&w| w).count();
            let streak_losses = recent.iter().take_while(|&&w| !w).count();
            if streak_wins >= 3 {
                insights.push(json!({
                    "type": "positive",
                    "title": "Hot Streak",
                    "message": format!("{} wins in a row recently.", streak_wins)
                }));
            } else if streak_losses >= 3 {
                insights.push(json!({
                    "type": "negative",
                    "title": "Cold Streak",
                    "message": format!("{} losses in a row — consider a comfort pick.", streak_losses)
                }));
            }
        }
    }

    Value::Array(insights)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyzes_lcu_match_history() {
        let data = json!({
            "games": {
                "gameCount": 2,
                "games": [
                    {
                        "gameId": 1,
                        "gameDuration": 1800,
                        "gameCreationDate": "2024-06-01T12:00:00Z",
                        "gameMode": "CLASSIC",
                        "queueId": 420,
                        "participants": [{
                            "championId": 103,
                            "championName": "Ahri",
                            "teamPosition": "MIDDLE",
                            "stats": {
                                "win": true,
                                "kills": 10,
                                "deaths": 2,
                                "assists": 8,
                                "totalMinionsKilled": 180,
                                "goldEarned": 12000,
                                "totalDamageDealtToChampions": 22000,
                                "visionScore": 25
                            }
                        }]
                    },
                    {
                        "gameId": 2,
                        "gameDuration": 1600,
                        "gameCreationDate": "2024-06-02T12:00:00Z",
                        "gameMode": "CLASSIC",
                        "queueId": 420,
                        "participants": [{
                            "championId": 103,
                            "championName": "Ahri",
                            "teamPosition": "MIDDLE",
                            "stats": {
                                "win": false,
                                "kills": 3,
                                "deaths": 7,
                                "assists": 4,
                                "totalMinionsKilled": 140,
                                "goldEarned": 9000,
                                "totalDamageDealtToChampions": 11000,
                                "visionScore": 12
                            }
                        }]
                    }
                ]
            }
        });

        let analysis = MatchHistoryAnalyzer::from_lcu(&data, Some("test-puuid")).unwrap();
        assert_eq!(analysis.total_matches, 2);
        assert!(analysis.avg_impact > 0.0);
        assert!(analysis.insights.as_array().unwrap().len() >= 1);
    }

    #[test]
    fn analyzes_riot_match_v5() {
        let puuid = "player-puuid";
        let match_data = json!({
            "metadata": {
                "matchId": "NA1_123",
                "participants": [puuid, "other"]
            },
            "info": {
                "gameCreation": 1_700_000_000_000i64,
                "gameDuration": 2000,
                "gameMode": "CLASSIC",
                "queueId": 420,
                "participants": [
                    {
                        "puuid": puuid,
                        "championId": 64,
                        "championName": "LeeSin",
                        "teamPosition": "JUNGLE",
                        "win": true,
                        "kills": 5,
                        "deaths": 1,
                        "assists": 12,
                        "totalMinionsKilled": 150,
                        "neutralMinionsKilled": 20,
                        "totalDamageDealtToChampions": 18000,
                        "goldEarned": 11000,
                        "visionScore": 30
                    },
                    {
                        "puuid": "other",
                        "championId": 1,
                        "championName": "Annie",
                        "teamPosition": "MIDDLE",
                        "win": false,
                        "kills": 0,
                        "deaths": 10,
                        "assists": 0,
                        "totalMinionsKilled": 50,
                        "neutralMinionsKilled": 0,
                        "totalDamageDealtToChampions": 3000,
                        "goldEarned": 5000,
                        "visionScore": 5
                    }
                ]
            }
        });

        let analysis = MatchHistoryAnalyzer::from_riot_matches(&[match_data], puuid).unwrap();
        assert_eq!(analysis.matches[0]["queueName"], "Ranked Solo");
        assert!(analysis.matches[0]["impactScore"].as_f64().unwrap() > 50.0);
    }
}
