# League Client Update (LCU) API & Live Client Data API Reference

## Overview
This document provides a comprehensive reference for integrating with League of Legends client APIs for the WingBot project. It covers both the League Client Update (LCU) API and the Live Client Data API.

## API Categories

### 1. League Client Update (LCU) API
- **Purpose**: Access to client-side data and game state
- **Base URL**: `https://127.0.0.1:{port}` (where port is extracted from client process)
- **Authentication**: Basic auth with username "riot" and password from `--remoting-auth-token`
- **Use Case**: Champion select, game flow, session management

### 2. Live Client Data API
- **Purpose**: Real-time in-game data and statistics
- **Base URL**: `https://127.0.0.1:2999`
- **Authentication**: None required (local endpoint)
- **Use Case**: Live game data, player stats, events, items

## LCU API Endpoints

### Champion Select
```
GET /lol-champ-select/v1/session
```
**Purpose**: Get current champion select state
**Response**: Champion select session data including picks, bans, timer, etc.

### Game Flow
```
GET /lol-gameflow/v1/session
```
**Purpose**: Get current game flow state
**Response**: Game session information, phase, etc.

### Summoner Information
```
GET /lol-summoner/v1/current-summoner
```
**Purpose**: Get current player's summoner information
**Response**: Summoner ID, name, level, profile icon, etc.

### Match History
```
GET /lol-match-history/v1/products/lol/{puuid}/matches
```
**Purpose**: Get player's recent match history
**Response**: List of recent games with basic stats

## Live Client Data API Endpoints

### Player Information
```
GET /liveclientdata/playerscores?riotId={riotId}
```
**Purpose**: Get current player scores (KDA, CS, wards)
**Response**: 
```json
{
    "assists": 0,
    "creepScore": 0,
    "deaths": 0,
    "kills": 0,
    "wardScore": 0.0
}
```

### Summoner Spells
```
GET /liveclientdata/playersummonerspells?riotId={riotId}
```
**Purpose**: Get player's current summoner spells
**Response**:
```json
{
    "summonerSpellOne": {
        "displayName": "Flash",
        "rawDescription": "GeneratedTip_SummonerSpell_SummonerFlash_Description",
        "rawDisplayName": "GeneratedTip_SummonerSpell_SummonerFlash_DisplayName"
    },
    "summonerSpellTwo": {
        "displayName": "Ignite",
        "rawDescription": "GeneratedTip_SummonerSpell_SummonerDot_Description",
        "rawDisplayName": "GeneratedTip_SummonerSpell_SummonerDot_DisplayName"
    }
}
```

### Runes
```
GET /liveclientdata/playermainrunes?riotId={riotId}
```
**Purpose**: Get player's current rune setup
**Response**:
```json
{
    "keystone": {
        "displayName": "Electrocute",
        "id": 8112,
        "rawDescription": "perk_tooltip_Electrocute",
        "rawDisplayName": "perk_displayname_Electrocute"
    },
    "primaryRuneTree": {
        "displayName": "Domination",
        "id": 8100,
        "rawDescription": "perkstyle_tooltip_7200",
        "rawDisplayName": "perkstyle_displayname_7200"
    },
    "secondaryRuneTree": {
        "displayName": "Sorcery",
        "id": 8200,
        "rawDescription": "perkstyle_tooltip_7202",
        "rawDisplayName": "perkstyle_displayname_7202"
    }
}
```

### Items
```
GET /liveclientdata/playeritems?riotId={riotId}
```
**Purpose**: Get player's current items
**Response**:
```json
[
    {
        "canUse": true,
        "consumable": false,
        "count": 1,
        "displayName": "Warding Totem (Trinket)",
        "itemID": 3340,
        "price": 0,
        "rawDescription": "game_item_description_3340",
        "rawDisplayName": "game_item_displayname_3340",
        "slot": 6
    }
]
```

### Game Statistics
```
GET /liveclientdata/gamestats
```
**Purpose**: Get basic game information
**Response**:
```json
{
    "gameMode": "CLASSIC",
    "gameTime": 0.000000000,
    "mapName": "Map11",
    "mapNumber": 11,
    "mapTerrain": "Default"
}
```

### Events
```
GET /liveclientdata/eventdata
```
**Purpose**: Get list of game events
**Response**:
```json
{
    "Events": [
        {
            "EventID": 0,
            "EventName": "GameStart",
            "EventTime": 0.0325561985373497
        }
    ]
}
```

### All Players
```
GET /liveclientdata/allgamedata
```
**Purpose**: Get comprehensive game data for all players
**Response**: Complete game state including all players, events, and statistics

## Implementation Strategy for WingBot

### 1. Connection Detection
- **Windows**: Parse `LeagueClientUx.exe` command line for `--app-port` and `--remoting-auth-token`
- **Unix/Linux**: TODO - Implement process detection and argument parsing
- **Fallback**: Use default port 2999 for Live Client Data API

### 2. Data Flow Architecture
```
Game State Detection → LCU API → Champion Select Data
                    ↓
                Live Game → Live Client Data API → Real-time Stats
                    ↓
                AI Service → Recommendations → Frontend Display
```

### 3. Key Data Points for AI Recommendations

#### Champion Select Phase
- Enemy team composition
- Ally team composition
- Available champions
- Current bans
- Pick order and timer

#### Live Game Phase
- Player performance (KDA, CS, items)
- Team composition analysis
- Objective control
- Vision control (wards)
- Build paths

### 4. Error Handling Strategy
- **Connection Loss**: Graceful fallback to cached data
- **API Rate Limits**: Implement request throttling
- **Authentication Failures**: Re-detect client connection
- **Data Parsing Errors**: Log errors, use fallback values

## Security Considerations

### Riot Games Policy Compliance
- **No Competitive Advantage**: Don't use data that gives unfair advantages
- **Game Integrity**: Don't alter game mechanics or objectives
- **Player Privacy**: Respect hidden player settings
- **API Key Security**: Never expose API keys in client code

### Local Security
- **HTTPS Only**: Use HTTPS for LCU API calls
- **Local Network**: All APIs are localhost only
- **Process Validation**: Verify League client process authenticity
- **Data Sanitization**: Validate all API responses

## Performance Optimization

### Caching Strategy
- **Static Data**: Champion stats, item data, rune information
- **Session Data**: Game state, player information
- **Real-time Data**: Live stats, events (minimal caching)

### Request Optimization
- **Batch Requests**: Combine multiple API calls where possible
- **Polling Frequency**: Adjust based on game phase
- **Connection Pooling**: Reuse HTTP clients
- **Timeout Management**: Different timeouts for different endpoints

## Testing and Development

### Development Environment
- **Mock Data**: Use sample responses for development
- **Local Testing**: Test with actual League client
- **Error Simulation**: Test various failure scenarios
- **Performance Testing**: Measure API response times

### Debugging Tools
- **API Logging**: Log all requests and responses
- **Error Tracking**: Capture and categorize errors
- **Performance Monitoring**: Track response times and success rates
- **Connection Status**: Monitor client connection health

## Future Enhancements

### Planned Features
- **Real-time Updates**: WebSocket-like updates for live data
- **Advanced Analytics**: Player performance tracking over time
- **Predictive AI**: Machine learning for better recommendations
- **Cross-platform Support**: Full Unix/Linux compatibility

### API Extensions
- **Custom Endpoints**: Additional data processing endpoints
- **Data Aggregation**: Combine multiple API sources
- **Historical Analysis**: Match history and trend analysis
- **Social Features**: Team coordination and communication

## References

- [Riot Games Developer Portal](https://developer.riotgames.com/)
- [Live Client Data API Documentation](https://developer.riotgames.com/docs/lol#game-client-api_live-client-data-api)
- [League Client API Documentation](https://developer.riotgames.com/docs/lol#game-client-api_league-client-api)
- [Riot Games API Policy](https://developer.riotgames.com/docs/lol#developer-api-policy)

## Notes

- **Riot ID Transition**: As of November 2023, Riot is transitioning from Summoner Names to Riot IDs
- **API Deprecation**: Some endpoints may be deprecated in future versions
- **Rate Limiting**: Implement proper rate limiting to avoid API restrictions
- **Error Handling**: Always implement graceful fallbacks for API failures
