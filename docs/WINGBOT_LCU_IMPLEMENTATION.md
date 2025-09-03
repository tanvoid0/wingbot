# WingBot LCU API Implementation Guide

## Current Implementation Status

### ✅ What's Already Implemented
- Basic LCU API structure in `src-tauri/src/lcu_api.rs`
- Windows connection detection via command line parsing
- Basic Tauri commands for LCU integration
- HTTP client setup with authentication

### 🔄 What Needs Enhancement
- Unix/Linux support
- Better error handling and fallbacks
- Integration with Live Client Data API
- Real-time data streaming

## Implementation Plan

### Phase 1: Enhance Current LCU API (Immediate)

#### 1.1 Fix AI Service Status Integration
**File**: `src-tauri/src/lib.rs`
**Issue**: AI service health check not showing in system info
**Solution**: Update `get_system_info` to properly call async health check

```rust
#[tauri::command]
async fn get_system_info() -> Result<Value, String> {
    let league_running = {
        let lcu_api = LCU_API.lock().map_err(|e| e.to_string())?;
        lcu_api.is_league_running()
    };
    
    let ai_service_status = {
        let ai_service = AI_SERVICE.lock().map_err(|e| e.to_string())?;
        ai_service.get_server_status().await
    };
    
    let info = serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "platform": std::env::consts::FAMILY,
        "league_running": league_running,
        "ai_service_status": ai_service_status
    });
    
    Ok(info)
}
```

#### 1.2 Add Unix/Linux Support
**File**: `src-tauri/src/lcu_api.rs`
**Current**: Only Windows detection implemented
**TODO**: Implement Unix/Linux process detection

```rust
fn detect_unix_connection(&self) -> Result<LCUConnection, String> {
    // Check for League client process on Unix systems
    let output = Command::new("ps")
        .args(&["aux"])
        .output()
        .map_err(|e| format!("Failed to execute ps: {}", e))?;
    
    let processes = String::from_utf8_lossy(&output.stdout);
    
    // Look for League client process
    if processes.contains("LeagueClientUx") {
        // Extract connection info from process
        // This will need platform-specific implementation
        Ok(LCUConnection {
            port: 2999, // Default fallback
            password: "".to_string(), // Will need different auth method
            protocol: "https".to_string(),
        })
    } else {
        Err("League client not found".to_string())
    }
}
```

### Phase 2: Live Client Data API Integration

#### 2.1 Add Live Client Data Endpoints
**File**: `src-tauri/src/lcu_api.rs`
**Purpose**: Access real-time in-game data

```rust
impl LCUAPI {
    // ... existing methods ...
    
    pub async fn get_live_game_stats(&self) -> Result<serde_json::Value, String> {
        let client = reqwest::Client::new();
        let url = "https://127.0.0.1:2999/liveclientdata/gamestats";
        
        let response = timeout(
            Duration::from_secs(5),
            client.get(url).send()
        ).await
        .map_err(|_| "Request timeout".to_string())?
        .map_err(|e| format!("Request failed: {}", e))?;
        
        if response.status().is_success() {
            let stats: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(stats)
        } else {
            Err(format!("Failed to get game stats: {}", response.status()))
        }
    }
    
    pub async fn get_player_scores(&self, riot_id: &str) -> Result<serde_json::Value, String> {
        let client = reqwest::Client::new();
        let url = format!("https://127.0.0.1:2999/liveclientdata/playerscores?riotId={}", riot_id);
        
        let response = timeout(
            Duration::from_secs(5),
            client.get(&url).send()
        ).await
        .map_err(|_| "Request timeout".to_string())?
        .map_err(|e| format!("Request failed: {}", e))?;
        
        if response.status().is_success() {
            let scores: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(scores)
        } else {
            Err(format!("Failed to get player scores: {}", response.status()))
        }
    }
    
    pub async fn get_all_game_data(&self) -> Result<serde_json::Value, String> {
        let client = reqwest::Client::new();
        let url = "https://127.0.0.1:2999/liveclientdata/allgamedata";
        
        let response = timeout(
            Duration::from_secs(5),
            client.get(url).send()
        ).await
        .map_err(|_| "Request timeout".to_string())?
        .map_err(|e| format!("Request failed: {}", e))?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(data)
        } else {
            Err(format!("Failed to get all game data: {}", response.status()))
        }
    }
}
```

#### 2.2 Add New Tauri Commands
**File**: `src-tauri/src/lib.rs`
**Purpose**: Expose new endpoints to frontend

```rust
// Live game data commands
#[tauri::command]
async fn get_live_game_stats() -> Result<Value, String> {
    let lcu_api = LCU_API.lock().map_err(|e| e.to_string())?;
    let stats = lcu_api.get_live_game_stats().await?;
    Ok(stats)
}

#[tauri::command]
async fn get_player_scores(riot_id: String) -> Result<Value, String> {
    let lcu_api = LCU_API.lock().map_err(|e| e.to_string())?;
    let scores = lcu_api.get_player_scores(&riot_id).await?;
    Ok(scores)
}

#[tauri::command]
async fn get_all_game_data() -> Result<Value, String> {
    let lcu_api = LCU_API.lock().map_err(|e| e.to_string())?;
    let data = lcu_api.get_all_game_data().await?;
    Ok(data)
}
```

### Phase 3: Enhanced Game State Management

#### 3.1 Update Game Manager
**File**: `src-tauri/src/game.rs`
**Purpose**: Integrate real LCU data instead of mock data

```rust
impl GameManager {
    // ... existing methods ...
    
    pub async fn update_from_lcu(&mut self, lcu_api: &LCUAPI) -> Result<(), String> {
        // Try to get champion select state
        if let Ok(champ_select) = lcu_api.get_champion_select_state().await {
            self.current_state.status = GameStatus::InChampionSelect;
            // Parse champion select data and update state
            // This will need implementation based on actual API response
        }
        
        // Try to get live game data
        if let Ok(game_data) = lcu_api.get_all_game_data().await {
            self.current_state.status = GameStatus::InGame;
            // Parse live game data and update state
            // This will need implementation based on actual API response
        }
        
        Ok(())
    }
    
    pub async fn get_real_champion_suggestions(
        &self,
        enemy_team: &[Player],
        ally_team: &[Player],
        role: &Role,
    ) -> Result<Vec<ChampionSuggestion>, String> {
        // TODO: Replace mock data with real AI service call
        // For now, return mock data but mark as TODO
        Ok(self.get_champion_suggestions(enemy_team, ally_team, role))
    }
}
```

### Phase 4: Frontend Integration

#### 4.1 Create Missing Route Pages
**Files**: 
- `src/routes/champion-select/+page.svelte`
- `src/routes/live-game/+page.svelte`
- `src/routes/ai-chat/+page.svelte`
- `src/routes/settings/+page.svelte`

#### 4.2 Update Dashboard with Real Data
**File**: `src/routes/+page.svelte`
**Purpose**: Show real LCU connection status and game data

```svelte
<script lang="ts">
  // ... existing imports ...
  
  async function loadData() {
    try {
      isLoading = true;
      error = null;
      
      // Load system info (includes LCU and AI service status)
      const systemInfoResult = await invoke('get_system_info');
      systemInfo = systemInfoResult;
      
      // Load game state
      const gameStateResult = await invoke('get_game_state');
      gameState = gameStateResult;
      
      // If in game, load live game data
      if (gameState?.status === 'InGame') {
        const liveGameData = await invoke('get_all_game_data');
        // Process and display live game data
      }
      
    } catch (e) {
      error = e as string;
    } finally {
      isLoading = false;
    }
  }
</script>
```

## Testing Strategy

### 1. Local Development Testing
- **League Client**: Test with actual running League client
- **Mock Responses**: Use sample API responses for development
- **Error Scenarios**: Test connection failures, timeouts, etc.

### 2. Integration Testing
- **End-to-End**: Test full data flow from LCU → Backend → Frontend
- **Performance**: Measure API response times and frontend rendering
- **Error Handling**: Verify graceful degradation on failures

### 3. Cross-Platform Testing
- **Windows**: Primary development platform
- **Unix/Linux**: Test when Unix support is implemented
- **macOS**: Future consideration

## Error Handling Improvements

### 1. Connection Failures
```rust
pub async fn get_connection_status(&self) -> String {
    if self.is_league_running() {
        match self.detect_connection() {
            Ok(_) => "Connected".to_string(),
            Err(_) => "Connection Failed".to_string(),
        }
    } else {
        "League Not Running".to_string()
    }
}
```

### 2. API Timeouts
```rust
// Use different timeouts for different operations
const CHAMP_SELECT_TIMEOUT: Duration = Duration::from_secs(3);
const LIVE_GAME_TIMEOUT: Duration = Duration::from_secs(5);
const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(2);
```

### 3. Graceful Fallbacks
```rust
// If LCU API fails, fall back to Live Client Data API
// If both fail, use cached data or show offline status
```

## Performance Considerations

### 1. Request Batching
- Combine multiple API calls where possible
- Cache static data (champion info, items, runes)
- Implement smart polling based on game phase

### 2. Memory Management
- Avoid holding large amounts of data in memory
- Implement data cleanup for completed games
- Use efficient data structures for real-time updates

### 3. Network Optimization
- Reuse HTTP clients
- Implement connection pooling
- Use appropriate timeouts for different operations

## Next Steps Priority

1. **Fix AI Service Status** - Get real connection status showing
2. **Implement Live Client Data API** - Add real-time game data endpoints
3. **Create Missing Route Pages** - Add champion-select, live-game, ai-chat, settings
4. **Replace Mock Data** - Integrate real LCU data into game state
5. **Add Unix/Linux Support** - Cross-platform compatibility
6. **Error Handling** - Robust error handling and user feedback
7. **Performance Optimization** - Caching and request optimization

## Success Metrics

- [ ] AI service status shows real connection status
- [ ] Live game data loads and displays correctly
- [ ] Champion select data updates in real-time
- [ ] All navigation routes work properly
- [ ] Error states are handled gracefully
- [ ] Performance is acceptable (< 100ms for most operations)
- [ ] Cross-platform compatibility achieved
