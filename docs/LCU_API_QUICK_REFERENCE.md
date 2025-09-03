# LCU API Quick Reference Card

## Most Used Endpoints

### 🎯 Champion Select
```
GET /lol-champ-select/v1/session
```
**Use**: Get current pick/ban state, timer, team compositions
**When**: During champion select phase
**Data**: Picks, bans, timer, pick order, team info

### 🎮 Live Game
```
GET /liveclientdata/allgamedata
```
**Use**: Get comprehensive live game data
**When**: During active game
**Data**: All players, scores, items, runes, events

### 📊 Player Stats
```
GET /liveclientdata/playerscores?riotId={riotId}
```
**Use**: Get individual player performance
**When**: During game, for specific player analysis
**Data**: KDA, CS, wards, objectives

### 🛡️ Items & Builds
```
GET /liveclientdata/playeritems?riotId={riotId}
```
**Use**: Get player's current items and build
**When**: During game, for build recommendations
**Data**: Items, slots, consumables, trinkets

### ⚡ Summoner Spells
```
GET /liveclientdata/playersummonerspells?riotId={riotId}
```
**Use**: Get player's summoner spells
**When**: During game, for spell tracking
**Data**: Spell names, descriptions, cooldowns

### 🌟 Runes
```
GET /liveclientdata/playermainrunes?riotId={riotId}
```
**Use**: Get player's rune setup
**When**: During game, for rune analysis
**Data**: Keystone, primary/secondary trees

### 🎲 Game Info
```
GET /liveclientdata/gamestats
```
**Use**: Get basic game information
**When**: During game, for game state
**Data**: Mode, time, map, terrain

## Connection Detection

### Windows
```bash
wmic process where "name='LeagueClientUx.exe'" get commandline
```
**Look for**: `--app-port=XXXXX` and `--remoting-auth-token=XXXXX`

### Unix/Linux
```bash
ps aux | grep LeagueClientUx
```
**Look for**: League client process (port 2999 fallback)

## Authentication

### LCU API
- **Username**: `riot`
- **Password**: From `--remoting-auth-token` argument
- **Protocol**: HTTPS
- **Port**: Dynamic (from `--app-port`)

### Live Client Data API
- **Authentication**: None required
- **Protocol**: HTTPS
- **Port**: Fixed (2999)

## Common Response Patterns

### Success Response
```json
{
  "status": "success",
  "data": { ... }
}
```

### Error Response
```json
{
  "status": "error",
  "message": "Error description"
}
```

### Timeout Response
```json
{
  "status": "timeout",
  "message": "Request timed out"
}
```

## Error Handling

### Connection Errors
- **League not running**: Show "League Not Running"
- **Connection failed**: Show "Connection Failed"
- **Authentication failed**: Re-detect client

### API Errors
- **404**: Endpoint not available (wrong game phase)
- **401**: Authentication failed
- **500**: Server error, retry later

### Timeout Errors
- **Champion select**: 3 seconds
- **Live game**: 5 seconds
- **Health check**: 2 seconds

## Data Flow

```
League Client → LCU API → WingBot Backend → AI Service → Frontend
     ↓
Live Client Data API → Real-time Stats → Game State Updates
```

## Performance Tips

- **Cache static data**: Champions, items, runes
- **Batch requests**: Combine multiple API calls
- **Smart polling**: Different frequencies per game phase
- **Connection reuse**: Reuse HTTP clients
- **Timeout management**: Different timeouts per endpoint

## Development Notes

- **Mock data**: Use for development when client not running
- **Error simulation**: Test various failure scenarios
- **Performance monitoring**: Track response times
- **Cross-platform**: Test on different operating systems
