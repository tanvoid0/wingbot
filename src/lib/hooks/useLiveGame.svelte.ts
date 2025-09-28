import { invoke } from '@tauri-apps/api/core';
import { onMount } from 'svelte';
import type { 
  ComprehensiveGameState, 
  LiveGameData, 
  GameStats, 
  PlayerScoresData
} from '$lib/types/game';

export function useLiveGame() {
  // Data state
  let gameData = $state<LiveGameData | null>(null);
  let liveGameStats = $state<GameStats | null>(null);
  let playerScores = $state<PlayerScoresData | null>(null);
  let gameState = $state<ComprehensiveGameState | null>(null);
  
  // UI state
  let isLoading = $state(true);
  let isRefreshing = $state(false);
  let error = $state<string | null>(null);
  
  // Game info
  let gameTime = $state<string>('00:00');
  let gamePhase = $state<string>('Unknown');
  let gameMode = $state<string>('Unknown');
  let isSpectatorMode = $state(false);
  
  // Timer state
  let gameStartTime = $state<number | null>(null);
  let lastApiGameTime = $state<number | null>(null);
  let lastApiSyncTime = $state<number | null>(null);
  let gameTimeInterval: number | null = null;
  
  // Settings
  let refreshInterval = $state(1000); // Default to 1 second (1000ms)
  
  // Debug state
  let copiedStates = $state<Record<string, boolean>>({});

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
      const currentTime = Date.now();
      const elapsedSeconds = Math.floor((currentTime - gameStartTime) / 1000);
      const minutes = Math.floor(elapsedSeconds / 60);
      const seconds = elapsedSeconds % 60;
      gameTime = `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
    }
  }
  
  function syncGameTimeWithApi(apiGameTime: number) {
    // If this is the first API time or if the API time is significantly different from our local time
    if (lastApiGameTime === null || Math.abs(apiGameTime - lastApiGameTime) > 2) {
      // Sync our local timer with the API time
      gameStartTime = Date.now() - (apiGameTime * 1000);
      lastApiGameTime = apiGameTime;
      lastApiSyncTime = Date.now();
      console.log(`Synced game time with API: ${apiGameTime}s`);
    } else {
      // Update the last API time to track sync status
      lastApiGameTime = apiGameTime;
      lastApiSyncTime = Date.now();
    }
  }
  
  function getSyncStatus() {
    if (!gameStartTime) return 'disconnected';
    if (!lastApiSyncTime) return 'local';
    
    const timeSinceLastSync = Date.now() - lastApiSyncTime;
    if (timeSinceLastSync < 10000) { // Less than 10 seconds
      return 'synced';
    } else if (timeSinceLastSync < 30000) { // Less than 30 seconds
      return 'stale';
    } else {
      return 'local';
    }
  }

  // Utility functions
  function getGamePhaseColor() {
    switch (gamePhase) {
      case 'Early': return 'bg-green-500';
      case 'Mid': return 'bg-yellow-500';
      case 'Late': return 'bg-red-500';
      default: return 'bg-gray-500';
    }
  }

  function getGameModeColor() {
    switch (gameMode) {
      case 'Arena': return 'bg-purple-500';
      case 'Classic': return 'bg-blue-500';
      case 'ARAM': return 'bg-orange-500';
      case 'URF': return 'bg-pink-500';
      default: return 'bg-gray-500';
    }
  }

  function formatGameTime(seconds: number) {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }

  // Data loading function
  async function loadLiveGameData(isRefresh = false) {
    try {
      if (isRefresh) {
        isRefreshing = true;
      } else {
        isLoading = true;
      }
      error = null;
      
      console.log('Loading live game data...');
      
      // Try to get comprehensive game state first
      const comprehensiveState = await invoke('get_comprehensive_game_state') as ComprehensiveGameState;
      console.log('Comprehensive state:', comprehensiveState);
      gameState = comprehensiveState;
      
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
          
          gameData = game;
          liveGameStats = stats;
          playerScores = scores;
          
          // Extract game time and mode from all game data
          if (allGameData && typeof allGameData === 'object' && 'gameData' in allGameData) {
            const gameData = allGameData as { gameData: { gameTime?: number; gameMode?: string } };
            if (gameData.gameData?.gameTime !== undefined) {
              // Sync our local timer with the API time
              syncGameTimeWithApi(gameData.gameData.gameTime);
            }
            
            if (gameData.gameData?.gameMode) {
              gameMode = gameData.gameData.gameMode;
            }
          }
          
          // Check if we're in spectator mode
          if (allGameData && typeof allGameData === 'object' && 'activePlayer' in allGameData) {
            const activePlayer = (allGameData as any).activePlayer;
            isSpectatorMode = activePlayer?.error === "Spectator mode doesn't currently support this feature";
          }
          
          gamePhase = 'In Game';
        } catch (apiError) {
          // If individual APIs fail, show error but don't crash
          console.warn('Some live game APIs failed:', apiError);
          error = `Some game data unavailable: ${typeof apiError === 'string' ? apiError : JSON.stringify(apiError)}`;
        }
      } else {
        // No live client available
        console.log('Live client not available');
        gameData = null;
        liveGameStats = null;
        playerScores = null;
        gamePhase = 'No Live Game';
        gameMode = 'Unknown';
        gameTime = '00:00';
        gameStartTime = null;
        lastApiGameTime = null;
        lastApiSyncTime = null;
        
        // Check if LCU is connected but no live game
        if (comprehensiveState?.lcu_connected) {
          error = 'League client connected but not in a live game';
        } else {
          error = 'League client not detected or not connected';
        }
      }
      
      // Update phase and mode from available data (game time is handled by ticker)
      if (gameData && 'phase' in gameData) {
        gamePhase = (gameData as any).phase;
      }
      if (gameState?.game_mode) {
        gameMode = gameState.game_mode;
      }
    } catch (err) {
      error = err as string;
      console.error('Failed to load live game data:', err);
      // Clear all data on error
      gameData = null;
      liveGameStats = null;
      playerScores = null;
      gamePhase = 'Error';
      gameMode = 'Unknown';
      gameTime = '00:00';
      gameStartTime = null;
      lastApiGameTime = null;
      lastApiSyncTime = null;
    } finally {
      if (isRefresh) {
        isRefreshing = false;
      } else {
        isLoading = false;
      }
    }
  }

  // Config loading function
  async function loadConfig() {
    try {
      const config = await invoke('get_config') as any;
      if (config?.ui?.refresh_interval) {
        // Convert from milliseconds to milliseconds (already in correct format)
        refreshInterval = config.ui.refresh_interval;
        console.log(`Loaded refresh interval from config: ${refreshInterval}ms`);
      } else {
        console.log('Using default refresh interval: 1000ms');
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
      copiedStates[key] = true;
      setTimeout(() => {
        copiedStates[key] = false;
      }, 2000);
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
    }
  }

  // Initialize on mount
  onMount(() => {
    let interval: number | null = null;
    
    // Load config first to get the refresh interval, then start everything
    loadConfig().then(() => {
      // Start initial data loading
      loadLiveGameData();
      startGameTimeTicker();
      
      // Set up auto-refresh with the configured interval
      interval = setInterval(() => loadLiveGameData(true), refreshInterval);
    });
    
    return () => {
      if (interval) {
        clearInterval(interval);
      }
      stopGameTimeTicker();
    };
  });

  return {
    // Data
    gameData,
    liveGameStats,
    playerScores,
    gameState,
    
    // UI State
    isLoading,
    isRefreshing,
    error,
    
    // Game Info
    gameTime,
    gamePhase,
    gameMode,
    isSpectatorMode,
    
    // Timer State
    gameStartTime,
    lastApiGameTime,
    lastApiSyncTime,
    
    // Settings
    refreshInterval,
    
    // Debug State
    copiedStates,
    
    // Actions
    refresh,
    copyToClipboard,
    getSyncStatus,
    getGameModeColor,
    getGamePhaseColor,
    formatGameTime
  };
}
