<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { RefreshCw, AlertCircle, Eye, Bug, Wifi, WifiOff, Gamepad2, Users, Target, Zap, Clock } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
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
    wsLCUConnecting,
    startWebSocketListening,
    stopWebSocketListening
  } from '$lib/stores/websocketStore';
  
  // Local state
  let isManualRefresh = $state(false);
  let lastUpdateTime = $state(Date.now());
  let currentGameState = $state('unknown');
  let gamePhase = $state('unknown');
  let isConnected = $state(false);
  let isLCUConnecting = $state(false);
  let connectionStatus = $state('Disconnected');
  
  // Game data
  let gameData: any = $state(null);
  let championSelectData: any = $state(null);
  let liveGameStats: any = $state(null);
  let playerScores: any = $state(null);
  
  // Debug Store
  import { isDebugModalOpen, openDebugModal, closeDebugModal } from '$lib/stores/debugStore';
  
  // WebSocket Live Game Components
  import WebSocketGameStatus from '$lib/components/live-game/WebSocketGameStatus.svelte';
  import WebSocketTeamComparison from '$lib/components/live-game/WebSocketTeamComparison.svelte';
  import WebSocketEventsTimeline from '$lib/components/live-game/WebSocketEventsTimeline.svelte';
  import WebSocketActivePlayerInfo from '$lib/components/live-game/WebSocketActivePlayerInfo.svelte';
  import LiveStats from '$lib/components/live-game/LiveStats.svelte';
  import AIRecommendations from '$lib/components/live-game/AIRecommendations.svelte';
  import ItemTracking from '$lib/components/live-game/ItemTracking.svelte';
  import MatchHistoryAnalysis from '$lib/components/live-game/MatchHistoryAnalysis.svelte';
  import RuneAnalysis from '$lib/components/live-game/RuneAnalysis.svelte';
  import LobbyManagement from '$lib/components/live-game/LobbyManagement.svelte';
  import DebugModal from '$lib/components/live-game/DebugModal.svelte';
  import NoLiveGameMessage from '$lib/components/live-game/NoLiveGameMessage.svelte';
  import ConnectionHealthCard from '$lib/components/live-game/ConnectionHealthCard.svelte';
  
  // Champion Select Components
  import ChampionSelectOverlay from '$lib/components/champion-select/ChampionSelectOverlay.svelte';
  
  // Reactive updates
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
    
    // Determine current game state based on comprehensive game state
    if (gameData) {
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
      } else if (gameData.champion_select && !gameData.champion_select.isNull) {
        currentGameState = 'champion-select';
        connectionStatus = 'In Champion Select';
      } else if (gameData.queue && gameData.queue.in_queue) {
        currentGameState = 'queue';
        connectionStatus = 'In Queue';
      } else if (gameData.lobby && !gameData.lobby.isNull) {
        currentGameState = 'lobby';
        connectionStatus = 'In Lobby';
      } else {
        // Fallback: Check if we have game time updates (indicates live game)
        if ($wsGameTime && Number($wsGameTime) > 0) {
          currentGameState = 'live-game';
          connectionStatus = $wsIsSpectatorMode ? 'Spectating Game' : 'In Game';
        } else {
          currentGameState = 'idle';
          connectionStatus = isConnected ? 'Connected' : 'Disconnected';
        }
      }
    } else {
      // Fallback: Check if we have game time updates (indicates live game)
      if ($wsGameTime && Number($wsGameTime) > 0) {
        console.log('[Game State] Fallback detection: Game time detected, setting live-game state');
        currentGameState = 'live-game';
        connectionStatus = $wsIsSpectatorMode ? 'Spectating Game' : 'In Game';
      } else {
        console.log('[Game State] No game data or time detected, setting idle state');
        currentGameState = 'idle';
        connectionStatus = isConnected ? 'Connected' : 'Disconnected';
      }
    }
  });

  onMount(() => {
    // Start WebSocket listening
    startWebSocketListening();
    
    // Initial connection attempt
    wsActions.detectLeagueClientWithRetry(3).catch(error => {
      console.error('Failed to detect League client:', error);
    });
    
    return () => {
      stopWebSocketListening();
    };
  });

  async function handleRefresh() {
    isManualRefresh = true;
    lastUpdateTime = Date.now();
    
    try {
      // Try to detect League client
      await wsActions.detectLeagueClientWithRetry(3);
      
      // Check connection health
      await wsActions.checkConnectionHealth();
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
        return 'Game Assistant';
    }
  }

  function getPageSubtitle(): string {
    switch (currentGameState) {
      case 'champion-select':
        return 'Pick and ban recommendations with real-time analysis';
      case 'live-game':
        return 'Real-time game analysis with WebSocket connection';
      case 'queue':
        return 'Waiting for matchmaking - prepare for champion select';
      case 'lobby':
        return 'Lobby management and team coordination';
      default:
        return 'Connect to League of Legends for assistance';
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
  <!-- Dynamic Header -->
  <Card class="border-b border-border rounded-none">
    <CardHeader class="pb-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg bg-primary flex items-center justify-center">
            {#if getStatusIcon()}
              {@const StatusIcon = getStatusIcon()}
              <StatusIcon class="w-5 h-5 text-primary-foreground" />
            {/if}
          </div>
          <div>
            <CardTitle class="text-xl">{getPageTitle()}</CardTitle>
            <p class="text-sm text-muted-foreground">{getPageSubtitle()}</p>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <Badge variant="outline" class={getStatusColor()}>
            {connectionStatus}
          </Badge>
          <Badge variant={wsConnected ? 'default' : 'destructive'}>
            {wsConnected ? 'Connected' : 'Disconnected'}
          </Badge>
          <Button 
            variant="ghost" 
            size="sm" 
            onclick={handleDetectLeagueClient}
            disabled={isLCUConnecting}
          >
            <RefreshCw class="w-4 h-4" />
          </Button>
          <Button 
            variant="ghost" 
            size="sm" 
            onclick={handleCheckConnectionHealth}
          >
            <Wifi class="w-4 h-4" />
          </Button>
          <Button 
            variant="ghost" 
            size="sm" 
            onclick={openDebugModal}
          >
            <Bug class="w-4 h-4" />
          </Button>
        </div>
      </div>
    </CardHeader>
  </Card>

  <!-- Connection Health -->
  <ConnectionHealthCard />

  <!-- Debug Information -->
  {#if currentGameState === 'idle' && isConnected}
    <div class="p-4">
      <Alert>
        <AlertCircle class="w-4 h-4" />
        <AlertDescription>
          <strong>Debug Info:</strong> Connected but no game data detected. 
          Game Data: {gameData ? 'Present' : 'Missing'}, 
          Spectator Mode: {$wsIsSpectatorMode ? 'Yes' : 'No'},
          Game Phase: {$wsGamePhase}
          {#if gameData}
            <br><strong>Data Structure:</strong> 
            activePlayer: {gameData.activePlayer ? 'Yes' : 'No'}, 
            allPlayers: {gameData.allPlayers ? 'Yes' : 'No'}, 
            gameData: {gameData.gameData ? 'Yes' : 'No'}, 
            events: {gameData.events ? 'Yes' : 'No'}, 
            gameTime: {gameData.gameTime !== undefined ? 'Yes' : 'No'}
          {/if}
        </AlertDescription>
      </Alert>
    </div>
  {/if}

  <!-- Dynamic Content Based on Game State -->
  {#if currentGameState === 'champion-select'}
    <!-- Champion Select Mode -->
    <div class="flex-1 p-4">
      <ChampionSelectOverlay />
    </div>
    
  {:else if currentGameState === 'queue'}
    <!-- Queue Mode -->
    <div class="flex-1 p-4">
      <div class="max-w-4xl mx-auto">
        <Card>
          <CardHeader>
            <CardTitle class="flex items-center gap-2">
              <Clock class="w-5 h-5" />
              In Queue
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div class="text-center py-8">
              <div class="w-16 h-16 rounded-full bg-yellow-500/10 flex items-center justify-center mx-auto mb-4">
                <Clock class="w-8 h-8 text-yellow-500" />
              </div>
              <h3 class="text-lg font-semibold mb-2">Waiting for Matchmaking</h3>
              <p class="text-muted-foreground mb-4">
                You're currently in queue. Prepare for champion select by reviewing your champion pool and strategies.
              </p>
              {#if gameData?.queue}
                <Badge variant="outline" class="mb-4">
                  Queue ID: {gameData.queue.queue_id}
                </Badge>
              {/if}
              <div class="flex flex-wrap gap-2 justify-center">
                <Button variant="outline" onclick={() => window.location.href = '/ai-chat'}>
                  <Zap class="w-4 h-4 mr-2" />
                  Get AI Advice
                </Button>
                <Button variant="outline" onclick={() => window.location.href = '/settings'}>
                  <Users class="w-4 h-4 mr-2" />
                  Review Settings
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
    
  {:else if currentGameState === 'live-game'}
    <!-- Live Game Mode -->
    <div class="flex-1 p-4 space-y-4">
      <!-- Spectator Mode Indicator -->
      {#if $wsIsSpectatorMode}
        <Alert class="border-blue-500/30 bg-blue-500/10">
          <Eye class="w-4 h-4" />
          <AlertDescription>
            <strong>Spectator Mode:</strong> You're currently spectating a game. Some features may be limited.
          </AlertDescription>
        </Alert>
      {/if}
      
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <!-- Game Status and Team Comparison -->
        <div class="space-y-4">
          <WebSocketGameStatus />
          <WebSocketTeamComparison />
        </div>
        
        <!-- Active Player Info and Events -->
        <div class="space-y-4">
          <WebSocketActivePlayerInfo />
          <WebSocketEventsTimeline />
        </div>
      </div>
      
      <!-- Live Stats and AI Recommendations -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <LiveStats />
        <AIRecommendations />
      </div>
      
      <!-- Item Tracking and Analysis -->
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <ItemTracking />
        <MatchHistoryAnalysis />
        <RuneAnalysis />
      </div>
    </div>
    
  {:else if currentGameState === 'lobby'}
    <!-- Lobby Mode -->
    <div class="flex-1 p-4">
      <LobbyManagement />
    </div>
    
  {:else}
    <!-- Idle/Disconnected State -->
    <div class="flex-1 p-4">
      <NoLiveGameMessage 
        error={$wsError}
        isLoading={$wsIsLoading}
        isRefreshing={isManualRefresh}
        onRetry={handleRefresh}
        onViewDebug={openDebugModal}
      />
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
