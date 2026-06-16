# WingBot League API Architecture

WingBot integrates with League of Legends on **Windows** through three local API layers.

## API Layers

| Layer | Endpoint | Auth | Active when |
|-------|----------|------|-------------|
| LCU REST | `https://127.0.0.1:{port}` | Basic `riot:{token}` | League client open |
| LCU WebSocket | `wss://127.0.0.1:{port}` | Same as REST; subscribe `[5,"OnJsonApiEvent"]` | League client open |
| Live Client Data | `http://127.0.0.1:2999` | None | In-game (not spectating) |

## Connection Discovery

1. **Primary**: Parse `LeagueClientUx.exe` command line for `--app-port` and `--remoting-auth-token`
2. **Fallback**: Read lockfile (`process:pid:port:password:protocol`) from Riot/League install paths
3. **Manual override**: Settings → League Client → manual port/password

## Backend Coordinator

`LeagueSession` (`src-tauri/src/league_session.rs`) coordinates:

- LCU detection and connection
- LCU WebSocket lifecycle
- `GameSnapshot` building via `LCUAPI::get_game_snapshot()`
- GameManager champion cache population

## GameSnapshot

Canonical state type shared with the frontend (`ComprehensiveGameState` in TypeScript):

- `lcu_connected`, `game_phase`, `live_client_available`, `is_spectator_mode`
- `current_summoner`, `champion_select`, `live_game`, `live_stats`, `player_scores`
- `timestamp`

## Data Flow

```
App start → LeagueSession.initialize()
         → WebSocket server (:8765) in +layout.svelte
         → Monitor loops poll/build GameSnapshot
         → Frontend websocketStore subscribes once
```

## Spectator Mode Limitation

When LCU reports `InProgress` but Live Client on port 2999 is unreachable, WingBot sets `is_spectator_mode: true`. Riot does not expose full live game data to spectators through the Live Client API. Full spectator stats require Riot's server-side Spectator API.

## Key Files

- `src-tauri/src/lcu_api.rs` — HTTP client for LCU + Live Client
- `src-tauri/src/lcu_websocket.rs` — LCU event WebSocket
- `src-tauri/src/league_session.rs` — Session coordinator
- `src-tauri/src/websocket_server.rs` — App WebSocket broadcast to UI
- `src/lib/stores/websocketStore.ts` — Frontend state hub
