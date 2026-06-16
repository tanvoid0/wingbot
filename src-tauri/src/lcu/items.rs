use crate::lcu_api::{Item, ItemImage, LCUAPI, LCU_BASE_URL, LCU_ITEMS_ENDPOINT};

impl LCUAPI {
    // Get items as structured data
    pub async fn get_items_structured(&self) -> Result<Vec<Item>, String> {
        println!("[LCU] Fetching items data...");
        
        let connection = self.connection.as_ref()
            .ok_or("No LCU connection established")?;
        let url = format!("{}:{}{}", LCU_BASE_URL, connection.port, LCU_ITEMS_ENDPOINT);
        let response = self.make_request(&url, "GET", None).await?;
        
        if !response.status().is_success() {
            return Err(format!("Failed to fetch items: HTTP {}", response.status()));
        }
        
        let items_json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse items JSON: {}", e))?;
        
        let mut items = Vec::new();
        
        if let Some(items_data) = items_json.as_object() {
            for (item_id_str, item_data) in items_data {
                if let Ok(item_id) = item_id_str.parse::<u64>() {
                    if let Some(item_obj) = item_data.as_object() {
                        let name = item_obj.get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        
                        let description = item_obj.get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("No description")
                            .to_string();
                        
                        let plaintext = item_obj.get("plaintext")
                            .and_then(|p| p.as_str())
                            .unwrap_or("")
                            .to_string();
                        
                        let gold = item_obj.get("gold")
                            .and_then(|g| g.as_object())
                            .map(|gold_obj| {
                                gold_obj.get("base")
                                    .and_then(|b| b.as_u64())
                                    .unwrap_or(0)
                            })
                            .unwrap_or(0);
                        
                        let tags = item_obj.get("tags")
                            .and_then(|t| t.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| s.to_string())
                                .collect())
                            .unwrap_or_default();
                        
                        let stats = item_obj.get("stats")
                            .and_then(|s| s.as_object())
                            .map(|stats_obj| {
                                let mut stats_map = std::collections::HashMap::new();
                                for (key, value) in stats_obj {
                                    if let Some(num) = value.as_f64() {
                                        stats_map.insert(key.clone(), num);
                                    }
                                }
                                stats_map
                            })
                            .unwrap_or_default();
                        
                        let image = if let Some(img_data) = item_obj.get("image").and_then(|i| i.as_object()) {
                            ItemImage {
                                full: img_data.get("full").and_then(|f| f.as_str()).unwrap_or("").to_string(),
                                sprite: img_data.get("sprite").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                                group: img_data.get("group").and_then(|g| g.as_str()).unwrap_or("").to_string(),
                                x: img_data.get("x").and_then(|x| x.as_u64()).unwrap_or(0) as u32,
                                y: img_data.get("y").and_then(|y| y.as_u64()).unwrap_or(0) as u32,
                                w: img_data.get("w").and_then(|w| w.as_u64()).unwrap_or(0) as u32,
                                h: img_data.get("h").and_then(|h| h.as_u64()).unwrap_or(0) as u32,
                            }
                        } else {
                            ItemImage {
                                full: "".to_string(),
                                sprite: "".to_string(),
                                group: "".to_string(),
                                x: 0, y: 0, w: 0, h: 0,
                            }
                        };
                        
                        items.push(Item {
                            id: item_id,
                            name,
                            description,
                            plaintext,
                            gold,
                            tags,
                            stats,
                            image,
                        });
                    }
                }
            }
        }
        
        println!("[LCU] Successfully fetched {} items", items.len());
        Ok(items)
    }
}
