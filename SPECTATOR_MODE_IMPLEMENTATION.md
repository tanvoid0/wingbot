# Spectator Mode Fix & LCU WebSocket Implementation

## Summary

Successfully implemented fixes for spectator mode detection and added LCU WebSocket support based on analysis of reference projects (mobahinted, lol-pick-ban-ui, LeagueClientDebugger).

## Critical Fixes Implemented

### 1. Protocol Fix (IMMEDIATE FIX - CRITICAL)
**Issue**: App was using `https://127.0.0.1:2999` but Live Client Data API uses HTTP, not HTTPS.

**Files Modified**:
- `src-tauri/src/lcu_api.rs`
- `src-tauri/src/lcu_api_new.rs`

**Status**: Applied in `src-tauri/src/lcu_api.rs` — `LIVE_CLIENT_BASE_URL` is `http://127.0.0.1:2999`.

**Impact**: This fixes live game data retrieval when actively playing. The API doesn't use SSL/TLS.

### 2. Enhanced Spectator Mode Detection
**Files Modified**:
- `src-tauri/src/lcu_api.rs`
- `src-tauri/src/lcu_api_new.rs`

**Improvements**:
- Added Method 0: Check game flow phase for "Spectator", "WatchingGame", or "Spectate" keywords
- Enhanced logging for better debugging
- Multiple fallback detection methods
- More robust phase checking in game flow session

**New Detection Logic**:
1. Check game flow phase directly for spectator-specific phases
2. Check if in game but Live Client Data API unavailable
3. Check for spectator-specific errors in game data
4. Check if can get game stats but not player data
5. Check if in game but can't access player-specific endpoints

### 3. Improved UI Error Messaging
**File Modified**: `src/lib/components/live-game/NoLiveGameMessage.svelte`

**Changes**:
- Clear explanation that spectator mode limitation is from Riot Games API, not a bug
- Explains that apps like Mobalytics use different APIs (server-side Spectator API)
- Provides context about when live data will work (when actively playing)

### 4. LCU WebSocket Implementation
**New Files Created**:
- `src-tauri/src/lcu_websocket.rs` - Complete LCU WebSocket client implementation

**Files Modified**:
- `src-tauri/Cargo.toml` - Added `native-tls` dependency and enabled `native-tls` feature for `tokio-tungstenite`
- `src-tauri/src/lib.rs` - Added module, global instance, and Tauri commands
- `src/lib/stores/websocketStore.ts` - Added LCU WebSocket integration

**Features**:
- Connects to LCU WebSocket (wss://127.0.0.1:{port})
- Uses Basic authentication with riot:{token}
- Subscribes to OnJsonApiEvent for real-time updates
- TLS with disabled certificate verification (required for LCU)
- Automatic event broadcasting to frontend
- Proper connection state management

**New Tauri Commands**:
```rust
- connect_lcu_websocket() -> Result<(), String>
- disconnect_lcu_websocket() -> Result<(), String>
- is_lcu_websocket_connected() -> Result<bool, String>
```

**New TypeScript Functions**:
```typescript
- connectLcuWebSocket()
- disconnectLcuWebSocket()
- isLcuWebSocketConnected()
```

### 5. WebSocket Store Enhancements
**File Modified**: `src/lib/stores/websocketStore.ts`

**Improvements**:
- Auto-connects LCU WebSocket when LCU connection is established
- Better spectator mode error messages
- Integrated LCU WebSocket functions into wsActions

## Why Spectator Mode Shows No Data

### The Fundamental Limitation
The Live Client Data API (`http://127.0.0.1:2999`) **DOES NOT RUN** when spectating. It only runs when you're actively in a game as a player. This is a fundamental limitation of Riot's API design.

### How Mobalytics Works Differently
Mobalytics and similar apps use:
1. **Riot Games Spectator API** - Server-side API (not local client)
2. **LCU WebSocket** - For game state detection and events
3. **Riot's servers** - Fetch spectator data from Riot's infrastructure, not local client

This requires:
- Official Riot API key (rate-limited)
- Different architecture (server-based, not client-based)
- Access to Riot's backend spectator endpoints

## Expected Behavior After Fix

### When Playing a Game
✅ Live Client Data API works on HTTP protocol
✅ Real-time game data retrieval
✅ All stats, items, events available
✅ Full functionality

### When Spectating
✅ Clear error message explaining API limitation
✅ No confusing "connection unhealthy" errors
✅ User understands it's Riot's limitation, not a bug
✅ LCU WebSocket can still detect game state changes
⚠️ Live game data unavailable (API limitation)

### LCU WebSocket Benefits
✅ Real-time game state updates
✅ Can detect spectator mode more reliably
✅ Works in both playing and spectating scenarios
✅ Better connection state management
✅ Foundation for future features (champion select events, lobby events, etc.)

## Testing the Changes

1. **Test Live Game (Playing)**:
   - Start a game as a player
   - App should now connect to Live Client Data API successfully (HTTP fix)
   - All live data should display correctly

2. **Test Spectator Mode**:
   - Spectate a game
   - App should detect spectator mode
   - Should show clear error message explaining limitation
   - LCU WebSocket should still connect and detect game state

3. **Test LCU WebSocket**:
   - Check browser console for WebSocket connection logs
   - Should see: `[LCU WebSocket] Connected successfully!`
   - Should see event logs: `[LCU WebSocket] Event: {type} {uri}`

## Future Enhancements

To support full spectator mode like Mobalytics:
1. Implement Riot Spectator API (requires Riot API key application)
2. Add server-side component to fetch spectator data
3. Use LCU WebSocket events to trigger data fetches
4. Cache and serve spectator game data through your own backend

## Files Modified Summary

### Rust Backend
- `src-tauri/Cargo.toml` - Dependencies
- `src-tauri/src/lcu_api.rs` - Protocol fix + spectator detection
- `src-tauri/src/lcu_api_new.rs` - Protocol fix + spectator detection
- `src-tauri/src/lcu_websocket.rs` - NEW: LCU WebSocket client
- `src-tauri/src/lib.rs` - Module registration + commands

### Frontend
- `src/lib/components/live-game/NoLiveGameMessage.svelte` - Better error messages
- `src/lib/stores/websocketStore.ts` - LCU WebSocket integration + improved spectator handling

## Key Learnings from Reference Projects

### LeagueClientDebugger
- Provided LCU WebSocket implementation pattern
- Showed proper SSL/TLS handling with certificate bypass
- Demonstrated event subscription format `[5, "OnJsonApiEvent"]`

### lol-pick-ban-ui
- Uses `league-connect` library for connection detection
- Focuses on champion select, not live game data
- Good pattern for connection state management

### mobahinted
- Uses Cassiopeia library for post-game data
- Accesses Riot API, not local client
- Different use case (match history analysis)

### None of the reference projects
- Actually implement live spectator mode data retrieval
- This confirms it requires Riot Spectator API (server-side)

