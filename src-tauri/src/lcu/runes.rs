use crate::lcu_api::{Rune, RuneSlot, RuneDetail, LCUAPI, LCU_BASE_URL, LCU_RUNES_ENDPOINT};

impl LCUAPI {
    // Get runes as structured data
    pub async fn get_runes_structured(&self) -> Result<Vec<Rune>, String> {
        println!("[LCU] Fetching runes data...");
        
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        let url = format!("{}:{}{}", LCU_BASE_URL, connection.port, LCU_RUNES_ENDPOINT);
        let response = self.make_request(&url, "GET", None).await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to fetch runes: HTTP {}", response.status()));
        }
        
        let runes_json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse runes JSON: {}", e))?;
        
        let mut runes = Vec::new();
        
        if let Some(runes_data) = runes_json.as_object() {
            for (rune_id_str, rune_data) in runes_data {
                if let Ok(rune_id) = rune_id_str.parse::<u64>() {
                    if let Some(rune_obj) = rune_data.as_object() {
                        let name = rune_obj.get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        
                        let icon = rune_obj.get("icon")
                            .and_then(|i| i.as_str())
                            .unwrap_or("")
                            .to_string();
                        
                        let short_desc = rune_obj.get("shortDesc")
                            .and_then(|s| s.as_str())
                            .unwrap_or("")
                            .to_string();
                        
                        let long_desc = rune_obj.get("longDesc")
                            .and_then(|l| l.as_str())
                            .unwrap_or("")
                            .to_string();
                        
                        // Parse rune slots from data
                        let slots = if let Some(slots_data) = rune_obj.get("slots").and_then(|s| s.as_array()) {
                            slots_data.iter()
                                .filter_map(|slot| {
                                    if let Some(slot_obj) = slot.as_object() {
                                        if let Some(runes_data) = slot_obj.get("runes").and_then(|r| r.as_array()) {
                                            let runes = runes_data.iter()
                                                .filter_map(|rune| {
                                                    if let Some(rune_obj) = rune.as_object() {
                                                        Some(RuneDetail {
                                                            id: rune_obj.get("id")
                                                                .and_then(|id| id.as_u64())
                                                                .unwrap_or(0) as u32,
                                                            key: rune_obj.get("key")
                                                                .and_then(|k| k.as_str())
                                                                .unwrap_or("")
                                                                .to_string(),
                                                            icon: rune_obj.get("icon")
                                                                .and_then(|i| i.as_str())
                                                                .unwrap_or("")
                                                                .to_string(),
                                                            name: rune_obj.get("name")
                                                                .and_then(|n| n.as_str())
                                                                .unwrap_or("Unknown Rune")
                                                                .to_string(),
                                                            short_desc: rune_obj.get("shortDesc")
                                                                .and_then(|s| s.as_str())
                                                                .unwrap_or("")
                                                                .to_string(),
                                                            long_desc: rune_obj.get("longDesc")
                                                                .and_then(|l| l.as_str())
                                                                .unwrap_or("")
                                                                .to_string(),
                                                        })
                                                    } else {
                                                        None
                                                    }
                                                })
                                                .collect();
                                            Some(RuneSlot { runes })
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                })
                                .collect()
                        } else {
                            Vec::new()
                        };
                        
                        runes.push(Rune {
                            id: rune_id,
                            name,
                            icon,
                            short_desc,
                            long_desc,
                            slots,
                        });
                    }
                }
            }
        }
        
        println!("[LCU] Successfully fetched {} runes", runes.len());
        Ok(runes)
    }
}
