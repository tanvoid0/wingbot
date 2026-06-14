use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::fs;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeagueRegion {
    pub name: String,
    pub patchline: String,
    pub client_config_url: String,
    pub chat_host: String,
    pub chat_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeagueInstallation {
    pub path: String,
    pub is_pbe: bool,
    pub is_esports: bool,
}

pub struct LeagueLauncher {
    regions: HashMap<String, LeagueRegion>,
    installations: Vec<LeagueInstallation>,
}

impl LeagueLauncher {
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
            installations: Vec::new(),
        }
    }

    /// Detect League of Legends installations and regions
    pub async fn detect_installations(&mut self) -> Result<Vec<LeagueInstallation>, String> {
        self.installations.clear();
        
        // Get Riot Client installs path
        let riot_installs_path = if cfg!(target_os = "windows") {
            env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".to_string()) + "\\Riot Games\\RiotClientInstalls.json"
        } else if cfg!(target_os = "macos") {
            "/Users/Shared/Riot Games/RiotClientInstalls.json".to_string()
        } else {
            return Err("Unsupported operating system".to_string());
        };

        if !Path::new(&riot_installs_path).exists() {
            return Err("Riot Client not found. Please install Riot Client first.".to_string());
        }

        // Read Riot Client installs
        let installs_content = fs::read_to_string(&riot_installs_path)
            .map_err(|e| format!("Failed to read Riot installs: {}", e))?;
        
        let installs: serde_json::Value = serde_json::from_str(&installs_content)
            .map_err(|e| format!("Failed to parse Riot installs: {}", e))?;

        // Get default Riot Client path
        let _riot_client_path = installs.get("rc_default")
            .and_then(|v| v.as_str())
            .ok_or("Riot Client path not found")?;

        // Detect League installations from metadata
        let metadata_path = if cfg!(target_os = "windows") {
            env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".to_string()) + "\\Riot Games\\Metadata"
        } else if cfg!(target_os = "macos") {
            "/Users/Shared/Riot Games/Metadata".to_string()
        } else {
            return Err("Unsupported operating system".to_string());
        };

        if Path::new(&metadata_path).exists() {
            self.detect_league_from_metadata(&metadata_path)?;
        }

        // Fallback to common installation paths
        if self.installations.is_empty() {
            self.detect_common_installations()?;
        }

        Ok(self.installations.clone())
    }

    fn detect_league_from_metadata(&mut self, metadata_path: &str) -> Result<(), String> {
        let entries = fs::read_dir(metadata_path)
            .map_err(|e| format!("Failed to read metadata directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_dir() {
                let dir_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                if dir_name.starts_with("league_of_legends") {
                    let yaml_path = path.join(format!("{}.product_settings.yaml", dir_name));
                    
                    if yaml_path.exists() {
                        if let Ok(content) = fs::read_to_string(&yaml_path) {
                            if let Ok(yaml_data) = serde_yaml::from_str::<serde_json::Value>(&content) {
                                if let Some(install_path) = yaml_data.get("product_install_full_path")
                                    .and_then(|v| v.as_str()) {
                                    
                                    let is_pbe = dir_name.contains("PBE");
                                    let is_esports = dir_name.contains("ESPORTSTMNT") || dir_name.contains("LOLTMNT");
                                    
                                    self.installations.push(LeagueInstallation {
                                        path: install_path.to_string(),
                                        is_pbe,
                                        is_esports,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn detect_common_installations(&mut self) -> Result<(), String> {
        let common_paths = if cfg!(target_os = "windows") {
            vec![
                "C:\\Riot Games\\League of Legends",
                "C:\\Riot Games\\League of Legends (PBE)",
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                "/Applications/League of Legends.app",
            ]
        } else {
            return Err("Unsupported operating system".to_string());
        };

        for path in common_paths {
            if Path::new(path).exists() {
                let is_pbe = path.contains("PBE");
                let is_esports = false; // Common installations are not esports
                
                self.installations.push(LeagueInstallation {
                    path: path.to_string(),
                    is_pbe,
                    is_esports,
                });
            }
        }

        Ok(())
    }

    /// Load regions from system.yaml files
    pub async fn load_regions(&mut self) -> Result<Vec<LeagueRegion>, String> {
        self.regions.clear();
        
        let installations = self.installations.clone();
        
        for installation in &installations {
            let system_yaml_path = if cfg!(target_os = "windows") {
                format!("{}\\system.yaml", installation.path)
            } else if cfg!(target_os = "macos") {
                format!("{}/Contents/LoL/system.yaml", installation.path)
            } else {
                continue;
            };

            if Path::new(&system_yaml_path).exists() {
                if let Ok(content) = fs::read_to_string(&system_yaml_path) {
                    if let Ok(yaml_data) = serde_yaml::from_str::<serde_json::Value>(&content) {
                        self.parse_regions_from_yaml(&yaml_data)?;
                    }
                }
            }
        }

        // Add default regions if none found
        if self.regions.is_empty() {
            self.add_default_regions();
        }

        Ok(self.regions.values().cloned().collect())
    }

    fn parse_regions_from_yaml(&mut self, yaml_data: &serde_json::Value) -> Result<(), String> {
        if let Some(region_data) = yaml_data.get("region_data") {
            for (region_name, region_info) in region_data.as_object()
                .ok_or("Invalid region_data format")? {
                
                if let Some(servers) = region_info.get("servers") {
                    let client_config_url = servers.get("client_config")
                        .and_then(|c| c.get("client_config_url"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let chat_host = servers.get("chat")
                        .and_then(|c| c.get("chat_host"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let chat_port = servers.get("chat")
                        .and_then(|c| c.get("chat_port"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(5223) as u16;

                    let patchline = if region_name.contains("PBE") {
                        "pbe".to_string()
                    } else {
                        "live".to_string()
                    };

                    self.regions.insert(region_name.clone(), LeagueRegion {
                        name: region_name.clone(),
                        patchline,
                        client_config_url,
                        chat_host,
                        chat_port,
                    });
                }
            }
        }

        Ok(())
    }

    fn add_default_regions(&mut self) {
        let default_regions = vec![
            ("NA", "live"),
            ("EUW", "live"),
            ("EUNE", "live"),
            ("KR", "live"),
            ("BR", "live"),
            ("LAN", "live"),
            ("LAS", "live"),
            ("OCE", "live"),
            ("RU", "live"),
            ("TR", "live"),
            ("JP", "live"),
            ("PBE", "pbe"),
        ];

        for (name, patchline) in default_regions {
            self.regions.insert(name.to_string(), LeagueRegion {
                name: name.to_string(),
                patchline: patchline.to_string(),
                client_config_url: "https://clientconfig.rpg.riotgames.com".to_string(),
                chat_host: format!("{}.chat.si.riotgames.com", name.to_lowercase()),
                chat_port: 5223,
            });
        }
    }

    /// Launch League of Legends with specified region
    pub async fn launch_league(&self, region: &str, custom_args: Option<Vec<String>>) -> Result<(), String> {
        // Get Riot Client path
        let riot_installs_path = if cfg!(target_os = "windows") {
            env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".to_string()) + "\\Riot Games\\RiotClientInstalls.json"
        } else if cfg!(target_os = "macos") {
            "/Users/Shared/Riot Games/RiotClientInstalls.json".to_string()
        } else {
            return Err("Unsupported operating system".to_string());
        };

        let installs_content = fs::read_to_string(&riot_installs_path)
            .map_err(|e| format!("Failed to read Riot installs: {}", e))?;
        
        let installs: serde_json::Value = serde_json::from_str(&installs_content)
            .map_err(|e| format!("Failed to parse Riot installs: {}", e))?;

        let riot_client_path = installs.get("rc_default")
            .and_then(|v| v.as_str())
            .ok_or("Riot Client path not found")?;

        // Build command arguments
        let mut args = Vec::new();
        
        // Add custom arguments if provided
        if let Some(custom) = custom_args {
            for arg in custom {
                if !arg.is_empty() {
                    args.push(format!("--{}", arg));
                }
            }
        }

        // Add region-specific arguments
        let region_info = self.regions.get(region)
            .ok_or_else(|| format!("Region '{}' not found", region))?;

        args.push(format!("--launch-patchline={}", region_info.patchline));
        args.push(format!("--region={}", region));

        // Launch the process
        let mut command = Command::new(riot_client_path);
        command.args(&args);

        println!("[League Launcher] Launching: {} {}", riot_client_path, args.join(" "));

        let child = command.spawn()
            .map_err(|e| format!("Failed to launch League of Legends: {}", e))?;

        // Don't wait for the process to complete
        std::mem::forget(child);

        Ok(())
    }

    /// Get available regions
    pub fn get_regions(&self) -> Vec<LeagueRegion> {
        self.regions.values().cloned().collect()
    }

    /// Get available installations
    pub fn get_installations(&self) -> Vec<LeagueInstallation> {
        self.installations.clone()
    }
}

impl Default for LeagueLauncher {
    fn default() -> Self {
        Self::new()
    }
}
