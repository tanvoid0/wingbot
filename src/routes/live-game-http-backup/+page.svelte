<script lang="ts">
  import LiveGameOverlay from '$lib/components/live-game/LiveGameOverlay.svelte';
  import ConnectionHealthCard from '$lib/components/live-game/ConnectionHealthCard.svelte';
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { 
    wsGameData, 
    wsLiveGameData,
    wsGameTime, 
    wsGamePhase, 
    wsGameMode,
    wsIsSpectatorMode,
    wsConnected,
    wsConnecting,
    wsLCUConnecting,
    wsError,
    wsGameEvents,
    wsActions,
    startWebSocketListening,
    stopWebSocketListening
  } from '$lib/stores/websocketStore';
  
  let isInGame = false;
  let gameTime = '00:00';
  let gamePhase = 'unknown';
  let gameMode = 'Unknown';
  let isSpectatorMode = false;
  let isWebSocketConnected = false;
  let isWebSocketConnecting = false;
  let isLCUConnecting = false;
  let connectionError: string | null = null;
  let refreshInterval: number | null = null;
  let gameEvents: any[] = [];
  let showEventsPanel = true;
  
  onMount(() => {
    // Subscribe to WebSocket data
    const unsub1 = wsGameData.subscribe((data: any) => {
      isInGame = data && data.allPlayers && data.allPlayers.length > 0;
    });
    
    const unsub2 = wsLiveGameData.subscribe((data: any) => {
      if (data && data.allPlayers && data.allPlayers.length > 0) {
        isInGame = true;
      }
    });
    
    const unsub3 = wsGameTime.subscribe((time: string) => {
      gameTime = time;
    });
    
    const unsub4 = wsGamePhase.subscribe((phase: string) => {
      gamePhase = phase;
    });
    
    const unsub5 = wsGameMode.subscribe((mode: string) => {
      gameMode = mode;
    });
    
    const unsub6 = wsIsSpectatorMode.subscribe((spectator: boolean) => {
      isSpectatorMode = spectator;
    });
    
    const unsub7 = wsConnected.subscribe((connected: boolean) => {
      isWebSocketConnected = connected;
    });
    
    const unsub8 = wsConnecting.subscribe((connecting: boolean) => {
      isWebSocketConnecting = connecting;
    });
    
    const unsub9 = wsLCUConnecting.subscribe((connecting: boolean) => {
      isLCUConnecting = connecting;
    });
    
    const unsub10 = wsError.subscribe((error: string | null) => {
      connectionError = error;
    });
    
    const unsub11 = wsGameEvents.subscribe((events: any[]) => {
      console.log('[Live Game] Received events from WebSocket store:', events.length, events);
      // Process WebSocket events and convert to display format
      gameEvents = events.map(event => ({
        time: formatEventTime(event.timestamp),
        type: event.type,
        description: formatEventDescription(event)
      })).slice(0, 10); // Show last 10 events
      console.log('[Live Game] Processed events for display:', gameEvents.length, gameEvents);
    });
    
    // Start WebSocket listening
    startWebSocketListening();
    
    // Set up periodic refresh for WebSocket data
    refreshInterval = setInterval(() => {
      if (isWebSocketConnected) {
        wsActions.refresh();
      }
    }, 10000); // Refresh every 10 seconds when connected
    
      return () => {
        unsub1();
        unsub2();
        unsub3();
        unsub4();
        unsub5();
        unsub6();
        unsub7();
        unsub8();
        unsub9();
        unsub10();
        unsub11();
        if (refreshInterval) {
          clearInterval(refreshInterval);
        }
        stopWebSocketListening();
      };
  });
  
  function handleRefresh() {
    wsActions.refresh();
  }
  
  async function handleDebugConnection() {
    try {
      const debugInfo = await wsActions.debugConnection();
      console.log('🔍 WebSocket Debug Info:', debugInfo);
      
      // Enhanced debug info
      const enhancedDebugInfo = {
        ...debugInfo,
        currentState: {
          isInGame,
          gameTime,
          gamePhase,
          gameMode,
          isSpectatorMode,
          isWebSocketConnected,
          isWebSocketConnecting,
          isLCUConnecting,
          connectionError
        },
        stores: {
          wsGameData: get(wsGameData),
          wsLiveGameData: get(wsLiveGameData),
          wsGameTime: get(wsGameTime),
          wsGamePhase: get(wsGamePhase),
          wsGameMode: get(wsGameMode),
          wsGameEvents: get(wsGameEvents)
        }
      };
      
      console.log('🔍 Enhanced Debug Info:', enhancedDebugInfo);
      alert(`WebSocket Debug Info:\n${JSON.stringify(enhancedDebugInfo, null, 2)}`);
    } catch (error) {
      console.error('🔍 Debug failed:', error);
      alert(`Debug failed: ${error}`);
    }
  }
  
  function handleTestEvents() {
    console.log('🧪 Testing events...');
    // Simulate adding test events to the WebSocket store
    const testEvents = [
      {
        type: 'kill',
        timestamp: Date.now() - 5000,
        killer: 'TestPlayer1',
        victim: 'TestPlayer2'
      },
      {
        type: 'dragon',
        timestamp: Date.now() - 10000,
        team: 'Blue Team',
        dragonType: 'Infernal'
      },
      {
        type: 'tower',
        timestamp: Date.now() - 15000,
        team: 'Red Team',
        towerType: 'top'
      }
    ];
    
    // Update the WebSocket events store directly
    wsGameEvents.update(events => {
      const newEvents = [...testEvents, ...events].slice(0, 50);
      console.log('🧪 Added test events:', newEvents.length);
      return newEvents;
    });
  }
  
  function getEventIcon(eventType: string): string {
    switch (eventType) {
      case 'kill': return '⚔️';
      case 'death': return '💀';
      case 'assist': return '🤝';
      case 'dragon': return '🐉';
      case 'baron': return '👹';
      case 'herald': return '🦄';
      case 'tower': return '🏰';
      case 'inhibitor': return '🛡️';
      case 'first_blood': return '🩸';
      case 'ace': return '🎯';
      case 'shutdown': return '💥';
      case 'penta': return '🏆';
      case 'quadra': return '⭐';
      case 'triple': return '🔥';
      case 'double': return '⚡';
      default: return '📊';
    }
  }
  
  
  function formatEventTime(timestamp: number): string {
    const date = new Date(timestamp);
    const minutes = date.getMinutes().toString().padStart(2, '0');
    const seconds = date.getSeconds().toString().padStart(2, '0');
    return `${minutes}:${seconds}`;
  }
  
  function formatEventDescription(eventData: any): string {
    switch (eventData.type) {
      case 'kill':
        return `${eventData.killer} killed ${eventData.victim}`;
      case 'death':
        return `${eventData.victim} was killed by ${eventData.killer}`;
      case 'assist':
        return `${eventData.assister} assisted in killing ${eventData.victim}`;
      case 'dragon':
        return `${eventData.team} secured ${eventData.dragonType} Dragon`;
      case 'baron':
        return `${eventData.team} secured Baron Nashor`;
      case 'herald':
        return `${eventData.team} secured Rift Herald`;
      case 'tower':
        return `${eventData.team} destroyed ${eventData.towerType} tower`;
      case 'inhibitor':
        return `${eventData.team} destroyed ${eventData.lane} inhibitor`;
      case 'first_blood':
        return `First Blood - ${eventData.team}`;
      case 'ace':
        return `${eventData.team} achieved an Ace`;
      case 'shutdown':
        return `${eventData.killer} shut down ${eventData.victim}`;
      case 'penta':
        return `${eventData.killer} achieved a PENTAKILL!`;
      case 'quadra':
        return `${eventData.killer} achieved a QUADRAKILL!`;
      case 'triple':
        return `${eventData.killer} achieved a TRIPLE KILL!`;
      case 'double':
        return `${eventData.killer} achieved a DOUBLE KILL!`;
      default:
        return eventData.description || 'Unknown event';
    }
  }
</script>

<svelte:head>
  <title>Live Game - WingBot</title>
</svelte:head>

<div class="live-game-page">
  <div class="page-header">
    <div class="header-left">
      <h1 class="page-title">
        <span class="page-icon">🎮</span>
        Live Game
      </h1>
      <div class="game-mode-badge {gameMode.toLowerCase()}">
        {gameMode}
      </div>
    </div>
    
    <div class="header-center">
      <div class="game-status-card">
        <div class="status-indicator {isInGame ? 'online' : (isWebSocketConnecting || isLCUConnecting ? 'connecting' : 'offline')}"></div>
        <div class="status-content">
          <div class="status-text">
            {#if isWebSocketConnecting}
              Connecting to WebSocket...
            {:else if isLCUConnecting}
              Connecting to League Client...
            {:else if connectionError}
              Connection Error
            {:else if isInGame}
              {isSpectatorMode ? 'Spectating' : 'In Game'}
            {:else}
              Not in Game
            {/if}
          </div>
          {#if isInGame}
            <div class="game-timer">{gameTime}</div>
          {/if}
        </div>
      </div>
    </div>
    
    <div class="header-right">
      <div class="connection-status-grid">
        <div class="connection-item">
          <span class="connection-label">WebSocket</span>
          <span class="connection-badge {isWebSocketConnected ? 'connected' : (isWebSocketConnecting ? 'connecting' : 'disconnected')}">
            {isWebSocketConnected ? 'Connected' : (isWebSocketConnecting ? 'Connecting' : 'Disconnected')}
          </span>
        </div>
        {#if isLCUConnecting}
          <div class="connection-item">
            <span class="connection-label">LCU</span>
            <span class="connection-badge connecting">Connecting</span>
          </div>
        {/if}
      </div>
      
      <div class="action-buttons">
        <button class="action-btn refresh" onclick={handleRefresh} title="Refresh Data" aria-label="Refresh Data">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
            <path d="M21 3v5h-5"/>
            <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/>
            <path d="M3 21v-5h5"/>
          </svg>
        </button>
        <button class="action-btn debug" onclick={handleDebugConnection} title="Debug Connection" aria-label="Debug Connection">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/>
            <path d="M12 17h.01"/>
          </svg>
        </button>u
        <button class="action-btn test" onclick={handleTestEvents} title="Test Events">
          🧪
        </button>
      </div>
    </div>
  </div>
  
  <div class="content-area">
    <!-- Connection Health Card -->
    <ConnectionHealthCard />
    
    {#if isInGame}
      <div class="game-layout">
        <div class="overlay-container animate-fade-in">
          <LiveGameOverlay />
        </div>
        
        {#if showEventsPanel}
          <div class="events-panel animate-fade-in">
            <div class="events-header">
              <h3>📊 Live Events</h3>
              <button class="toggle-events-btn" onclick={() => showEventsPanel = false} title="Hide Events">
                ✕
              </button>
            </div>
            
            <div class="events-list">
              {#if gameEvents.length > 0}
                {#each gameEvents.slice(0, 10) as event}
                  <div class="event-item {event.type}">
                    <div class="event-time">{event.time}</div>
                    <div class="event-icon">{getEventIcon(event.type)}</div>
                    <div class="event-description">{event.description}</div>
                  </div>
                {/each}
              {:else}
                <div class="no-events">
                  <div class="no-events-icon">📊</div>
                  <p>Waiting for game events...</p>
                  <small>
                    {#if isWebSocketConnected}
                      Connected to WebSocket - events will appear here as they happen
                    {:else}
                      WebSocket disconnected - events will appear when connection is restored
                    {/if}
                  </small>
                </div>
              {/if}
            </div>
          </div>
        {:else}
          <button class="show-events-btn" onclick={() => showEventsPanel = true} title="Show Events">
            📊 Events
          </button>
        {/if}
      </div>
    {:else}
      <div class="loading-state animate-fade-in">
        <div class="loading-spinner"></div>
        <h2>
          {#if isWebSocketConnecting}
            Connecting to WebSocket Server...
          {:else if isLCUConnecting}
            Connecting to League Client...
          {:else if connectionError}
            Connection Error
          {:else}
            Waiting for Game to Start...
          {/if}
        </h2>
        <p>
          {#if isWebSocketConnecting}
            Establishing connection to WingBot WebSocket server...
          {:else if isLCUConnecting}
            Detecting League of Legends client connection...
          {:else if connectionError}
            {connectionError}
          {:else}
            Please make sure you're in an active League of Legends game.
          {/if}
        </p>
        {#if connectionError}
          <button class="retry-btn" onclick={handleRefresh}>
            🔄 Retry Connection
          </button>
        {/if}
      </div>
    {/if}
  </div>
  
  <div class="page-footer">
    <div class="feature-grid">
      <div class="feature-card">
        <div class="feature-icon">📊</div>
        <div class="feature-content">
          <h3>Real-time Stats</h3>
          <p>Live KDA, CS, gold, and damage tracking for all players.</p>
        </div>
      </div>
      
      <div class="feature-card">
        <div class="feature-icon">⏰</div>
        <div class="feature-content">
          <h3>Objective Timers</h3>
          <p>Track dragon, baron, and herald spawn timers with notifications.</p>
          </div>
          </div>
      
      <div class="feature-card">
        <div class="feature-icon">🎯</div>
        <div class="feature-content">
          <h3>AI Insights</h3>
          <p>Get strategic recommendations based on current game state.</p>
          </div>
        </div>
        
      <div class="feature-card">
        <div class="feature-icon">⚡</div>
        <div class="feature-content">
          <h3>Performance Analysis</h3>
          <p>Compare your performance against team averages and trends.</p>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .live-game-page {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    background: var(--bg-primary);
    color: var(--text-primary);
  }
  
  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.5rem 2rem;
    background: linear-gradient(135deg, var(--bg-secondary) 0%, var(--bg-primary) 100%);
    border-bottom: 1px solid var(--border-color);
    position: relative;
    overflow: hidden;
  }
  
  .page-header::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: linear-gradient(90deg, var(--accent-color), var(--success-color), var(--warning-color));
    opacity: 0.8;
  }
  
  .header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
  }
  
  .page-title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: 1.75rem;
    font-weight: var(--font-weight-bold);
    margin: 0;
    color: var(--accent-color);
  }
  
  .page-icon {
    font-size: 2rem;
  }
  
  .game-mode-badge {
    padding: 0.25rem 0.75rem;
    border-radius: var(--radius-full);
    font-size: 0.75rem;
    font-weight: var(--font-weight-medium);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
  }
  
  .game-mode-badge.ranked {
    background: rgba(74, 144, 226, 0.2);
    border-color: var(--accent-color);
    color: var(--accent-color);
  }
  
  .game-mode-badge.normal {
    background: rgba(34, 197, 94, 0.2);
    border-color: var(--success-color);
    color: var(--success-color);
  }
  
  .header-center {
    flex: 1;
    display: flex;
    justify-content: center;
  }
  
  .header-right {
    display: flex;
    align-items: center;
    gap: 1.5rem;
  }
  
  .game-status-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    background: var(--bg-primary);
    padding: 1rem 1.5rem;
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-color);
    box-shadow: var(--shadow-md);
    min-width: 200px;
  }
  
  .status-indicator {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #666;
  }
  
  .status-indicator.online {
    background: var(--success-color);
    animation: pulse 2s infinite;
  }
  
  .status-indicator.connecting {
    background: var(--warning-color);
    animation: pulse 1s infinite;
  }
  
  .status-indicator.offline {
    background: var(--error-color);
  }
  
  .status-content {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .status-text {
    font-weight: var(--font-weight-medium);
    color: var(--text-primary);
    font-size: 0.875rem;
  }
  
  .connection-status-grid {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .connection-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .connection-label {
    font-size: 0.75rem;
    color: var(--text-tertiary);
    min-width: 60px;
  }
  
  .connection-badge {
    padding: 0.125rem 0.5rem;
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    font-weight: var(--font-weight-medium);
    text-transform: uppercase;
    letter-spacing: 0.25px;
  }
  
  .connection-badge.connected {
    background: rgba(34, 197, 94, 0.2);
    color: var(--success-color);
    border: 1px solid rgba(34, 197, 94, 0.3);
  }
  
  .connection-badge.connecting {
    background: rgba(245, 158, 11, 0.2);
    color: var(--warning-color);
    border: 1px solid rgba(245, 158, 11, 0.3);
  }
  
  .connection-badge.disconnected {
    background: rgba(239, 68, 68, 0.2);
    color: var(--error-color);
    border: 1px solid rgba(239, 68, 68, 0.3);
  }
  
  .game-timer {
    font-family: 'Nova Flat', monospace;
    font-size: 1.25rem;
    font-weight: var(--font-weight-bold);
    color: var(--accent-color);
    background: var(--bg-secondary);
    padding: 0.5rem 1rem;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-color);
  }
  
  .action-buttons {
    display: flex;
    gap: 0.5rem;
  }
  
  .action-btn {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: 0.75rem;
    cursor: pointer;
    transition: var(--transition-normal);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
  }
  
  .action-btn:hover {
    background: var(--bg-hover);
    border-color: var(--border-hover);
    color: var(--text-primary);
  }
  
  .action-btn.refresh:hover {
    transform: rotate(180deg);
  }
  
  .action-btn.debug:hover {
    transform: scale(1.1);
  }
  
  .connection-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .ws-status {
    font-size: 0.75rem;
    padding: 0.25rem 0.5rem;
    border-radius: var(--radius-sm);
    font-weight: var(--font-weight-medium);
  }
  
  .ws-status.connected {
    background: rgba(34, 197, 94, 0.2);
    color: var(--success-color);
    border: 1px solid rgba(34, 197, 94, 0.3);
  }
  
  .ws-status.disconnected {
    background: rgba(239, 68, 68, 0.2);
    color: var(--error-color);
    border: 1px solid rgba(239, 68, 68, 0.3);
  }
  
  .ws-status.connecting {
    background: rgba(245, 158, 11, 0.2);
    color: var(--warning-color);
    border: 1px solid rgba(245, 158, 11, 0.3);
  }
  
  .lcu-status {
    font-size: 0.75rem;
    padding: 0.25rem 0.5rem;
    border-radius: var(--radius-sm);
    font-weight: var(--font-weight-medium);
  }
  
  .lcu-status.connecting {
    background: rgba(245, 158, 11, 0.2);
    color: var(--warning-color);
    border: 1px solid rgba(245, 158, 11, 0.3);
  }
  
  .retry-btn {
    background: var(--accent-color);
    border: 1px solid var(--accent-color);
    border-radius: var(--radius-md);
    padding: 0.75rem 1.5rem;
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: white;
    transition: var(--transition-normal);
    margin-top: 1rem;
  }
  
  .retry-btn:hover {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
    transform: translateY(-1px);
  }
  
  .content-area {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
  }
  
  .game-layout {
    display: flex;
    gap: 2rem;
    width: 100%;
    max-width: 1600px;
    align-items: flex-start;
    min-height: 600px;
  }
  
  .overlay-container {
    flex: 1;
    max-width: 1200px;
    min-width: 800px;
  }
  
  .events-panel {
    width: 400px;
    background: linear-gradient(135deg, var(--bg-secondary) 0%, var(--bg-primary) 100%);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xl);
    max-height: 600px;
    min-height: 500px;
    display: flex;
    flex-direction: column;
    position: relative;
    overflow: hidden;
    flex-shrink: 0;
  }
  
  .events-panel::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    background: linear-gradient(90deg, var(--accent-color), var(--success-color), var(--warning-color));
    opacity: 0.9;
  }
  
  .events-panel::after {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    bottom: 0;
    width: 2px;
    background: linear-gradient(180deg, var(--accent-color), var(--success-color), var(--warning-color));
    opacity: 0.6;
  }
  
  .events-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-primary);
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
  }
  
  .events-header h3 {
    margin: 0;
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
  }
  
  .toggle-events-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 0.25rem;
    border-radius: var(--radius-sm);
    transition: var(--transition-normal);
    font-size: 1rem;
  }
  
  .toggle-events-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  
  .events-list {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }
  
  .event-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.875rem;
    margin-bottom: 0.5rem;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    position: relative;
    overflow: hidden;
  }
  
  .event-item::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    width: 3px;
    height: 100%;
    background: var(--border-color);
    transition: var(--transition-normal);
  }
  
  .event-item:hover {
    border-color: var(--border-hover);
    box-shadow: var(--shadow-md);
    transform: translateY(-1px);
  }
  
  .event-item:hover::before {
    width: 4px;
  }
  
  .event-item.kill::before {
    background: var(--success-color);
  }
  
  .event-item.death::before {
    background: var(--error-color);
  }
  
  .event-item.dragon::before {
    background: #e74c3c;
  }
  
  .event-item.baron::before {
    background: #8e44ad;
  }
  
  .event-item.tower::before {
    background: #f39c12;
  }
  
  .event-item.first_blood::before {
    background: #e91e63;
  }
  
  .event-item.ace::before {
    background: #ff9800;
  }
  
  .event-time {
    font-family: 'Nova Flat', monospace;
    font-size: 0.75rem;
    color: var(--text-tertiary);
    min-width: 40px;
  }
  
  .event-icon {
    font-size: 1.25rem;
    min-width: 24px;
    text-align: center;
  }
  
  .event-description {
    flex: 1;
    font-size: 0.875rem;
    color: var(--text-primary);
    line-height: 1.4;
  }
  
  .no-events {
    text-align: center;
    padding: 2rem 1rem;
    color: var(--text-secondary);
  }
  
  .no-events-icon {
    font-size: 2rem;
    margin-bottom: 1rem;
    opacity: 0.5;
  }
  
  .no-events p {
    margin: 0 0 0.5rem 0;
    font-weight: var(--font-weight-medium);
  }
  
  .no-events small {
    font-size: 0.75rem;
    opacity: 0.7;
  }
  
  .show-events-btn {
    position: fixed;
    top: 50%;
    right: 2rem;
    transform: translateY(-50%);
    background: linear-gradient(135deg, var(--accent-color) 0%, var(--accent-hover) 100%);
    border: 1px solid var(--accent-color);
    border-radius: var(--radius-lg);
    padding: 1rem 1.5rem;
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: white;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: var(--shadow-lg);
    z-index: 10;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .show-events-btn:hover {
    background: linear-gradient(135deg, var(--accent-hover) 0%, var(--accent-color) 100%);
    border-color: var(--accent-hover);
    transform: translateY(-50%) translateY(-2px);
    box-shadow: var(--shadow-xl);
  }
  
  .show-events-btn:active {
    transform: translateY(-50%) translateY(0px);
  }
  
  .loading-state {
    text-align: center;
    padding: 4rem 2rem;
  }
  
  .loading-spinner {
    width: 60px;
    height: 60px;
    border: 4px solid var(--border-color);
    border-top: 4px solid var(--accent-color);
    border-radius: 50%;
    animation: rotate 1s linear infinite;
    margin: 0 auto 2rem;
  }
  
  .loading-state h2 {
    font-size: 1.5rem;
    font-weight: var(--font-weight-semibold);
    margin-bottom: 1rem;
    color: var(--text-primary);
  }
  
  .loading-state p {
    color: var(--text-secondary);
    font-size: var(--font-size-lg);
  }
  
  .page-footer {
    padding: 2rem;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
  }
  
  .feature-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1.5rem;
    max-width: 1200px;
    margin: 0 auto;
  }
  
  .feature-card {
    display: flex;
    align-items: flex-start;
    gap: 1rem;
    padding: 1.5rem;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    transition: var(--transition-normal);
  }
  
  .feature-card:hover {
    border-color: var(--border-hover);
    box-shadow: var(--shadow-lg);
    transform: translateY(-2px);
  }
  
  .feature-icon {
    font-size: 2rem;
    flex-shrink: 0;
  }
  
  .feature-content h3 {
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-semibold);
    margin: 0 0 0.5rem 0;
    color: var(--text-primary);
  }
  
  .feature-content p {
    color: var(--text-secondary);
    line-height: var(--line-height-relaxed);
    margin: 0;
  }
  
  /* Responsive Design */
  @media (max-width: 1400px) {
    .game-layout {
      flex-direction: column;
      gap: 1.5rem;
      align-items: center;
    }
    
    .overlay-container {
      min-width: auto;
      width: 100%;
    }
    
    .events-panel {
      width: 100%;
      max-width: 600px;
      min-height: 400px;
    }
    
    .show-events-btn {
      position: static;
      transform: none;
      margin-top: 1rem;
      align-self: center;
    }
  }
  
  @media (max-width: 768px) {
    .page-header {
      flex-direction: column;
      gap: 1rem;
      text-align: center;
    }
    
    .page-title {
      font-size: 1.5rem;
    }
    
    .game-status {
      flex-direction: column;
      gap: 0.5rem;
    }
    
    .content-area {
      padding: 1rem;
    }
    
    .game-layout {
      gap: 1rem;
    }
    
    .events-panel {
      max-height: 400px;
    }
    
    .feature-grid {
      grid-template-columns: 1fr;
    }
    
    .feature-card {
      flex-direction: column;
      text-align: center;
    }
  }
  
  /* Animations */
  @keyframes slideInFromRight {
    0% { 
      transform: translateX(100%); 
      opacity: 0; 
    }
    100% { 
      transform: translateX(0); 
      opacity: 1; 
    }
  }
  
  @keyframes fadeInUp {
    0% { 
      transform: translateY(20px); 
      opacity: 0; 
    }
    100% { 
      transform: translateY(0); 
      opacity: 1; 
    }
  }
  
  @keyframes glow {
    0%, 100% { 
      box-shadow: 0 0 5px rgba(74, 144, 226, 0.3); 
    }
    50% { 
      box-shadow: 0 0 20px rgba(74, 144, 226, 0.6); 
    }
  }
  
  .animate-fade-in {
    animation: fadeInUp 0.6s cubic-bezier(0.4, 0, 0.2, 1);
  }
  
  .events-panel {
    animation: slideInFromRight 0.5s cubic-bezier(0.4, 0, 0.2, 1);
  }
  
  .event-item {
    animation: fadeInUp 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  }
  
  .event-item:nth-child(1) { animation-delay: 0.1s; }
  .event-item:nth-child(2) { animation-delay: 0.2s; }
  .event-item:nth-child(3) { animation-delay: 0.3s; }
  .event-item:nth-child(4) { animation-delay: 0.4s; }
  .event-item:nth-child(5) { animation-delay: 0.5s; }
</style>