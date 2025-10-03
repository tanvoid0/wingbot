<script lang="ts">
  import { RefreshCw, AlertCircle, Eye, Bug } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  
  // Live Game Store
  import { 
    gameData, 
    liveGameStats, 
    playerScores, 
    gameState,
    isLoading, 
    isRefreshing, 
    error,
    errorStable,
    gameTime, 
    gamePhase, 
    gameMode, 
    isSpectatorMode,
    copiedStates,
    liveGameActions,
    startListening,
    stopListening
  } from '$lib/stores/liveGameStore';
  
  // Local state for manual refresh
  let isManualRefresh = $state(false);
  let lastUpdateTime = $state(Date.now());
  // Debug Store
  import { isDebugModalOpen, openDebugModal, closeDebugModal } from '$lib/stores/debugStore';
  
  // Live Game Components
  import GameStatus from '$lib/components/live-game/GameStatus.svelte';
  import TeamComparison from '$lib/components/live-game/TeamComparison.svelte';
  import EventsTimeline from '$lib/components/live-game/EventsTimeline.svelte';
  import LiveStats from '$lib/components/live-game/LiveStats.svelte';
  import AIRecommendations from '$lib/components/live-game/AIRecommendations.svelte';
  import DebugModal from '$lib/components/live-game/DebugModal.svelte';
  import ActivePlayerInfo from '$lib/components/live-game/ActivePlayerInfo.svelte';
  import NoLiveGameMessage from '$lib/components/live-game/NoLiveGameMessage.svelte';
  import { onMount, onDestroy } from 'svelte';

  // Handle manual refresh
  function handleManualRefresh() {
    isManualRefresh = true;
    lastUpdateTime = Date.now();
    liveGameActions.refresh();
    
    // Reset manual refresh state after a delay
    setTimeout(() => {
      isManualRefresh = false;
    }, 2000);
  }
  
  // Track automatic updates
  $effect(() => {
    if ($gameData && !isManualRefresh) {
      lastUpdateTime = Date.now();
    }
  });

  // Start listening when component mounts, stop when it unmounts
  onMount(() => {
    startListening();
  });

  onDestroy(() => {
    stopListening();
  });
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
          Real-time game analysis and recommendations
        </p>
      </div>
      
      <!-- Timer and Controls -->
      <div class="flex flex-col sm:flex-row items-start sm:items-center gap-4">
        <!-- Game Timer -->
        <div class="flex items-center gap-3 bg-card/50 backdrop-blur-sm border rounded-xl px-4 py-3 shadow-sm">
          <div class="text-xl sm:text-2xl font-mono font-bold flex items-center gap-2">
            {$gameTime}
            {#if liveGameActions.getSyncStatus() === 'synced'}
              <div class="w-2 h-2 bg-green-500 rounded-full animate-pulse" title="Live timer synced with API"></div>
            {:else if liveGameActions.getSyncStatus() === 'stale'}
              <div class="w-2 h-2 bg-yellow-500 rounded-full" title="API sync stale, using local timer"></div>
            {:else if liveGameActions.getSyncStatus() === 'local'}
              <div class="w-2 h-2 bg-blue-500 rounded-full" title="Local timer (no API sync)"></div>
            {:else}
              <div class="w-2 h-2 bg-gray-400 rounded-full" title="No live game detected"></div>
            {/if}
          </div>
          <div class="text-xs sm:text-sm text-muted-foreground">
            {liveGameActions.getSyncStatus() === 'synced' ? 'Synced' : 
             liveGameActions.getSyncStatus() === 'stale' ? 'Stale' :
             liveGameActions.getSyncStatus() === 'local' ? 'Local' : 'Offline'}
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
            disabled={$isLoading || isManualRefresh}
            class="flex items-center gap-2"
          >
            <RefreshCw class="w-4 h-4 {isManualRefresh ? 'animate-spin' : ''}" />
            <span class="hidden sm:inline">{isManualRefresh ? 'Refreshing...' : 'Refresh'}</span>
          </Button>
        </div>
      </div>
    </div>

  {#if $error && !$isSpectatorMode}
    <Alert variant="destructive">
      <AlertCircle class="h-4 w-4" />
      <AlertDescription>Error: {$error}</AlertDescription>
    </Alert>
  {/if}

  <!-- Spectator Mode Notice -->
  {#if $isSpectatorMode}
    <Alert class="border-orange-200 bg-orange-50 dark:border-orange-800 dark:bg-orange-950">
      <Eye class="h-4 w-4 text-orange-600" />
      <AlertDescription class="text-orange-800 dark:text-orange-200">
        <strong>Spectator Mode Detected:</strong> You are currently spectating a game. Some features like active player data and personal statistics are not available in spectator mode.
      </AlertDescription>
    </Alert>
  {/if}

  <!-- Debug Modal -->
  {#if $isDebugModalOpen}
    <DebugModal 
      isOpen={$isDebugModalOpen}
      gameState={$gameState}
      gameData={$gameData}
      liveGameStats={$liveGameStats}
      playerScores={$playerScores}
      isSpectatorMode={$isSpectatorMode}
      copiedStates={$copiedStates}
      onClose={closeDebugModal}
      onCopyToClipboard={liveGameActions.copyToClipboard}
      onDebugLcuConnection={liveGameActions.debugLcuConnection}
    />
  {/if}

  <!-- Show game data if we have valid game data (either live game or spectator mode) -->
  {#if ($gameData?.allPlayers && $gameData.allPlayers.length > 0) && !$error}
  
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
  <GameStatus />

  <!-- Active Player Info -->
  <ActivePlayerInfo />

  <!-- Responsive Layout: Events First on Mobile, Sidebar on Desktop -->
  <div class="space-y-6">
    <!-- Events Timeline - Always visible, prominent on mobile -->
    <div class="lg:hidden">
      <EventsTimeline />
    </div>

    <!-- Desktop Two Column Layout -->
    <div class="hidden lg:grid lg:grid-cols-3 gap-6">
      <!-- Main Content Column -->
      <div class="lg:col-span-2 space-y-6">
        <!-- Team Comparison -->
        <TeamComparison />

        <!-- Live Game Stats -->
        <LiveStats liveGameStats={$liveGameStats} gameMode={$gameMode} />

        <!-- AI Recommendations -->
        <AIRecommendations gameMode={$gameMode} />
      </div>

      <!-- Events Sidebar Column -->
      <div class="lg:col-span-1">
        <!-- Game Events Timeline -->
        <EventsTimeline />
      </div>
    </div>

    <!-- Mobile/Tablet Single Column Layout -->
    <div class="lg:hidden space-y-6">
      <!-- Team Comparison -->
      <TeamComparison />

      <!-- Live Game Stats -->
      <LiveStats liveGameStats={$liveGameStats} gameMode={$gameMode} />

      <!-- AI Recommendations -->
      <AIRecommendations gameMode={$gameMode} />
    </div>
  </div>


  {/if}
  <!-- End of game data sections - only shown when client is connected -->

  <!-- No Live Game Message -->
  {#if !$gameData?.allPlayers || $gameData.allPlayers.length === 0 || $error}
    <NoLiveGameMessage 
      error={$error}
      isLoading={$isLoading}
      isRefreshing={isManualRefresh}
      onRetry={handleManualRefresh}
      onViewDebug={openDebugModal}
    />
  {/if}
  </div>
</div>
