<script lang="ts">
  import { RefreshCw, AlertCircle, Eye, Bug, Wifi, WifiOff } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  
  // WebSocket Live Game Store
  import { 
    wsGameData, 
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
    startWebSocketListening,
    stopWebSocketListening
  } from '$lib/stores/websocketStore';
  
  // Local state for manual refresh
  let isManualRefresh = $state(false);
  let lastUpdateTime = $state(Date.now());
  
  // Debug Store (reuse existing)
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
  import { onMount, onDestroy } from 'svelte';

  // Handle manual refresh
  function handleManualRefresh() {
    isManualRefresh = true;
    lastUpdateTime = Date.now();
    wsActions.refresh();
    
    // Reset manual refresh state after a delay
    setTimeout(() => {
      isManualRefresh = false;
    }, 2000);
  }

  // Enhanced LCU API functions
  async function handleDetectLeagueClient() {
    try {
      const result = await wsActions.detectLeagueClientWithRetry(5);
      console.log('League client detected:', result);
      // Optionally show a success message
    } catch (error) {
      console.error('Failed to detect League client:', error);
    }
  }

  async function handleCheckConnectionHealth() {
    try {
      const isHealthy = await wsActions.checkConnectionHealth();
      console.log('Connection health:', isHealthy);
    } catch (error) {
      console.error('Failed to check connection health:', error);
    }
  }
  
  // Track automatic updates
  $effect(() => {
    if ($wsGameData && !isManualRefresh) {
      lastUpdateTime = Date.now();
    }
  });

  // Start listening when component mounts, stop when it unmounts
  onMount(() => {
    startWebSocketListening();
  });

  onDestroy(() => {
    stopWebSocketListening();
  });

  // WebSocket connection status indicator
  function getConnectionStatus() {
    if ($wsConnecting) return 'connecting';
    if ($wsConnected) return 'connected';
    if ($wsServerRunning) return 'server_running';
    return 'disconnected';
  }

  function getConnectionStatusText() {
    switch (getConnectionStatus()) {
      case 'connecting': return 'Connecting...';
      case 'connected': return 'Connected';
      case 'server_running': return 'Server Running';
      case 'disconnected': return 'Disconnected';
      default: return 'Unknown';
    }
  }

  function getConnectionStatusColor() {
    switch (getConnectionStatus()) {
      case 'connecting': return 'text-yellow-500';
      case 'connected': return 'text-green-500';
      case 'server_running': return 'text-blue-500';
      case 'disconnected': return 'text-red-500';
      default: return 'text-gray-500';
    }
  }
</script>

<div class="min-h-screen bg-gradient-to-br from-background via-background to-muted/20">
  <div class="container mx-auto px-4 py-6 space-y-6 max-w-7xl">
    <!-- Header -->
    <div class="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
      <div class="flex-1">
        <h1 class="text-2xl sm:text-3xl lg:text-4xl font-bold text-foreground bg-gradient-to-r from-foreground to-foreground/80 bg-clip-text">
          Live Game
        </h1>
        <p class="text-muted-foreground mt-1 text-sm sm:text-base">
          Real-time game analysis with WebSocket connection
        </p>
      </div>
      
      <!-- Timer and Controls -->
      <div class="flex flex-col sm:flex-row items-start sm:items-center gap-4">
        <!-- WebSocket Connection Status -->
        <div class="flex items-center gap-3 bg-card/50 backdrop-blur-sm border rounded-xl px-4 py-3 shadow-sm">
          <div class="flex items-center gap-2">
            {#if $wsConnected}
              <Wifi class="w-5 h-5 text-green-500" />
            {:else if $wsConnecting}
              <Wifi class="w-5 h-5 text-yellow-500 animate-pulse" />
            {:else}
              <WifiOff class="w-5 h-5 text-red-500" />
            {/if}
            <span class="text-sm font-medium {getConnectionStatusColor()}">
              {getConnectionStatusText()}
            </span>
          </div>
          {#if $wsServerPort}
            <div class="text-xs text-muted-foreground">
              Port {$wsServerPort}
            </div>
          {/if}
        </div>

        <!-- Game Timer -->
        <div class="flex items-center gap-3 bg-card/50 backdrop-blur-sm border rounded-xl px-4 py-3 shadow-sm">
          <div class="text-xl sm:text-2xl font-mono font-bold flex items-center gap-2">
            {$wsGameTime}
            {#if wsActions.getSyncStatus() === 'synced'}
              <div class="w-2 h-2 bg-green-500 rounded-full animate-pulse" title="Live timer synced with API"></div>
            {:else if wsActions.getSyncStatus() === 'stale'}
              <div class="w-2 h-2 bg-yellow-500 rounded-full" title="API sync stale, using local timer"></div>
            {:else if wsActions.getSyncStatus() === 'local'}
              <div class="w-2 h-2 bg-blue-500 rounded-full" title="Local timer (no API sync)"></div>
            {:else}
              <div class="w-2 h-2 bg-gray-400 rounded-full" title="No live game detected"></div>
            {/if}
          </div>
          <div class="text-xs sm:text-sm text-muted-foreground">
            {wsActions.getSyncStatus() === 'synced' ? 'Synced' : 
             wsActions.getSyncStatus() === 'stale' ? 'Stale' :
             wsActions.getSyncStatus() === 'local' ? 'Local' : 'Offline'}
          </div>
          <!-- Subtle update indicator -->
          <div class="flex items-center gap-1 text-xs text-muted-foreground">
            <div class="w-1.5 h-1.5 bg-green-400 rounded-full {Date.now() - lastUpdateTime < 3000 ? 'animate-pulse' : 'opacity-50'}" title="Last updated: {new Date(lastUpdateTime).toLocaleTimeString()}"></div>
            <span class="hidden sm:inline">Live</span>
          </div>
        </div>
        
        <!-- Action Buttons -->
        <div class="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onclick={handleDetectLeagueClient}
            class="flex items-center gap-2"
            title="Detect League Client with Retry"
          >
            <Wifi class="w-4 h-4" />
            <span class="hidden sm:inline">Detect LCU</span>
          </Button>
          <Button
            variant="outline"
            size="sm"
            onclick={handleCheckConnectionHealth}
            class="flex items-center gap-2"
            title="Check Connection Health"
          >
            <AlertCircle class="w-4 h-4" />
            <span class="hidden sm:inline">Health</span>
          </Button>
          <Button
            variant="outline"
            size="sm"
            onclick={() => {
              console.log('Debug button clicked, opening modal...');
              openDebugModal();
            }}
            class="flex items-center gap-2"
          >
            <Bug class="w-4 h-4" />
            <span class="hidden sm:inline">Debug</span>
          </Button>
          <Button
            size="sm"
            onclick={handleManualRefresh}
            disabled={$wsIsLoading || isManualRefresh}
            class="flex items-center gap-2"
          >
            <RefreshCw class="w-4 h-4 {isManualRefresh ? 'animate-spin' : ''}" />
            <span class="hidden sm:inline">{isManualRefresh ? 'Refreshing...' : 'Refresh'}</span>
          </Button>
        </div>
      </div>
    </div>

    <!-- WebSocket Connection Status Alert -->
    {#if !$wsConnected && !$wsConnecting}
      <Alert variant="destructive">
        <WifiOff class="h-4 w-4" />
        <AlertDescription>
          WebSocket connection is not established. 
          {#if $wsServerRunning}
            Server is running but client is not connected.
          {:else}
            WebSocket server is not running.
          {/if}
        </AlertDescription>
      </Alert>
    {/if}

    {#if $wsError && !$wsIsSpectatorMode}
      <Alert variant="destructive">
        <AlertCircle class="h-4 w-4" />
        <AlertDescription>Error: {$wsError}</AlertDescription>
      </Alert>
    {/if}

    <!-- Spectator Mode Notice -->
    {#if $wsIsSpectatorMode}
      <Alert class="border-orange-200 bg-orange-50 dark:border-orange-800 dark:bg-orange-950">
        <Eye class="h-4 w-4 text-orange-600" />
        <AlertDescription class="text-orange-800 dark:text-orange-200">
          <div class="space-y-2">
            <div class="font-semibold">Spectator Mode Detected</div>
            <div class="text-sm">
              You are currently spectating a game. Available features:
            </div>
            <ul class="text-sm list-disc list-inside space-y-1 ml-2">
              <li>Live game statistics and events</li>
              <li>Team comparison and player data</li>
              <li>Real-time game timeline</li>
            </ul>
            <div class="text-sm text-orange-700 dark:text-orange-300">
              Personal recommendations and utilities are not available in spectator mode.
            </div>
          </div>
        </AlertDescription>
      </Alert>
    {/if}

    <!-- Debug Modal -->
    {#if $isDebugModalOpen}
      <DebugModal 
        isOpen={$isDebugModalOpen}
        gameState={$wsGameState}
        gameData={$wsGameData}
        liveGameStats={$wsLiveGameStats}
        playerScores={$wsPlayerScores}
        isSpectatorMode={$wsIsSpectatorMode}
        copiedStates={{}}
        onClose={closeDebugModal}
        onCopyToClipboard={async () => {}}
        onDebugLcuConnection={wsActions.debugConnection}
      />
    {/if}

    <!-- Connection Health Card -->
    <ConnectionHealthCard />

    <!-- Show game data if we have valid game data (either live game or spectator mode) -->
    {#if $wsGameData && $wsGameData.allPlayers && Array.isArray($wsGameData.allPlayers) && $wsGameData.allPlayers.length > 0 && !$wsError}
    
    <!-- Manual Refresh Loading Overlay -->
    {#if isManualRefresh}
      <div class="fixed top-4 right-4 z-50 bg-background/90 backdrop-blur-sm border rounded-lg px-3 py-2 shadow-lg">
        <div class="flex items-center gap-2 text-sm">
          <RefreshCw class="w-4 h-4 animate-spin text-primary" />
          <span class="text-muted-foreground">Refreshing data...</span>
        </div>
      </div>
    {/if}

    <!-- Game Status -->
    <WebSocketGameStatus />

    <!-- Active Player Info - Hidden in spectator mode -->
    {#if !$wsIsSpectatorMode}
      <WebSocketActivePlayerInfo />
    {/if}

    <!-- Responsive Layout: Events First on Mobile, Sidebar on Desktop -->
    <div class="space-y-6">
      <!-- Events Timeline - Always visible, prominent on mobile -->
      <div class="lg:hidden">
        <WebSocketEventsTimeline />
      </div>

      <!-- Desktop Two Column Layout -->
      <div class="hidden lg:grid lg:grid-cols-3 gap-6">
        <!-- Main Content Column -->
        <div class="lg:col-span-2 space-y-6">
          <!-- Team Comparison -->
          <WebSocketTeamComparison />

          <!-- Live Game Stats -->
          <LiveStats />

          <!-- Item Tracking - Hidden in spectator mode -->
          {#if !$wsIsSpectatorMode}
            <ItemTracking />
          {/if}

          <!-- Match History Analysis - Hidden in spectator mode -->
          {#if !$wsIsSpectatorMode}
            <MatchHistoryAnalysis />
          {/if}

          <!-- Rune Analysis - Hidden in spectator mode -->
          {#if !$wsIsSpectatorMode}
            <RuneAnalysis />
          {/if}

          <!-- Lobby Management - Hidden in spectator mode -->
          {#if !$wsIsSpectatorMode}
            <LobbyManagement />
          {/if}

          <!-- AI Recommendations - Hidden in spectator mode -->
          {#if !$wsIsSpectatorMode}
            <AIRecommendations />
          {/if}
        </div>

        <!-- Events Sidebar Column -->
        <div class="lg:col-span-1">
          <!-- Game Events Timeline -->
          <WebSocketEventsTimeline />
        </div>
      </div>

      <!-- Mobile/Tablet Single Column Layout -->
      <div class="lg:hidden space-y-6">
        <!-- Team Comparison -->
        <WebSocketTeamComparison />

        <!-- Live Game Stats -->
        <LiveStats />

        <!-- Item Tracking - Hidden in spectator mode -->
        {#if !$wsIsSpectatorMode}
          <ItemTracking />
        {/if}

        <!-- AI Recommendations - Hidden in spectator mode -->
        {#if !$wsIsSpectatorMode}
          <AIRecommendations />
        {/if}
      </div>
    </div>

    {/if}
    <!-- End of game data sections - only shown when client is connected -->

    <!-- No Live Game Message -->
    {#if !$wsGameData?.allPlayers || $wsGameData.allPlayers.length === 0 || $wsError}
      <NoLiveGameMessage 
        error={$wsError}
        isLoading={$wsIsLoading}
        isRefreshing={isManualRefresh}
        onRetry={handleManualRefresh}
        onViewDebug={openDebugModal}
      />
    {/if}
  </div>
</div>
