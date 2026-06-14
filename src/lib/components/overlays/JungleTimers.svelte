<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { jungleTimers, overlayConfigs, overlayActions } from '$lib/stores/overlayStore';
  import { wsLiveGameData } from '$lib/stores/websocketStore';

  let config = $derived($overlayConfigs.jungleTimers);
  let jungleData = $derived($jungleTimers);
  let updateInterval: number | null = null;

  // Extract jungle data from WebSocket data
  $effect(() => {
    const liveData = $wsLiveGameData;
    if (liveData && liveData.events) {
      const events = liveData.events.Events || [];
      const gameTime = liveData.gameData?.gameTime || 0;
      
      // Parse jungle camp events
      const camps = [
        { id: 'blue_buff', campName: 'Blue_Buff', type: 'buff' as const, respawnTime: 300, isAlive: true, team: 'blue' as const },
        { id: 'red_buff', campName: 'Red_Buff', type: 'buff' as const, respawnTime: 300, isAlive: true, team: 'red' as const },
        { id: 'gromp', campName: 'Gromp', type: 'camp' as const, respawnTime: 150, isAlive: true, team: 'blue' as const },
        { id: 'krugs', campName: 'Krugs', type: 'camp' as const, respawnTime: 150, isAlive: true, team: 'red' as const },
        { id: 'wolves', campName: 'Wolves', type: 'camp' as const, respawnTime: 150, isAlive: true, team: 'blue' as const },
        { id: 'raptors', campName: 'Raptors', type: 'camp' as const, respawnTime: 150, isAlive: true, team: 'red' as const }
      ];
      
      // Update camp status based on events
      events.forEach((event: any) => {
        if (event.EventName === 'JungleCampKill') {
          const campName = event.VictimName;
          const camp = camps.find(c => c.campName === campName);
          if (camp) {
            camp.isAlive = false;
            camp.respawnTime = gameTime + (camp.type === 'buff' ? 300 : 150);
          }
        }
      });
      
      jungleTimers.set(camps);
    }
  });

  onMount(() => {
    // No need for intervals since we're using reactive WebSocket data
  });

  function getCampIcon(campName: string) {
    switch (campName.toLowerCase()) {
      case 'blue_buff': return '🔵';
      case 'red_buff': return '🔴';
      case 'gromp': return '🐸';
      case 'wolves': return '🐺';
      case 'raptors': return '🦅';
      case 'krugs': return '🪨';
      default: return '🌿';
    }
  }

  function getTeamColor(team: string) {
    return team === 'ORDER' ? 'text-blue-400' : 'text-red-400';
  }

  function formatTime(seconds: number) {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function getRespawnStatus(camp: any) {
    const now = Date.now() / 1000;
    const timeUntilRespawn = camp.respawnTime - now;
    
    if (timeUntilRespawn <= 0) {
      return { status: 'alive', color: 'text-green-400', text: 'Alive' };
    } else if (timeUntilRespawn <= 30) {
      return { status: 'spawning', color: 'text-yellow-400', text: 'Spawning Soon' };
    } else {
      return { status: 'dead', color: 'text-red-400', text: formatTime(timeUntilRespawn) };
    }
  }
</script>

{#if config.enabled && jungleData && jungleData.length > 0}
  <div 
    class="fixed z-50 bg-black/80 backdrop-blur-sm rounded-lg border border-white/20 p-4 min-w-[250px]"
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
            <h3 class="text-sm font-semibold text-white">Jungle Timers</h3>
            <button 
              class="text-white/60 hover:text-white text-xs"
              onclick={() => overlayActions.toggleOverlay('jungleTimers')}
            >
              ✕
            </button>
          </div>

          <!-- Jungle Camps -->
          <div class="space-y-2">
            {#each jungleData as camp}
              {@const status = getRespawnStatus(camp)}
              <div class="flex items-center justify-between p-2 bg-white/5 rounded">
                <div class="flex items-center gap-2">
                  <span class="text-lg">{getCampIcon(camp.campName)}</span>
                  <div>
                    <div class="text-white text-sm font-medium">{camp.campName.replace('_', ' ')}</div>
                    <div class="text-xs {getTeamColor(camp.team)}">
                      {camp.team === 'ORDER' ? 'Blue' : 'Red'} Team
                    </div>
                  </div>
                </div>
                
                <div class="text-right">
                  <div class="text-sm {status.color} font-mono">
                    {status.text}
                  </div>
                  {#if status.status === 'dead'}
                    <div class="text-xs text-white/60">
                      Respawn: {formatTime(camp.respawnTime - Date.now() / 1000)}
                    </div>
                  {/if}
                </div>
              </div>
            {/each}
          </div>

          <!-- Legend -->
          <div class="flex items-center justify-center gap-4 text-xs text-white/60">
            <div class="flex items-center gap-1">
              <div class="w-2 h-2 bg-green-400 rounded"></div>
              <span>Alive</span>
            </div>
            <div class="flex items-center gap-1">
              <div class="w-2 h-2 bg-yellow-400 rounded"></div>
              <span>Spawning</span>
            </div>
            <div class="flex items-center gap-1">
              <div class="w-2 h-2 bg-red-400 rounded"></div>
              <span>Dead</span>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  </div>
{/if}
