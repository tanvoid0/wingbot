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
  try {
    if (isRefresh) {
      isRefreshing.set(true);
    } else {
      isLoading.set(true);
    }
    error.set(null);
    
    console.log('Loading live game data...');
    
    // Try to get comprehensive game state first
    const comprehensiveState = await invoke('get_comprehensive_game_state') as ComprehensiveGameState;
    console.log('Comprehensive state:', comprehensiveState);
    gameState.set(comprehensiveState);
    
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
        }
        
        console.log('Game data:', game);
        console.log('Live game stats:', stats);
        console.log('Player scores:', scores);
        console.log('All game data:', allGameData);
        
        gameData.set(game);
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
        if (allGameData && typeof allGameData === 'object' && 'activePlayer' in allGameData) {
          const activePlayer = (allGameData as any).activePlayer;
          isSpectatorMode.set(activePlayer?.error === "Spectator mode doesn't currently support this feature");
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
        error.set('League client connected but not in a live game');
      } else {
        error.set('League client not detected or not connected');
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
    
    if (currentGameState && 'game_mode' in currentGameState) {
      gameMode.set((currentGameState as any).game_mode);
    }
  } catch (err) {
    error.set(err as string);
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
let cleanup: (() => void) | null = null;

export function startListening() {
  if (isListening) return;
  isListening = true;
  
  console.log('Starting live game data listening...');
  
  // Load config first to get the refresh interval, then start everything
  loadConfig().then(() => {
    // Start initial data loading
    loadLiveGameData();
    startGameTimeTicker();
    
    // Set up auto-refresh with the configured interval
    let currentInterval: number;
    refreshInterval.subscribe(interval => {
      currentInterval = interval;
      if (refreshIntervalId) {
        clearInterval(refreshIntervalId);
      }
      if (interval) {
        refreshIntervalId = setInterval(() => loadLiveGameData(true), currentInterval);
      }
    })();
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
  copyToClipboard,
  getSyncStatus,
  getGameModeColor,
  getGamePhaseColor,
  formatGameTime,
  formatEventTime,
  formatEventTimeWithContext
};
