# WingBot Migration Plan

## Current Stack

- **Frontend**: SvelteKit 2 + Svelte 5, Tailwind 4, Tauri invoke + WebSocket
- **Backend**: Rust/Tauri 2, `LCUAPI`, `LeagueSession`, `WebSocketServer`
- **Platform**: Windows only (LCU detection)

## Architecture

See [docs/LEAGUE_API.md](docs/LEAGUE_API.md).

## Completed Refactor (2025)

- Fixed Live Client API to use **HTTP** on port 2999
- Introduced `GameSnapshot` as canonical game state
- Added `LeagueSession` coordinator
- App-level WebSocket lifecycle in `+layout.svelte`
- Removed AI service dependency and duplicate routes/stores
- Populated `GameManager` champion cache from LCU

## Routes

| Route | Purpose |
|-------|---------|
| `/` | Dashboard |
| `/game` | Session assistant — queue, champ select, live game |
| `/settings` | App configuration |
| `/champion-select`, `/league-launcher`, `/overlays` | Redirect → `/game` or modal via `?open=` |
| `/live-game`, `/live-game-websocket` | Redirect → `/game` |

## Remaining / Future

- macOS/Linux LCU detection (out of scope)
- Riot Spectator API for full spectator stats (requires API key)
- Optional: `tauri-specta` for generated TS types from Rust
