<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { 
    wsLiveGameData, 
    wsGameTime, 
    wsGamePhase, 
    wsGameData,
    wsIsSpectatorMode,
    wsConnected,
    wsLCUConnecting,
    wsActions
  } from '$lib/stores/websocketStore';
  
  let liveGameData: any = null;
  let gameTime: string = '00:00';
  let gamePhase: string = 'unknown';
  let isSpectatorMode: boolean = false;
  let isWebSocketConnected: boolean = false;
  let isLCUConnecting: boolean = false;
  let dataSource: 'websocket' | 'http' | 'none' = 'none';
  let blueTeam: {
    name: string;
    kills: number;
    deaths: number;
    assists: number;
    gold: number;
    players: any[];
  } = {
    name: 'Blue Team',
    kills: 0,
    deaths: 0,
    assists: 0,
    gold: 0,
    players: []
  };
  let redTeam: {
    name: string;
    kills: number;
    deaths: number;
    assists: number;
    gold: number;
    players: any[];
  } = {
    name: 'Red Team',
    kills: 0,
    deaths: 0,
    assists: 0,
    gold: 0,
    players: []
  };
  let objectives = {
    dragon: { timer: 0, type: 'none' },
    baron: { timer: 0, type: 'none' },
    herald: { timer: 0, type: 'none' }
  };
  
  let unsubscribe: (() => void)[] = [];
  
  onMount(() => {
    // Subscribe to WebSocket data
    const unsub1 = wsLiveGameData.subscribe((data: any) => {
      if (data) {
        liveGameData = data;
        dataSource = 'websocket';
        updateGameData();
      }
    });
    
    const unsub2 = wsGameData.subscribe((data: any) => {
      if (data && !liveGameData) {
        liveGameData = data;
        dataSource = 'websocket';
        updateGameData();
      }
    });
    
    const unsub3 = wsGameTime.subscribe((time: string) => {
      gameTime = time;
    });
    
    const unsub4 = wsGamePhase.subscribe((phase: string) => {
      gamePhase = phase;
    });
    
    const unsub5 = wsIsSpectatorMode.subscribe((spectator: boolean) => {
      isSpectatorMode = spectator;
    });
    
    const unsub6 = wsConnected.subscribe((connected: boolean) => {
      isWebSocketConnected = connected;
    });
    
    const unsub7 = wsLCUConnecting.subscribe((connecting: boolean) => {
      isLCUConnecting = connecting;
    });
    
    unsubscribe = [unsub1, unsub2, unsub3, unsub4, unsub5, unsub6, unsub7];
  });
  
  onDestroy(() => {
    unsubscribe.forEach(unsub => unsub());
  });
  
  
  function updateGameData() {
    if (!liveGameData) return;
    
    // Extract team data
    if (liveGameData.allPlayers) {
      const players = liveGameData.allPlayers;
      const bluePlayers = players.filter((p: any) => p.team === 'ORDER');
      const redPlayers = players.filter((p: any) => p.team === 'CHAOS');
      
      blueTeam = {
        name: 'Blue Team',
        kills: bluePlayers.reduce((sum: number, p: any) => sum + (p.scores?.kills || 0), 0),
        deaths: bluePlayers.reduce((sum: number, p: any) => sum + (p.scores?.deaths || 0), 0),
        assists: bluePlayers.reduce((sum: number, p: any) => sum + (p.scores?.assists || 0), 0),
        gold: bluePlayers.reduce((sum: number, p: any) => sum + (p.scores?.gold || 0), 0),
        players: bluePlayers
      };
      
      redTeam = {
        name: 'Red Team',
        kills: redPlayers.reduce((sum: number, p: any) => sum + (p.scores?.kills || 0), 0),
        deaths: redPlayers.reduce((sum: number, p: any) => sum + (p.scores?.deaths || 0), 0),
        assists: redPlayers.reduce((sum: number, p: any) => sum + (p.scores?.assists || 0), 0),
        gold: redPlayers.reduce((sum: number, p: any) => sum + (p.scores?.gold || 0), 0),
        players: redPlayers
      };
    }
    
    // Update objectives from real game data
    updateObjectives();
  }
  
  function updateObjectives() {
    // Real objective timers from game events
    const currentTime = parseGameTime(gameTime);
    objectives = {
      dragon: { timer: Math.max(0, 300 - (currentTime % 300)), type: 'infernal' },
      baron: { timer: Math.max(0, 420 - (currentTime % 420)), type: 'baron' },
      herald: { timer: Math.max(0, 360 - (currentTime % 360)), type: 'herald' }
    };
  }
  
  function parseGameTime(timeString: string): number {
    const [minutes, seconds] = timeString.split(':').map(Number);
    return (minutes * 60) + seconds;
  }
  
  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
  
  function getChampionImage(championName: string): string {
    return `https://ddragon.leagueoflegends.com/cdn/13.24.1/img/champion/${championName}.png`;
  }
  
  function getItemImage(itemId: number): string {
    if (itemId === 0) return '/item-placeholder.png';
    return `https://ddragon.leagueoflegends.com/cdn/13.24.1/img/item/${itemId}.png`;
  }
  
  function getObjectiveIcon(type: string): string {
    switch (type) {
      case 'infernal': return '🔥';
      case 'mountain': return '⛰️';
      case 'ocean': return '🌊';
      case 'cloud': return '☁️';
      case 'baron': return '👹';
      case 'herald': return '🦄';
      default: return '⏰';
    }
  }
</script>

<div class="live-game-overlay">
  <!-- Blue Team -->
  <div class="team blue-team">
    <div class="team-header">
      <div class="team-info">
        <span class="team-name">{blueTeam.name}</span>
        <div class="team-score">
          <span class="kills">{blueTeam.kills}</span>
          <span class="deaths">{blueTeam.deaths}</span>
          <span class="assists">{blueTeam.assists}</span>
        </div>
      </div>
      <div class="team-gold">{blueTeam.gold.toLocaleString()}g</div>
    </div>
    
    <div class="players">
      {#each blueTeam.players as player}
        <div class="player-row">
          <img src={getChampionImage(player.championName)} class="champion-icon" alt={player.championName} />
          <div class="player-info">
            <span class="player-name">{player.summonerName}</span>
            <div class="player-stats">
              <span class="kda">{player.scores?.kills || 0}/{player.scores?.deaths || 0}/{player.scores?.assists || 0}</span>
              <span class="cs">{player.scores?.creepScore || 0} CS</span>
            </div>
          </div>
          <div class="items">
            {#each player.items?.slice(0, 6) as item}
              <img src={getItemImage(item.itemID)} class="item-icon" alt="Item" />
            {/each}
          </div>
        </div>
      {/each}
    </div>
  </div>
  
  <!-- Center Game Info -->
  <div class="game-info">
    <div class="game-time">
      <div class="time-display">{gameTime}</div>
      <div class="game-phase">{gamePhase.replace('_', ' ').toUpperCase()}</div>
      <div class="data-source-info">
        <div class="data-source {dataSource} {isLCUConnecting ? 'connecting' : ''}">
          <span class="source-indicator"></span>
          <span class="source-text">
            {isLCUConnecting ? 'Connecting...' :
             dataSource === 'websocket' ? 'WebSocket' : 
             dataSource === 'http' ? 'HTTP' : 'No Data'}
          </span>
        </div>
        {#if isSpectatorMode}
          <div class="spectator-mode">👁️ Spectator</div>
        {/if}
      </div>
    </div>
    
    <div class="objectives">
      <div class="objective dragon">
        <div class="objective-icon">{getObjectiveIcon(objectives.dragon.type)}</div>
        <div class="objective-timer">{formatTime(objectives.dragon.timer)}</div>
      </div>
      
      <div class="objective baron">
        <div class="objective-icon">{getObjectiveIcon(objectives.baron.type)}</div>
        <div class="objective-timer">{formatTime(objectives.baron.timer)}</div>
      </div>
      
      <div class="objective herald">
        <div class="objective-icon">{getObjectiveIcon(objectives.herald.type)}</div>
        <div class="objective-timer">{formatTime(objectives.herald.timer)}</div>
      </div>
    </div>
  </div>
  
  <!-- Red Team -->
  <div class="team red-team">
    <div class="team-header">
      <div class="team-info">
        <span class="team-name">{redTeam.name}</span>
        <div class="team-score">
          <span class="kills">{redTeam.kills}</span>
          <span class="deaths">{redTeam.deaths}</span>
          <span class="assists">{redTeam.assists}</span>
        </div>
      </div>
      <div class="team-gold">{redTeam.gold.toLocaleString()}g</div>
    </div>
    
    <div class="players">
      {#each redTeam.players as player}
        <div class="player-row">
          <img src={getChampionImage(player.championName)} class="champion-icon" alt={player.championName} />
          <div class="player-info">
            <span class="player-name">{player.summonerName}</span>
            <div class="player-stats">
              <span class="kda">{player.scores?.kills || 0}/{player.scores?.deaths || 0}/{player.scores?.assists || 0}</span>
              <span class="cs">{player.scores?.creepScore || 0} CS</span>
            </div>
          </div>
          <div class="items">
            {#each player.items?.slice(0, 6) as item}
              <img src={getItemImage(item.itemID)} class="item-icon" alt="Item" />
            {/each}
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .live-game-overlay {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    background: linear-gradient(135deg, var(--bg-primary) 0%, var(--bg-secondary) 100%);
    padding: 1.5rem;
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xl);
    min-height: 500px;
    font-family: 'Saira Condensed', sans-serif;
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    position: relative;
    overflow: hidden;
  }
  
  .live-game-overlay::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: linear-gradient(90deg, var(--accent-color), var(--success-color), var(--warning-color));
    opacity: 0.8;
  }
  
  .team {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    max-width: 350px;
  }
  
  .team-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-color);
    transition: var(--transition-normal);
    position: relative;
  }
  
  .team-header:hover {
    border-color: var(--border-hover);
    box-shadow: var(--shadow-md);
  }
  
  .team-info {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .team-name {
    font-size: 1.2rem;
    font-weight: bold;
  }
  
  .team-score {
    display: flex;
    gap: 0.5rem;
    font-size: 0.9rem;
  }
  
  .kills { color: #27ae60; }
  .deaths { color: #e74c3c; }
  .assists { color: #f39c12; }
  
  .team-gold {
    font-size: 1.1rem;
    font-weight: bold;
    color: #f1c40f;
  }
  
  .blue-team .team-name {
    color: #4a90e2;
  }
  
  .red-team .team-name {
    color: #e74c3c;
  }
  
  .players {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .player-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--bg-primary);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-color);
    transition: var(--transition-normal);
    position: relative;
  }
  
  .player-row:hover {
    background: var(--bg-hover);
    border-color: var(--border-hover);
    box-shadow: var(--shadow-sm);
    transform: translateY(-1px);
  }
  
  .player-row::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 3px;
    background: var(--accent-color);
    border-radius: var(--radius-sm) 0 0 var(--radius-sm);
    opacity: 0;
    transition: var(--transition-normal);
  }
  
  .player-row:hover::before {
    opacity: 1;
  }
  
  .champion-icon {
    width: 40px;
    height: 40px;
    border-radius: var(--radius-sm);
    border: 2px solid var(--border-color);
    transition: var(--transition-normal);
  }
  
  .champion-icon:hover {
    border-color: var(--accent-color);
    box-shadow: 0 0 8px rgba(74, 144, 226, 0.3);
  }
  
  .player-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .player-name {
    font-size: 0.9rem;
    font-weight: 500;
  }
  
  .player-stats {
    display: flex;
    gap: 0.75rem;
    font-size: 0.8rem;
    color: #8b949e;
  }
  
  .items {
    display: flex;
    gap: 2px;
  }
  
  .item-icon {
    width: 24px;
    height: 24px;
    border-radius: 3px;
    border: 1px solid #30363d;
  }
  
  .game-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-width: 200px;
    gap: 1.5rem;
  }
  
  .game-time {
    text-align: center;
    background: var(--bg-secondary);
    padding: 1.5rem 2rem;
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-color);
    position: relative;
    overflow: hidden;
  }
  
  .game-time::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(45deg, transparent 30%, rgba(74, 144, 226, 0.1) 50%, transparent 70%);
    animation: shimmer 3s infinite;
  }
  
  .time-display {
    font-size: 2.5rem;
    font-weight: bold;
    color: var(--accent-color);
    font-family: 'Nova Flat', monospace;
    margin-bottom: 0.5rem;
    position: relative;
    z-index: 1;
    text-shadow: 0 0 10px rgba(74, 144, 226, 0.3);
  }
  
  .game-phase {
    font-size: 0.9rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 1px;
    position: relative;
    z-index: 1;
  }
  
  .data-source-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    margin-top: 1rem;
  }
  
  .data-source {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 500;
  }
  
  .data-source.websocket {
    background: rgba(34, 197, 94, 0.2);
    color: #22c55e;
    border: 1px solid rgba(34, 197, 94, 0.3);
  }
  
  .data-source.http {
    background: rgba(59, 130, 246, 0.2);
    color: #3b82f6;
    border: 1px solid rgba(59, 130, 246, 0.3);
  }
  
  .data-source.none {
    background: rgba(156, 163, 175, 0.2);
    color: #9ca3af;
    border: 1px solid rgba(156, 163, 175, 0.3);
  }
  
  .source-indicator {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
  }
  
  .data-source.websocket .source-indicator {
    animation: pulse 2s infinite;
  }
  
  .data-source.connecting {
    background: rgba(245, 158, 11, 0.2);
    color: #f59e0b;
    border: 1px solid rgba(245, 158, 11, 0.3);
  }
  
  .data-source.connecting .source-indicator {
    animation: pulse 1s infinite;
  }
  
  .spectator-mode {
    font-size: 0.7rem;
    color: #f59e0b;
    background: rgba(245, 158, 11, 0.2);
    padding: 0.25rem 0.5rem;
    border-radius: 8px;
    border: 1px solid rgba(245, 158, 11, 0.3);
  }
  
  .objectives {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .objective {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-color);
    min-width: 120px;
    transition: var(--transition-normal);
    position: relative;
  }
  
  .objective:hover {
    border-color: var(--border-hover);
    box-shadow: var(--shadow-sm);
    transform: translateY(-1px);
  }
  
  .objective-icon {
    font-size: 1.5rem;
  }
  
  .objective-timer {
    font-size: 1.1rem;
    font-weight: bold;
    color: #f39c12;
    font-family: 'Nova Flat', monospace;
  }
  
  .objective.dragon .objective-timer {
    color: #e74c3c;
  }
  
  .objective.baron .objective-timer {
    color: #8e44ad;
  }
  
  .objective.herald .objective-timer {
    color: #27ae60;
  }
  
  /* Animations */
  @keyframes shimmer {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(100%); }
  }
  
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
  
  /* Responsive design */
  @media (max-width: 1400px) {
    .live-game-overlay {
      flex-direction: column;
      gap: 1.5rem;
    }
    
    .game-info {
      order: -1;
      flex-direction: row;
      min-width: auto;
      justify-content: space-around;
    }
    
    .team {
      max-width: none;
    }
  }
  
  @media (max-width: 768px) {
    .live-game-overlay {
      padding: 1rem;
    }
    
    .game-info {
      flex-direction: column;
      gap: 1rem;
    }
    
    .objectives {
      flex-direction: row;
      justify-content: center;
    }
  }
</style>
