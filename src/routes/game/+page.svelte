<script lang="ts">
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { invoke } from '@tauri-apps/api/core';
  import { RefreshCw, Eye, Bug, Wifi, Gamepad2, Users, Target, Clock } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  // WebSocket Live Game Store
  import { 
    wsGameData,
    wsLiveGameData, 
    wsLiveGameStats, 
    wsPlayerScores, 
    wsGameState,
    wsIsLoading, 
    wsError,
    wsErrorStable,
    wsGameTime,
    wsGameSeconds,
    wsGamePhase, 
    wsGameMode, 
    wsIsSpectatorMode,
    wsConnected,
    wsConnecting,
    wsServerRunning,
    wsServerPort,
    wsActions,
    wsSyncStatus,
    wsChampionSelectData,
    wsIsChampionSelectActive,
    wsLCUConnecting,
    isPostChampSelectPhase,
    isInMatchOrLoadingPhase,
  } from '$lib/stores/websocketStore';
  
  function matchLoadingStatus(phase: string): string {
    return phase === 'GameStart' ? 'Loading game…' : 'In game (waiting for live data)';
  }

  // Local state
  let isManualRefresh = $state(false);
  let lastUpdateTime = $state(Date.now());
  let currentGameState = $state('unknown');
  let gamePhase = $state('unknown');
  let isConnected = $state(false);
  let isLCUConnecting = $state(false);
  let connectionStatus = $state('Disconnected');
  let effectivePhase = $state('unknown');
  let loadingIntoGame = $state(false);
  
  // Game data
  let gameData: any = $state(null);
  let championSelectData: any = $state(null);
  let liveGameStats: any = $state(null);
  let playerScores: any = $state(null);
  
  // Debug Store
  import { isDebugModalOpen, openDebugModal, closeDebugModal } from '$lib/stores/debugStore';
  import { openLauncherModal } from '$lib/stores/appModalsStore';
  
  // WebSocket Live Game Components
  import WebSocketGameStatus from '$lib/components/live-game/WebSocketGameStatus.svelte';
  import WebSocketTeamComparison from '$lib/components/live-game/WebSocketTeamComparison.svelte';
  import WebSocketEventsTimeline from '$lib/components/live-game/WebSocketEventsTimeline.svelte';
  import WebSocketActivePlayerInfo from '$lib/components/live-game/WebSocketActivePlayerInfo.svelte';
  import LiveStats from '$lib/components/live-game/LiveStats.svelte';
  import EnemyDeathTimers from '$lib/components/live-game/EnemyDeathTimers.svelte';
  import AIRecommendations from '$lib/components/live-game/AIRecommendations.svelte';
  import ItemTracking from '$lib/components/live-game/ItemTracking.svelte';
  import MatchHistoryAnalysis from '$lib/components/live-game/MatchHistoryAnalysis.svelte';
  import RuneAnalysis from '$lib/components/live-game/RuneAnalysis.svelte';
  import LobbyManagement from '$lib/components/live-game/LobbyManagement.svelte';
  import SpectatorTeamPanel from '$lib/components/live-game/SpectatorTeamPanel.svelte';
  import DebugModal from '$lib/components/live-game/DebugModal.svelte';
  import NoLiveGameMessage from '$lib/components/live-game/NoLiveGameMessage.svelte';
  import { appStatusBarContext, clearAppStatusBarPageState } from '$lib/stores/appStatusBarStore';
  
  // Champion Select Components
  import ChampionSelectOverlay from '$lib/components/champion-select/ChampionSelectOverlay.svelte';
  import EnemyScoutPanel from '$lib/components/champion-select/EnemyScoutPanel.svelte';
  
  $effect(() => {
    gameData = $wsGameData || $wsLiveGameData;
    championSelectData = $wsChampionSelectData;
    liveGameStats = $wsLiveGameStats;
    playerScores = $wsPlayerScores;
    gamePhase = $wsGamePhase;
    isConnected = $wsConnected;
    isLCUConnecting = $wsLCUConnecting;
    
    // Debug logging for WebSocket data
    console.log('[Game Page] WebSocket data update:', {
      wsGameData: $wsGameData,
      wsLiveGameData: $wsLiveGameData,
      wsGameTime: $wsGameTime,
      wsIsSpectatorMode: $wsIsSpectatorMode,
      wsGamePhase: $wsGamePhase,
      wsConnected: $wsConnected
    });
    
    // Champion select wins unless LCU phase says the game has already started
    effectivePhase =
      ($wsGameState?.game_phase as string | undefined) ?? $wsGamePhase;
    const hasChampSelectSession =
      $wsIsChampionSelectActive || !!$wsChampionSelectData;
    loadingIntoGame =
      !hasChampSelectSession && isInMatchOrLoadingPhase(effectivePhase);
    const inChampSelect =
      !isPostChampSelectPhase(effectivePhase) && hasChampSelectSession;

    if (inChampSelect) {
      currentGameState = 'champion-select';
      connectionStatus = 'In Champion Select';
    } else if (loadingIntoGame) {
      currentGameState = 'live-game';
      connectionStatus = matchLoadingStatus(effectivePhase);
    } else if (gameData) {
      // Debug logging for game state detection
      console.log('[Game State] Checking gameData:', {
        hasActivePlayer: !!gameData.activePlayer,
        hasAllPlayers: !!gameData.allPlayers,
        hasGameData: !!gameData.gameData,
        hasEvents: !!gameData.events,
        gameTime: gameData.gameTime,
        gameDataGameTime: gameData.gameData?.gameTime,
        spectatorMode: $wsIsSpectatorMode,
        gamePhase: $wsGamePhase
      });
      
      // Check for live game data first - in spectator mode, we might have different data structure
      if (gameData.activePlayer || gameData.allPlayers || gameData.gameData || 
          gameData.events || gameData.gameTime !== undefined || 
          (gameData.gameData && gameData.gameData.gameTime !== undefined)) {
        console.log('[Game State] Primary detection: Full game data found, setting live-game state');
        currentGameState = 'live-game';
        connectionStatus = $wsIsSpectatorMode ? 'Spectating Game' : 'In Game';
      } else if (isPostChampSelectPhase(effectivePhase)) {
        currentGameState = 'live-game';
        connectionStatus = matchLoadingStatus(effectivePhase);
      } else if (gameData.queue && gameData.queue.in_queue) {
        currentGameState = 'queue';
        connectionStatus = 'In Queue';
      } else if (gameData.lobby && !gameData.lobby.isNull) {
        currentGameState = 'lobby';
        connectionStatus = 'In Lobby';
      } else {
        // Fallback: Check if we have game time updates (indicates live game)
        if ($wsGameSeconds > 0) {
          currentGameState = 'live-game';
          connectionStatus = $wsIsSpectatorMode ? 'Spectating Game' : 'In Game';
        } else {
          currentGameState = 'idle';
          connectionStatus = isConnected ? 'Connected' : 'Disconnected';
        }
      }
    } else {
      // Fallback: Check if we have game time updates (indicates live game)
      if ($wsGameSeconds > 0) {
        console.log('[Game State] Fallback detection: Game time detected, setting live-game state');
        currentGameState = 'live-game';
        connectionStatus = $wsIsSpectatorMode ? 'Spectating Game' : 'In Game';
      } else {
        console.log('[Game State] No game data or time detected, setting idle state');
        if (isPostChampSelectPhase(effectivePhase) || loadingIntoGame) {
          currentGameState = 'live-game';
          connectionStatus = matchLoadingStatus(effectivePhase);
        } else if ($wsChampionSelectData) {
          currentGameState = 'champion-select';
          connectionStatus = 'In Champion Select';
        } else {
          currentGameState = 'idle';
          connectionStatus = isConnected ? 'Connected' : 'Disconnected';
        }
      }
    }

    appStatusBarContext.set({
      title: getPageTitle(),
      badge: connectionStatus,
      badgeClass: getStatusColor(),
    });
  });

  onMount(() => {
    let poll: ReturnType<typeof setInterval> | undefined;

    void (async () => {
      try {
        await wsActions.detectLeagueClientWithRetry(3);
        await wsActions.refreshGameStateFromBackend();
      } catch (error) {
        console.error('Failed to detect League client:', error);
      }
    })();

    poll = setInterval(() => {
      if (!get(wsConnected)) return;
      // Keep polling until live game data arrives (Live Client API on port 2999)
      if (get(wsLiveGameData)) return;
      wsActions.refreshGameStateFromBackend().catch(() => {});
    }, 2000);

    return () => {
      if (poll !== undefined) clearInterval(poll);
      clearAppStatusBarPageState();
    };
  });

  async function handleRefresh() {
    isManualRefresh = true;
    lastUpdateTime = Date.now();
    
    try {
      await wsActions.detectLeagueClientWithRetry(3);
      await wsActions.checkConnectionHealth();
      await wsActions.refreshGameStateFromBackend();
    } catch (error) {
      console.error('Refresh failed:', error);
    } finally {
      isManualRefresh = false;
    }
  }

  async function handleDetectLeagueClient() {
    try {
      await wsActions.detectLeagueClientWithRetry(3);
    } catch (error) {
      console.error('Failed to detect League client:', error);
    }
  }

  async function handleCheckConnectionHealth() {
    try {
      await wsActions.checkConnectionHealth();
    } catch (error) {
      console.error('Failed to check connection health:', error);
    }
  }

  function getPageTitle(): string {
    switch (currentGameState) {
      case 'champion-select':
        return 'Champion Select';
      case 'live-game':
        return 'Live Game';
      case 'queue':
        return 'In Queue';
      case 'lobby':
        return 'Lobby';
      default:
        return 'Game';
    }
  }

  function getStatusIcon() {
    switch (currentGameState) {
      case 'champion-select':
        return Target;
      case 'live-game':
        return Gamepad2;
      case 'queue':
        return Clock;
      case 'lobby':
        return Users;
      default:
        return Wifi;
    }
  }

  function getStatusColor(): string {
    if (isConnected) {
      switch (currentGameState) {
        case 'champion-select':
          return 'bg-blue-500/20 text-blue-600 border-blue-500/30';
        case 'live-game':
          return 'bg-green-500/20 text-green-600 border-green-500/30';
        case 'queue':
          return 'bg-yellow-500/20 text-yellow-600 border-yellow-500/30';
        case 'lobby':
          return 'bg-purple-500/20 text-purple-600 border-purple-500/30';
        default:
          return 'bg-gray-500/20 text-gray-600 border-gray-500/30';
      }
    }
    return 'bg-red-500/20 text-red-600 border-red-500/30';
  }
</script>

<div class="flex flex-col h-full bg-background">
  <!-- Dynamic Content Based on Game State -->
  {#if currentGameState === 'champion-select'}
    <div class="flex-1 p-3 space-y-3">
      <EnemyScoutPanel />
      <ChampionSelectOverlay />
    </div>

  {:else if currentGameState === 'queue'}
    <div class="flex-1 p-3">
      <div class="max-w-lg mx-auto">
        <Card>
          <CardContent class="py-4 text-center">
            <Clock class="w-8 h-8 text-yellow-500 mx-auto mb-2" />
            <h3 class="font-semibold mb-1">In Queue</h3>
            <p class="text-sm text-muted-foreground mb-3">
              Waiting for matchmaking.
            </p>
            {#if gameData?.queue}
              <Badge variant="outline" class="text-xs">
                Queue {gameData.queue.queue_id}
              </Badge>
            {/if}
          </CardContent>
        </Card>
      </div>
    </div>

  {:else if currentGameState === 'live-game'}
    <div class="flex-1 p-3 space-y-3">
      {#if !gameData && (isPostChampSelectPhase(effectivePhase) || loadingIntoGame)}
        <Alert class="py-2 border-yellow-500/30 bg-yellow-500/10">
          <Clock class="w-4 h-4" />
          <AlertDescription class="text-sm">
            {#if effectivePhase === 'GameStart' || loadingIntoGame}
              Loading into game… Live stats appear once the match starts.
            {:else}
              In game, but Live Client Data (port 2999) is not responding yet. If this persists, enable
              <strong>Live Client Data</strong> in League client settings (Settings → General).
            {/if}
          </AlertDescription>
        </Alert>
      {/if}

      {#if effectivePhase === 'GameStart' || loadingIntoGame || $wsGameSeconds < 180}
        <EnemyScoutPanel mode="loading" hideWhenEmpty={true} />
      {/if}

      {#if $wsIsSpectatorMode}
        <Alert class="py-2 border-blue-500/30 bg-blue-500/10">
          <Eye class="w-4 h-4" />
          <AlertDescription class="text-sm">
            Spectator mode — live stats are limited; roster data comes from Riot Spectator API.
          </AlertDescription>
        </Alert>
        <SpectatorTeamPanel />
      {/if}

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-3">
        <div class="space-y-3">
          <WebSocketGameStatus />
          <WebSocketTeamComparison />
        </div>

        <div class="space-y-3">
          <WebSocketActivePlayerInfo />
          <EnemyDeathTimers />
          <WebSocketEventsTimeline />
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-3">
        <LiveStats />
        <AIRecommendations />
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
        <ItemTracking />
        <MatchHistoryAnalysis />
        <RuneAnalysis />
      </div>
    </div>
    
  {:else if currentGameState === 'lobby'}
    <div class="flex-1 p-3 space-y-3">
      <LobbyManagement />
      <MatchHistoryAnalysis />
    </div>

  {:else}
    <div class="flex-1 p-3 space-y-3">
      <NoLiveGameMessage 
        error={$wsError}
        isLoading={$wsIsLoading}
        isRefreshing={isManualRefresh}
        onRetry={handleRefresh}
        onViewDebug={openDebugModal}
        onLaunchLeague={openLauncherModal}
      />
      <MatchHistoryAnalysis />
    </div>
  {/if}

  <!-- Debug Modal -->
  <DebugModal 
    isOpen={$isDebugModalOpen}
    gameState={gameData}
    gameData={gameData}
    liveGameStats={liveGameStats}
    playerScores={playerScores}
    isSpectatorMode={$wsIsSpectatorMode}
    copiedStates={{}}
    onClose={closeDebugModal}
    onCopyToClipboard={async (data, key) => {
      try {
        await navigator.clipboard.writeText(JSON.stringify(data, null, 2));
        console.log(`Copied ${key} to clipboard`);
      } catch (error) {
        console.error('Failed to copy to clipboard:', error);
      }
    }}
    onDebugLcuConnection={async () => {
      try {
        return await wsActions.debugConnection();
      } catch (error) {
        console.error('Failed to debug LCU connection:', error);
        return {
          wingbot_websocket: null,
          lcu: { success: false, error: error instanceof Error ? error.message : String(error) },
        };
      }
    }}
  />
</div>
