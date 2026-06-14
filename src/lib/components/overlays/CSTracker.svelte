<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Progress } from '$lib/components/ui/progress';
  import { Badge } from '$lib/components/ui/badge';
  import { csTrackerData, overlayConfigs, overlayActions } from '$lib/stores/overlayStore';
  import { wsLiveGameData } from '$lib/stores/websocketStore';

  let config = $derived($overlayConfigs.csTracker);
  let csData = $derived($csTrackerData);
  let gameTime = $state('00:00');
  let gameTimeInterval: number | null = null;

  // Extract CS data from WebSocket data
  $effect(() => {
    const liveData = $wsLiveGameData;
    if (liveData && liveData.activePlayer) {
      const activePlayer = liveData.activePlayer;
      const gameTime = liveData.gameData?.gameTime || 0;
      const gameTimeMinutes = gameTime / 60;
      
      // Calculate CS data
      const cs = (activePlayer as any).cs || 0;
      const targetCS = Math.floor(gameTimeMinutes * 10); // 10 CS per minute target
      const csPerMin = gameTimeMinutes > 0 ? cs / gameTimeMinutes : 0;
      const goldFromCS = cs * 20; // Approximate gold per CS
      
      let benchmark: 'poor' | 'average' | 'good' | 'excellent' = 'poor';
      if (csPerMin >= 8) benchmark = 'excellent';
      else if (csPerMin >= 6) benchmark = 'good';
      else if (csPerMin >= 4) benchmark = 'average';
      
      const csData = {
        current: cs,
        target: targetCS,
        perMinute: csPerMin,
        goldFromCS: goldFromCS,
        benchmark: benchmark
      };
      
      csTrackerData.set(csData);
    }
  });

  onMount(() => {
    // Start game time tracking
    gameTimeInterval = setInterval(() => {
      const now = new Date();
      const minutes = Math.floor(now.getSeconds() / 60);
      const seconds = now.getSeconds() % 60;
      gameTime = `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
    }, 1000);

    onDestroy(() => {
      if (gameTimeInterval) clearInterval(gameTimeInterval);
    });
  });

  function getBenchmarkColor(benchmark: string) {
    switch (benchmark) {
      case 'excellent': return 'text-green-500';
      case 'good': return 'text-blue-500';
      case 'average': return 'text-yellow-500';
      case 'poor': return 'text-red-500';
      default: return 'text-gray-500';
    }
  }

  function getBenchmarkIcon(benchmark: string) {
    switch (benchmark) {
      case 'excellent': return '⭐';
      case 'good': return '👍';
      case 'average': return '👌';
      case 'poor': return '👎';
      default: return '📊';
    }
  }
</script>

{#if config.enabled && csData}
  <div 
    class="fixed z-50 bg-black/80 backdrop-blur-sm rounded-lg border border-white/20 p-4 min-w-[200px]"
    style="
      left: {config.x}px; 
      top: {config.y}px; 
      opacity: {config.opacity}; 
      transform: scale({config.scale});
    "
  >
    <Card class="bg-transparent border-0">
      <CardContent class="p-0">
        <div class="space-y-3">
          <!-- Header -->
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-semibold text-white">CS Tracker</h3>
            <div class="flex items-center gap-2">
              <Badge variant="outline" class="text-xs">
                {getBenchmarkIcon(csData.benchmark)}
              </Badge>
              <button 
                class="text-white/60 hover:text-white text-xs"
                onclick={() => overlayActions.toggleOverlay('csTracker')}
              >
                ✕
              </button>
            </div>
          </div>

          <!-- CS Stats -->
          <div class="space-y-2">
            <div class="flex justify-between items-center">
              <span class="text-white/80 text-sm">Current CS:</span>
              <span class="text-white font-mono text-sm">{csData.current}</span>
            </div>
            
            <div class="flex justify-between items-center">
              <span class="text-white/80 text-sm">Target CS:</span>
              <span class="text-white font-mono text-sm">{csData.target}</span>
            </div>

            <div class="flex justify-between items-center">
              <span class="text-white/80 text-sm">CS/min:</span>
              <span class="text-white font-mono text-sm">{csData.perMinute.toFixed(1)}</span>
            </div>

            <div class="flex justify-between items-center">
              <span class="text-white/80 text-sm">Gold from CS:</span>
              <span class="text-yellow-400 font-mono text-sm">{csData.goldFromCS}g</span>
            </div>
          </div>

          <!-- Progress Bar -->
          <div class="space-y-1">
            <div class="flex justify-between text-xs">
              <span class="text-white/60">Progress</span>
              <span class="text-white/60">{Math.round((csData.current / csData.target) * 100)}%</span>
            </div>
            <Progress 
              value={(csData.current / csData.target) * 100} 
              class="h-2"
            />
          </div>

          <!-- Benchmark Status -->
          <div class="flex items-center justify-center">
            <Badge 
              variant="outline" 
              class="text-xs {getBenchmarkColor(csData.benchmark)}"
            >
              {csData.benchmark.toUpperCase()}
            </Badge>
          </div>
        </div>
      </CardContent>
    </Card>
  </div>
{/if}

<style>
  /* Make overlay draggable */
  :global(.overlay-draggable) {
    cursor: move;
  }
</style>
