import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { onMount } from 'svelte';
import type { 
  LiveGameData, 
  GameStats, 
  PlayerScoresData, 
  ComprehensiveGameState 
} from '$lib/types/game';

// Global state stores
export const gameData = writable<LiveGameData | null>(null);
export const liveGameStats = writable<GameStats | null>(null);
export const playerScores = writable<PlayerScoresData | null>(null);
export const gameState = writable<ComprehensiveGameState | null>(null);

// UI State
export const isLoading = writable(false);
export const isRefreshing = writable(false);
export const error = writable<string | null>(null);
export const lastErrorState = writable<string | null>(null);
export const errorStable = writable(false);

// Game Info
export const gameTime = writable('00:00');
export const gamePhase = writable('No Live Game');
export const gameMode = writable('Unknown');
export const isSpectatorMode = writable(false);

// Game Time Tracking
let gameTimeInterval: number | null = null;
let gameStartTime: number | null = null;
let lastApiGameTime: number | null = null;
let lastApiSyncTime: number | null = null;

// Settings
export const refreshInterval = writable(1000); // Default to 1 second

// Debug state
export const copiedStates = writable<Record<string, boolean>>({});

// Stable error handling
function setStableError(errorMessage: string | null) {
  let currentError: string | null = null;
  let currentLastError: string | null = null;
  
  const unsubscribeError = error.subscribe(e => currentError = e);
  const unsubscribeLastError = lastErrorState.subscribe(e => currentLastError = e);
  
  unsubscribeError();
  unsubscribeLastError();
  
  // Only update error if it's different from the last stable state
  if (errorMessage !== currentLastError) {
    error.set(errorMessage);
    lastErrorState.set(errorMessage);
    errorStable.set(true);
    
    // Reset stability flag after a delay to allow for future changes
    setTimeout(() => {
      errorStable.set(false);
    }, 2000);
  }
}

// Game time tracking functions
function startGameTimeTicker() {
  stopGameTimeTicker();
  gameTimeInterval = setInterval(updateGameTime, 1000);
}

function stopGameTimeTicker() {
  if (gameTimeInterval) {
    clearInterval(gameTimeInterval);
    gameTimeInterval = null;
  }
}

function updateGameTime() {
  if (gameStartTime) {
    const elapsed = Math.floor((Date.now() - gameStartTime) / 1000);
    const formatted = formatGameTime(elapsed);
    gameTime.set(formatted);
  }
}

function syncGameTimeWithApi(apiGameTime: number) {
  lastApiGameTime = apiGameTime;
  lastApiSyncTime = Date.now();
  
  // Start local timer from API time
  gameStartTime = Date.now() - (apiGameTime * 1000);
  updateGameTime();
}

function getSyncStatus(): string {
  if (!lastApiSyncTime) return 'local';
  
  const timeSinceSync = Date.now() - lastApiSyncTime;
  if (timeSinceSync < 5000) return 'synced'; // Less than 5 seconds
  if (timeSinceSync < 30000) return 'stale'; // Less than 30 seconds
  return 'local';
}

function getGameModeColor(): string {
  let currentMode = 'Unknown';
  const unsubscribe = gameMode.subscribe(m => currentMode = m);
  unsubscribe();
  
  switch (currentMode) {
    case 'Arena': return 'bg-purple-500';
    case 'Classic': return 'bg-blue-500';
    case 'ARAM': return 'bg-orange-500';
    case 'URF': return 'bg-pink-500';
    default: return 'bg-gray-500';
  }
}

function getGamePhaseColor(): string {
  let currentPhase = 'No Live Game';
  const unsubscribe = gamePhase.subscribe(p => currentPhase = p);
  unsubscribe();
  
  switch (currentPhase) {
    case 'In Game': return 'bg-green-500';
    case 'Champion Select': return 'bg-yellow-500';
    case 'Loading': return 'bg-blue-500';
    case 'No Live Game': return 'bg-gray-500';
    case 'Error': return 'bg-red-500';
    default: 
      if (currentPhase.includes('Blue Team Victory')) return 'bg-blue-500';
      if (currentPhase.includes('Red Team Victory')) return 'bg-red-500';
      if (currentPhase.startsWith('Game Ended - Win')) return 'bg-emerald-500';
      if (currentPhase.startsWith('Game Ended - Loss')) return 'bg-red-500';
      if (currentPhase.startsWith('Game Ended')) return 'bg-purple-500';
      return 'bg-gray-500';
  }
}

function formatGameTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

function formatEventTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  
  // For very early game events, show relative time
  if (mins === 0 && secs < 10) {
    return `${secs}s`;
  }
  
  // For first minute, show seconds
  if (mins === 0) {
    return `${secs}s`;
  }
  
  // For longer times, show minutes and seconds
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

function formatEventTimeWithContext(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  
  // Game start
  if (seconds < 1) {
    return "Start";
  }
  
  // Very early game (first 10 seconds) - show as seconds
  if (mins === 0 && secs < 10) {
    return `${secs}s`;
  }
  
  // First minute - show as seconds for precision
  if (mins === 0) {
    return `${secs}s`;
  }
  
  // Early game (first 5 minutes) - show minutes:seconds
  if (mins < 5) {
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
  
  // Mid to late game - show minutes:seconds
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

// Data loading function
async function loadLiveGameData(isRefresh = false) {
  // Debounce rapid calls
  const now = Date.now();
  if (now - lastLoadTime < MIN_LOAD_INTERVAL) {
    console.log('Skipping load - too soon since last load');
    return;
  }
  lastLoadTime = now;
  
  try {
    if (isRefresh) {
      isRefreshing.set(true);
    } else {
      isLoading.set(true);
    }
    setStableError(null);
    
    console.log('Loading live game data...');
    
    // Try to get comprehensive game state first
    console.log('Calling get_comprehensive_game_state...');
    const comprehensiveState = await invoke('get_comprehensive_game_state') as ComprehensiveGameState;
    console.log('Comprehensive state received:', comprehensiveState);
    console.log('Game phase:', comprehensiveState?.game_phase);
    console.log('LCU connected:', comprehensiveState?.lcu_connected);
    console.log('Live client available:', comprehensiveState?.live_client_available);
    console.log('Full comprehensive state JSON:', JSON.stringify(comprehensiveState, null, 2));
    gameState.set(comprehensiveState);
    
    // Check if we're in spectator mode from comprehensive state
    if (comprehensiveState?.game_phase === 'in_game' && !comprehensiveState?.live_client_available) {
      console.log('Spectator mode detected from comprehensive state');
      isSpectatorMode.set(true);
      setStableError('Spectator mode - limited data available');
      return;
    }
    
    // If live client is available, get detailed data
    if (comprehensiveState?.live_client_available) {
      console.log('Live client is available, fetching detailed data...');
      try {
        const [game, stats, allGameData] = await Promise.all([
          invoke('get_live_game_data') as Promise<LiveGameData>,
          invoke('get_live_game_stats', { riotId: 'current_player' }) as Promise<GameStats>,
          invoke('get_all_game_data') as Promise<unknown>
        ]);
        
        // Try to get player scores separately to avoid breaking the main flow
        let scores = null;
        try {
          scores = await invoke('get_player_scores', { riotId: 'current_player' }) as PlayerScoresData;
        } catch (scoreError) {
          console.warn('Player scores not available:', scoreError);
          // Don't set this as an error since it's optional data
        }
        
        console.log('Game data:', game);
        console.log('Live game stats:', stats);
        console.log('Player scores:', scores);
        console.log('All game data:', allGameData);
        
        // Use allGameData if available, otherwise fall back to game data
        const finalGameData = allGameData && typeof allGameData === 'object' && 'allPlayers' in allGameData 
          ? allGameData 
          : game;
        
        gameData.set(finalGameData);
        liveGameStats.set(stats);
        playerScores.set(scores);
        
        // Extract game time and mode from all game data
        if (allGameData && typeof allGameData === 'object' && 'gameData' in allGameData) {
          const gameData = allGameData as { gameData: { gameTime?: number; gameMode?: string } };
          if (gameData.gameData?.gameTime !== undefined) {
            // Sync our local timer with the API time
            syncGameTimeWithApi(gameData.gameData.gameTime);
          }
          
          if (gameData.gameData?.gameMode) {
            gameMode.set(gameData.gameData.gameMode);
          }
        }
        
        // Check if we're in spectator mode
        let isSpectator = false;
        if (allGameData && typeof allGameData === 'object' && 'activePlayer' in allGameData) {
          const activePlayer = (allGameData as any).activePlayer;
          isSpectator = activePlayer?.error === "Spectator mode doesn't currently support this feature";
          isSpectatorMode.set(isSpectator);
        }
        
        // Handle spectator mode - show limited data but don't treat as error
        if (isSpectator) {
          console.log('Spectator mode detected - showing limited data');
          // Don't clear error here, let the UI handle spectator mode gracefully
        }
        
        // Check if game has ended by looking for GameEnd event
        let gameHasEnded = false;
        let gameResult = '';
        let winningTeam = '';
        if (allGameData && typeof allGameData === 'object' && 'events' in allGameData) {
          const events = (allGameData as any).events;
          if (events?.Events && Array.isArray(events.Events)) {
            const gameEndEvent = events.Events.find((event: any) => event.EventName === 'GameEnd');
            if (gameEndEvent) {
              gameHasEnded = true;
              gameResult = gameEndEvent.Result || 'Unknown';
              
              // Find the ace event to determine winning team
              const aceEvent = events.Events.find((event: any) => event.EventName === 'Ace');
              if (aceEvent && aceEvent.AcingTeam) {
                winningTeam = aceEvent.AcingTeam === 'ORDER' ? 'Blue Team' : 'Red Team';
              }
            }
          }
        }
        
        if (gameHasEnded) {
          if (winningTeam && gameResult === 'Win') {
            gamePhase.set(`${winningTeam} Victory!`);
          } else {
            gamePhase.set(`Game Ended - ${gameResult}`);
          }
          // Stop the game timer when game ends
          stopGameTimeTicker();
        } else {
          gamePhase.set('In Game');
        }
      } catch (apiError) {
        // If individual APIs fail, show error but don't crash
        console.warn('Some live game APIs failed:', apiError);
        error.set(`Some game data unavailable: ${typeof apiError === 'string' ? apiError : JSON.stringify(apiError)}`);
      }
    } else {
      // No live client available
      console.log('Live client not available');
      gameData.set(null);
      liveGameStats.set(null);
      playerScores.set(null);
      gamePhase.set('No Live Game');
      gameMode.set('Unknown');
      gameTime.set('00:00');
      gameStartTime = null;
      lastApiGameTime = null;
      lastApiSyncTime = null;
      
      // Check if LCU is connected but no live game
      if (comprehensiveState?.lcu_connected) {
        setStableError('League client connected but not in a live game');
      } else {
        setStableError('League client not detected or not connected');
      }
    }
    
    // Update phase and mode from available data (game time is handled by ticker)
    let currentGameData: LiveGameData | null = null;
    const unsubscribeGameData = gameData.subscribe(gd => currentGameData = gd);
    unsubscribeGameData();
    
    if (currentGameData && 'phase' in currentGameData) {
      gamePhase.set((currentGameData as any).phase);
    }
    
    let currentGameState: ComprehensiveGameState | null = null;
    const unsubscribeGameState = gameState.subscribe(gs => currentGameState = gs);
    unsubscribeGameState();
    
    if (currentGameState && 'gameMode' in currentGameState) {
      gameMode.set((currentGameState as any).gameMode);
    }
  } catch (err) {
    setStableError(err as string);
    console.error('Failed to load live game data:', err);
    // Clear all data on error
    gameData.set(null);
    liveGameStats.set(null);
    playerScores.set(null);
    gamePhase.set('Error');
    gameMode.set('Unknown');
    gameTime.set('00:00');
    gameStartTime = null;
    lastApiGameTime = null;
    lastApiSyncTime = null;
  } finally {
    if (isRefresh) {
      isRefreshing.set(false);
    } else {
      isLoading.set(false);
    }
  }
}

// Load config function
async function loadConfig() {
  try {
    const config = await invoke('get_config') as any;
    if (config?.ui?.refresh_interval) {
      refreshInterval.set(config.ui.refresh_interval);
    }
  } catch (err) {
    console.warn('Failed to load config, using default refresh interval:', err);
  }
}

// Action functions
async function refresh() {
  await loadLiveGameData(true);
}

async function debugLcuConnection() {
  try {
    console.log('🔍 Calling debug_lcu_connection command...');
    const debugInfo = await invoke('debug_lcu_connection');
    console.log('🔍 Debug info received:', debugInfo);
    console.log('🔍 Connection status:', debugInfo?.connection_status);
    console.log('🔍 Live client available:', debugInfo?.live_client_available);
    console.log('🔍 Game flow phase:', debugInfo?.game_flow_phase);
    console.log('🔍 Errors:', debugInfo?.errors);
    return debugInfo;
  } catch (err) {
    console.error('🔍 Failed to get debug info:', err);
    throw err;
  }
}

async function copyToClipboard(data: any, key: string) {
  try {
    const jsonString = JSON.stringify(data, null, 2);
    await navigator.clipboard.writeText(jsonString);
    copiedStates.update(states => ({ ...states, [key]: true }));
    
    // Reset the copied state after 2 seconds
    setTimeout(() => {
      copiedStates.update(states => ({ ...states, [key]: false }));
    }, 2000);
  } catch (err) {
    console.error('Failed to copy to clipboard:', err);
  }
}

// Store management
let isListening = false;
let refreshIntervalId: number | null = null;
let refreshIntervalUnsubscribe: (() => void) | null = null;
let lastLoadTime = 0;
const MIN_LOAD_INTERVAL = 500; // Minimum 500ms between loads

export function startListening() {
  if (isListening) {
    console.log('Live game data listening already active, skipping start');
    return;
  }
  isListening = true;
  
  console.log('Starting live game data listening...');
  
  // Load config first to get the refresh interval, then start everything
  loadConfig().then(() => {
    // Start initial data loading
    loadLiveGameData();
    startGameTimeTicker();
    
    // Set up auto-refresh with the configured interval
    refreshIntervalUnsubscribe = refreshInterval.subscribe(interval => {
      // Clear existing interval
      if (refreshIntervalId) {
        clearInterval(refreshIntervalId);
        refreshIntervalId = null;
      }
      
      // Set up new interval if we have a valid interval value (minimum 1 second)
      if (interval && interval >= 1000) {
        console.log(`Setting up auto-refresh with interval: ${interval}ms`);
        refreshIntervalId = setInterval(() => {
          loadLiveGameData(true);
        }, interval);
      } else {
        console.log('Skipping auto-refresh setup - interval too small or invalid:', interval);
      }
    });
  });
}

export function stopListening() {
  if (!isListening) return;
  isListening = false;
  
  console.log('Stopping live game data listening...');
  
  // Clear refresh interval
  if (refreshIntervalId) {
    clearInterval(refreshIntervalId);
    refreshIntervalId = null;
  }
  
  // Unsubscribe from refresh interval changes
  if (refreshIntervalUnsubscribe) {
    refreshIntervalUnsubscribe();
    refreshIntervalUnsubscribe = null;
  }
  
  // Stop game time ticker
  stopGameTimeTicker();
  
  // Clear all data
  gameData.set(null);
  liveGameStats.set(null);
  playerScores.set(null);
  gameState.set(null);
  error.set(null);
  gameTime.set('00:00');
  gamePhase.set('No Live Game');
  gameMode.set('Unknown');
  isSpectatorMode.set(false);
  
  // Reset tracking variables
  gameStartTime = null;
  lastApiGameTime = null;
  lastApiSyncTime = null;
}

export function isActive(): boolean {
  return isListening;
}

// Export derived stores for computed values
export const syncStatus = derived([], () => getSyncStatus());
export const gameModeColor = derived([gameMode], () => getGameModeColor());
export const gamePhaseColor = derived([gamePhase], () => getGamePhaseColor());

// Export action functions
export const liveGameActions = {
  refresh,
  debugLcuConnection,
  copyToClipboard,
  getSyncStatus,
  getGameModeColor,
  getGamePhaseColor,
  formatGameTime,
  formatEventTime,
  formatEventTimeWithContext
};
