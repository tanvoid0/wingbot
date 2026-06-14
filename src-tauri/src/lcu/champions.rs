use crate::lcu_api::{Champion, ChampionStats, ChampionAbility, ChampionPassive, ChampionImage, LCUAPI, LCU_BASE_URL, LCU_CHAMPIONS_ENDPOINT};

impl LCUAPI {
    // Get champions as structured data
    pub async fn get_champions_structured(&self) -> Result<Vec<Champion>, String> {
        println!("[LCU] Fetching champions data...");
        
        let url = format!("{}{}", LCU_BASE_URL, LCU_CHAMPIONS_ENDPOINT);
        let response = self.make_request(&url, "GET", None).await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to fetch champions: HTTP {}", response.status()));
        }
        
        let champions_json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse champions JSON: {}", e))?;
        
        let mut champions = Vec::new();
        
        if let Some(champions_data) = champions_json.as_object() {
            for (champion_id_str, champion_data) in champions_data {
                if let Ok(champion_id) = champion_id_str.parse::<u64>() {
                    if let Some(champion_obj) = champion_data.as_object() {
                        // Parse champion data
                        let name = champion_obj.get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        
                        let title = champion_obj.get("title")
                            .and_then(|t| t.as_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        
                        let description = champion_obj.get("blurb")
                            .and_then(|d| d.as_str())
                            .unwrap_or("No description")
                            .to_string();
                        
                        let tags = champion_obj.get("tags")
                            .and_then(|t| t.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| s.to_string())
                                .collect())
                            .unwrap_or_default();
                        
                        // Parse stats
                        let stats = if let Some(stats_data) = champion_obj.get("stats").and_then(|s| s.as_object()) {
                            ChampionStats {
                                hp: stats_data.get("hp").and_then(|h| h.as_f64()).unwrap_or(0.0),
                                hpperlevel: stats_data.get("hpperlevel").and_then(|h| h.as_f64()).unwrap_or(0.0),
                                mp: stats_data.get("mp").and_then(|m| m.as_f64()).unwrap_or(0.0),
                                mpperlevel: stats_data.get("mpperlevel").and_then(|m| m.as_f64()).unwrap_or(0.0),
                                movespeed: stats_data.get("movespeed").and_then(|m| m.as_f64()).unwrap_or(0.0),
                                armor: stats_data.get("armor").and_then(|a| a.as_f64()).unwrap_or(0.0),
                                armorperlevel: stats_data.get("armorperlevel").and_then(|a| a.as_f64()).unwrap_or(0.0),
                                spellblock: stats_data.get("spellblock").and_then(|s| s.as_f64()).unwrap_or(0.0),
                                spellblockperlevel: stats_data.get("spellblockperlevel").and_then(|s| s.as_f64()).unwrap_or(0.0),
                                attackrange: stats_data.get("attackrange").and_then(|a| a.as_f64()).unwrap_or(0.0),
                                hpregen: stats_data.get("hpregen").and_then(|h| h.as_f64()).unwrap_or(0.0),
                                hpregenperlevel: stats_data.get("hpregenperlevel").and_then(|h| h.as_f64()).unwrap_or(0.0),
                                mpregen: stats_data.get("mpregen").and_then(|m| m.as_f64()).unwrap_or(0.0),
                                mpregenperlevel: stats_data.get("mpregenperlevel").and_then(|m| m.as_f64()).unwrap_or(0.0),
                                crit: stats_data.get("crit").and_then(|c| c.as_f64()).unwrap_or(0.0),
                                critperlevel: stats_data.get("critperlevel").and_then(|c| c.as_f64()).unwrap_or(0.0),
                                attackdamage: stats_data.get("attackdamage").and_then(|a| a.as_f64()).unwrap_or(0.0),
                                attackdamageperlevel: stats_data.get("attackdamageperlevel").and_then(|a| a.as_f64()).unwrap_or(0.0),
                                attackspeedperlevel: stats_data.get("attackspeedperlevel").and_then(|a| a.as_f64()).unwrap_or(0.0),
                                attackspeed: stats_data.get("attackspeed").and_then(|a| a.as_f64()).unwrap_or(0.0),
                            }
                        } else {
                            ChampionStats {
                                hp: 0.0, hpperlevel: 0.0, mp: 0.0, mpperlevel: 0.0,
                                movespeed: 0.0, armor: 0.0, armorperlevel: 0.0,
                                spellblock: 0.0, spellblockperlevel: 0.0, attackrange: 0.0,
                                hpregen: 0.0, hpregenperlevel: 0.0, mpregen: 0.0, mpregenperlevel: 0.0,
                                crit: 0.0, critperlevel: 0.0, attackdamage: 0.0, attackdamageperlevel: 0.0,
                                attackspeedperlevel: 0.0, attackspeed: 0.0,
                            }
                        };
                        
                        // Parse abilities from champion data
                        let abilities = if let Some(spells_data) = champion_obj.get("spells").and_then(|s| s.as_array()) {
                            spells_data.iter()
                                .filter_map(|spell| {
                                    if let Some(spell_obj) = spell.as_object() {
                                        Some(ChampionAbility {
                                            id: spell_obj.get("id")
                                                .and_then(|id| id.as_str())
                                                .unwrap_or("unknown")
                                                .to_string(),
                                            name: spell_obj.get("name")
                                                .and_then(|n| n.as_str())
                                                .unwrap_or("Unknown Ability")
                                                .to_string(),
                                            description: spell_obj.get("description")
                                                .and_then(|d| d.as_str())
                                                .unwrap_or("No description")
                                                .to_string(),
                                            tooltip: spell_obj.get("tooltip")
                                                .and_then(|t| t.as_str())
                                                .unwrap_or("No tooltip")
                                                .to_string(),
                                            cooldown: spell_obj.get("cooldown")
                                                .and_then(|c| c.as_array())
                                                .map(|arr| arr.iter()
                                                    .filter_map(|v| v.as_f64())
                                                    .collect())
                                                .unwrap_or_default(),
                                            cost: spell_obj.get("cost")
                                                .and_then(|c| c.as_array())
                                                .map(|arr| arr.iter()
                                                    .filter_map(|v| v.as_f64())
                                                    .collect())
                                                .unwrap_or_default(),
                                            range: spell_obj.get("range")
                                                .and_then(|r| r.as_array())
                                                .map(|arr| arr.iter()
                                                    .filter_map(|v| v.as_f64())
                                                    .collect())
                                                .unwrap_or_default(),
                                            maxrank: spell_obj.get("maxrank")
                                                .and_then(|m| m.as_u64())
                                                .unwrap_or(5) as u8,
                                            effect: spell_obj.get("effect")
                                                .and_then(|e| e.as_array())
                                                .map(|arr| arr.iter()
                                                    .filter_map(|v| v.as_f64())
                                                    .collect())
                                                .unwrap_or_default(),
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .collect()
                        } else {
                            Vec::new()
                        };
                        
                        // Parse passive ability
                        let passive = if let Some(passive_data) = champion_obj.get("passive").and_then(|p| p.as_object()) {
                            ChampionPassive {
                                name: passive_data.get("name")
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("Passive")
                                    .to_string(),
                                description: passive_data.get("description")
                                    .and_then(|d| d.as_str())
                                    .unwrap_or("Champion passive ability")
                                    .to_string(),
                                image: if let Some(img_data) = passive_data.get("image").and_then(|i| i.as_object()) {
                                    ChampionImage {
                                        full: img_data.get("full").and_then(|f| f.as_str()).unwrap_or("").to_string(),
                                        sprite: img_data.get("sprite").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                                        group: img_data.get("group").and_then(|g| g.as_str()).unwrap_or("").to_string(),
                                        x: img_data.get("x").and_then(|x| x.as_u64()).unwrap_or(0) as u32,
                                        y: img_data.get("y").and_then(|y| y.as_u64()).unwrap_or(0) as u32,
                                        w: img_data.get("w").and_then(|w| w.as_u64()).unwrap_or(0) as u32,
                                        h: img_data.get("h").and_then(|h| h.as_u64()).unwrap_or(0) as u32,
                                    }
                                } else {
                                    ChampionImage {
                                        full: "".to_string(),
                                        sprite: "".to_string(),
                                        group: "".to_string(),
                                        x: 0, y: 0, w: 0, h: 0,
                                    }
                                },
                            }
                        } else {
                            ChampionPassive {
                                name: "Passive".to_string(),
                                description: "Champion passive ability".to_string(),
                                image: ChampionImage {
                                    full: "".to_string(),
                                    sprite: "".to_string(),
                                    group: "".to_string(),
                                    x: 0, y: 0, w: 0, h: 0,
                                },
                            }
                        };
                        
                        champions.push(Champion {
                            id: champion_id,
                            name,
                            title,
                            description,
                            tags,
                            stats,
                            abilities,
                            passive,
                        });
                    }
                }
            }
        }
        
        println!("[LCU] Successfully fetched {} champions", champions.len());
        Ok(champions)
    }
}
