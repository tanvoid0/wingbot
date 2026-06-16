# WingBot

League of Legends desktop companion built with **Tauri 2**, **SvelteKit**, and **Rust**.

## Features

- League client detection and LCU integration (Windows)
- Live game dashboard with real-time WebSocket updates
- Champion select overlay
- In-game overlay configuration (CS tracker, timers, etc.)
- League client launcher

## Requirements

- Windows 10+
- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) 1.85+ (edition 2024)
- League of Legends client (for live data)

## Development

```bash
pnpm install
pnpm tauri dev
```

## Architecture

See [docs/LEAGUE_API.md](docs/LEAGUE_API.md) for League API integration details.

## Configuration

Settings are stored at `%APPDATA%/WingBot/wingbot_config.json` on Windows.
