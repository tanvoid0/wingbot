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
    gameTime, 
    gamePhase, 
    gameMode, 
    isSpectatorMode,
    copiedStates,
    liveGameActions,
    startListening,
    stopListening
  } from '$lib/stores/liveGameStore';
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

  // Start listening when component mounts, stop when it unmounts
  onMount(() => {
    startListening();
  });

  onDestroy(() => {
    stopListening();
  });
</script>

<div class="p-6 space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold text-foreground">Live Game</h1>
      <p class="text-muted-foreground mt-1">Real-time game analysis and recommendations</p>
    </div>
    <div class="flex items-center gap-4">
      <div class="text-right">
        <div class="text-2xl font-mono font-bold flex items-center gap-2">
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
        <div class="text-sm text-muted-foreground">
          {liveGameActions.getSyncStatus() === 'synced' ? 'Live Timer (Synced)' : 
           liveGameActions.getSyncStatus() === 'stale' ? 'Live Timer (Stale)' :
           liveGameActions.getSyncStatus() === 'local' ? 'Live Timer (Local)' : 'Game Time'}
        </div>
      </div>
      <div class="flex items-center gap-2">
        <Button
          variant="outline"
          onclick={() => {
            console.log('Debug button clicked, opening modal...');
            openDebugModal();
          }}
          class="flex items-center gap-2"
        >
          <Bug class="w-4 h-4" />
          Debug
        </Button>
        <Button
          onclick={liveGameActions.refresh}
          disabled={$isLoading || $isRefreshing}
          class="flex items-center gap-2"
        >
          <RefreshCw class="w-4 h-4 {($isLoading || $isRefreshing) ? 'animate-spin' : ''}" />
          {$isRefreshing ? 'Refreshing...' : 'Refresh'}
        </Button>
      </div>
    </div>
  </div>

  {#if $error}
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
    />
  {/if}

  <!-- Show game data if we have valid game data (either live game or spectator mode) -->
  {#if ($gameData?.allPlayers && $gameData.allPlayers.length > 0) && !$error}
  
  <!-- Refresh Loading Overlay -->
  {#if $isRefreshing}
    <div class="fixed top-4 right-4 z-50 bg-background/90 backdrop-blur-sm border rounded-lg px-3 py-2 shadow-lg">
      <div class="flex items-center gap-2 text-sm">
        <RefreshCw class="w-4 h-4 animate-spin text-primary" />
        <span class="text-muted-foreground">Updating data...</span>
      </div>
    </div>
  {/if}

  <!-- Game Status -->
  <GameStatus />

  <!-- Active Player Info -->
  <ActivePlayerInfo />

  <!-- Team Comparison -->
  <TeamComparison />

  <!-- Game Events Timeline -->
  <EventsTimeline />

  <!-- Live Game Stats -->
  <LiveStats liveGameStats={$liveGameStats} gameMode={$gameMode} />

  <!-- AI Recommendations -->
  <AIRecommendations gameMode={$gameMode} />


  {/if}
  <!-- End of game data sections - only shown when client is connected -->

  <!-- No Live Game Message -->
  {#if !$gameData?.allPlayers || $gameData.allPlayers.length === 0 || $error}
    <NoLiveGameMessage 
      error={$error}
      isLoading={$isLoading}
      isRefreshing={$isRefreshing}
      onRetry={liveGameActions.refresh}
      onViewDebug={openDebugModal}
    />
  {/if}
</div>
