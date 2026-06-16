use crate::lcu_api::{SummonerSpell, SpellImage, LCUAPI, LCU_BASE_URL, LCU_SUMMONER_SPELLS_ENDPOINT};

impl LCUAPI {
    // Get summoner spells as structured data
    pub async fn get_summoner_spells_structured(&self) -> Result<Vec<SummonerSpell>, String> {
        println!("[LCU] Fetching summoner spells data...");
        
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        let url = format!("{}:{}{}", LCU_BASE_URL, connection.port, LCU_SUMMONER_SPELLS_ENDPOINT);
        let response = self.make_request(&url, "GET", None).await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to fetch summoner spells: HTTP {}", response.status()));
        }
        
        let spells_json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse summoner spells JSON: {}", e))?;
        
        let mut spells = Vec::new();
        
        if let Some(spells_data) = spells_json.as_object() {
            for (spell_id_str, spell_data) in spells_data {
                if let Ok(spell_id) = spell_id_str.parse::<u64>() {
                    if let Some(spell_obj) = spell_data.as_object() {
                        let name = spell_obj.get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        
                        let description = spell_obj.get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("No description")
                            .to_string();
                        
                        let tooltip = spell_obj.get("tooltip")
                            .and_then(|t| t.as_str())
                            .unwrap_or("No tooltip")
                            .to_string();
                        
                        let cooldown = spell_obj.get("cooldown")
                            .and_then(|c| c.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|v| v.as_f64())
                                .collect())
                            .unwrap_or_default();
                        
                        let cost = spell_obj.get("cost")
                            .and_then(|c| c.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|v| v.as_f64())
                                .collect())
                            .unwrap_or_default();
                        
                        let range = spell_obj.get("range")
                            .and_then(|r| r.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|v| v.as_f64())
                                .collect())
                            .unwrap_or_default();
                        
                        let maxrank = spell_obj.get("maxrank")
                            .and_then(|m| m.as_u64())
                            .unwrap_or(1) as u8;
                        
                        let effect = spell_obj.get("effect")
                            .and_then(|e| e.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|v| v.as_f64())
                                .collect())
                            .unwrap_or_default();
                        
                        let image = if let Some(img_data) = spell_obj.get("image").and_then(|i| i.as_object()) {
                            SpellImage {
                                full: img_data.get("full").and_then(|f| f.as_str()).unwrap_or("").to_string(),
                                sprite: img_data.get("sprite").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                                group: img_data.get("group").and_then(|g| g.as_str()).unwrap_or("").to_string(),
                                x: img_data.get("x").and_then(|x| x.as_u64()).unwrap_or(0) as u32,
                                y: img_data.get("y").and_then(|y| y.as_u64()).unwrap_or(0) as u32,
                                w: img_data.get("w").and_then(|w| w.as_u64()).unwrap_or(0) as u32,
                                h: img_data.get("h").and_then(|h| h.as_u64()).unwrap_or(0) as u32,
                            }
                        } else {
                            SpellImage {
                                full: "".to_string(),
                                sprite: "".to_string(),
                                group: "".to_string(),
                                x: 0, y: 0, w: 0, h: 0,
                            }
                        };
                        
                        spells.push(SummonerSpell {
                            id: spell_id,
                            name,
                            description,
                            tooltip,
                            cooldown,
                            cost,
                            range,
                            maxrank,
                            effect,
                            image,
                        });
                    }
                }
            }
        }
        
        println!("[LCU] Successfully fetched {} summoner spells", spells.len());
        Ok(spells)
    }
}
