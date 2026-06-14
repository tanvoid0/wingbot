<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { objectiveTimers, overlayConfigs, overlayActions } from '$lib/stores/overlayStore';
  import { wsLiveGameData } from '$lib/stores/websocketStore';

  let config = $derived($overlayConfigs.objectiveTimers);
  let objectiveData = $derived($objectiveTimers);
  let updateInterval: number | null = null;

  // Extract objective data from WebSocket data
  $effect(() => {
    const liveData = $wsLiveGameData;
    if (liveData) {
      const gameTime = liveData.gameData?.gameTime || 0;
      
      const objectives = [
        {
          id: 'dragon',
          objective: 'Dragon',
          type: 'dragon' as const,
          spawnTime: 300,
          isAlive: gameTime >= 300,
          timeUntilSpawn: Math.max(0, 300 - gameTime),
          nextSpawn: 300
        },
        {
          id: 'baron',
          objective: 'Baron',
          type: 'baron' as const,
          spawnTime: 1200,
          isAlive: gameTime >= 1200,
          timeUntilSpawn: Math.max(0, 1200 - gameTime),
          nextSpawn: 1200
        },
        {
          id: 'herald',
          objective: 'Herald',
          type: 'herald' as const,
          spawnTime: 600,
          isAlive: gameTime >= 600,
          timeUntilSpawn: Math.max(0, 600 - gameTime),
          nextSpawn: 600
        }
      ];
      
      objectiveTimers.set(objectives);
    }
  });

  onMount(() => {
    // No need for intervals since we're using reactive WebSocket data
  });

  function getObjectiveIcon(objective: string) {
    switch (objective.toLowerCase()) {
      case 'dragon': return '🐉';
      case 'baron': return '👹';
      case 'herald': return '🏰';
      case 'grubs': return '🪲';
      default: return '⚔️';
    }
  }

  function formatTime(seconds: number) {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function getObjectiveStatus(objective: any) {
    if (objective.isAlive) {
      return { 
        status: 'alive', 
        color: 'text-green-400', 
        text: 'Available',
        bgColor: 'bg-green-500/20'
      };
    } else {
      const timeUntilSpawn = objective.timeUntilSpawn;
      if (timeUntilSpawn <= 30) {
        return { 
          status: 'spawning', 
          color: 'text-yellow-400', 
          text: 'Spawning Soon',
          bgColor: 'bg-yellow-500/20'
        };
      } else {
        return { 
          status: 'dead', 
          color: 'text-red-400', 
          text: formatTime(timeUntilSpawn),
          bgColor: 'bg-red-500/20'
        };
      }
    }
  }

  function getPriority(objective: any) {
    if (objective.objective === 'Baron' && objective.timeUntilSpawn <= 60) return 'high';
    if (objective.objective === 'Dragon' && objective.timeUntilSpawn <= 30) return 'high';
    if (objective.timeUntilSpawn <= 15) return 'medium';
    return 'low';
  }
</script>

{#if config.enabled && objectiveData && objectiveData.length > 0}
  <div 
    class="fixed z-50 bg-black/80 backdrop-blur-sm rounded-lg border border-white/20 p-4 min-w-[280px]"
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
            <h3 class="text-sm font-semibold text-white">Objective Timers</h3>
            <button 
              class="text-white/60 hover:text-white text-xs"
              onclick={() => overlayActions.toggleOverlay('objectiveTimers')}
            >
              ✕
            </button>
          </div>

          <!-- Objectives -->
          <div class="space-y-2">
            {#each objectiveData as objective}
              {@const status = getObjectiveStatus(objective)}
              {@const priority = getPriority(objective)}
              <div class="p-3 rounded-lg {status.bgColor} border border-white/10">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-3">
                    <span class="text-2xl">{getObjectiveIcon(objective.objective)}</span>
                    <div>
                      <div class="text-white font-medium">{objective.objective}</div>
                      <div class="text-xs text-white/70">
                        {objective.isAlive ? 'Available Now' : `Next: ${formatTime(objective.nextSpawn)}`}
                      </div>
                    </div>
                  </div>
                  
                  <div class="text-right">
                    <div class="text-lg {status.color} font-mono font-bold">
                      {status.text}
                    </div>
                    {#if !objective.isAlive}
                      <div class="text-xs text-white/60">
                        {formatTime(objective.timeUntilSpawn)} remaining
                      </div>
                    {/if}
                  </div>
                </div>

                <!-- Priority indicator -->
                {#if priority === 'high'}
                  <div class="mt-2 flex items-center gap-1">
                    <div class="w-2 h-2 bg-red-500 rounded-full animate-pulse"></div>
                    <span class="text-xs text-red-400 font-medium">High Priority</span>
                  </div>
                {/if}
              </div>
            {/each}
          </div>

          <!-- Quick Actions -->
          <div class="flex gap-2">
            <button 
              class="flex-1 px-3 py-1 bg-blue-500/20 hover:bg-blue-500/30 text-blue-400 text-xs rounded transition-colors"
              onclick={() => console.log('Ping objectives')}
            >
              Ping Team
            </button>
            <button 
              class="flex-1 px-3 py-1 bg-purple-500/20 hover:bg-purple-500/30 text-purple-400 text-xs rounded transition-colors"
              onclick={() => console.log('Set reminder')}
            >
              Set Reminder
            </button>
          </div>
        </div>
      </CardContent>
    </Card>
  </div>
{/if}
