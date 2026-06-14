<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { counterItems, overlayConfigs, overlayActions } from '$lib/stores/overlayStore';
  import { wsLiveGameData } from '$lib/stores/websocketStore';

  let config = $derived($overlayConfigs.counterItems);
  let counterData = $derived($counterItems);
  let updateInterval: number | null = null;

  // Extract counter item data from WebSocket data
  $effect(() => {
    const liveData = $wsLiveGameData;
    if (liveData && liveData.allPlayers) {
      const allPlayers = liveData.allPlayers;
      const enemyTeam = allPlayers.filter((player: any) => player.team === 'CHAOS');
      
      // Generate counter item suggestions based on enemy team
      const suggestions: any[] = [];
      
      enemyTeam.forEach((enemy: any) => {
        const championName = enemy.championName;
        const items = enemy.items || [];
        
        // Basic counter suggestions based on champion
        if (championName === 'Yasuo') {
          suggestions.push({
            itemId: 3031,
            name: 'Infinity Edge',
            cost: 3400,
            reason: 'High damage against Yasuo',
            priority: 'high' as const,
            enemyThreat: 'Yasuo'
          });
        }
        
        if (championName === 'Zed' || championName === 'Talon') {
          suggestions.push({
            itemId: 3072,
            name: 'The Bloodthirster',
            cost: 3200,
            reason: 'Shield against burst damage',
            priority: 'high' as const,
            enemyThreat: championName
          });
        }
      });
      
      counterItems.set(suggestions);
    }
  });

  onMount(() => {
    // No need for intervals since we're using reactive WebSocket data
  });

  function getPriorityColor(priority: string) {
    switch (priority) {
      case 'high': return 'text-red-400 bg-red-500/20';
      case 'medium': return 'text-yellow-400 bg-yellow-500/20';
      case 'low': return 'text-blue-400 bg-blue-500/20';
      default: return 'text-gray-400 bg-gray-500/20';
    }
  }

  function getPriorityIcon(priority: string) {
    switch (priority) {
      case 'high': return '🔥';
      case 'medium': return '⚡';
      case 'low': return '💡';
      default: return '📋';
    }
  }

  function formatCost(cost: number) {
    return `${cost.toLocaleString()}g`;
  }

  function getItemIcon(itemId: number) {
    // This would ideally use actual item icons from the game
    // For now, return emoji representations
    switch (itemId) {
      case 3140: return '⚔️'; // Mercurial Scimitar
      case 3157: return '⏰'; // Zhonya's Hourglass
      case 3047: return '👢'; // Plated Steelcaps
      case 3031: return '💀'; // Mortal Reminder
      case 3036: return '🏹'; // Lord Dominik's Regards
      case 3085: return '🌪️'; // Runaan's Hurricane
      default: return '🛡️';
    }
  }
</script>

{#if config.enabled && counterData && counterData.length > 0}
  <div 
    class="fixed z-50 bg-black/80 backdrop-blur-sm rounded-lg border border-white/20 p-4 min-w-[300px]"
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
            <h3 class="text-sm font-semibold text-white">Counter Items</h3>
            <button 
              class="text-white/60 hover:text-white text-xs"
              onclick={() => overlayActions.toggleOverlay('counterItems')}
            >
              ✕
            </button>
          </div>

          <!-- Counter Suggestions -->
          <div class="space-y-2">
            {#each counterData as item}
              <div class="p-3 rounded-lg bg-white/5 border border-white/10 hover:bg-white/10 transition-colors">
                <div class="flex items-start gap-3">
                  <div class="text-2xl">{getItemIcon(item.itemId)}</div>
                  
                  <div class="flex-1">
                    <div class="flex items-center gap-2 mb-1">
                      <span class="text-white font-medium">{item.name}</span>
                      <Badge 
                        variant="outline" 
                        class="text-xs {getPriorityColor(item.priority)}"
                      >
                        {getPriorityIcon(item.priority)} {item.priority.toUpperCase()}
                      </Badge>
                    </div>
                    
                    <div class="text-sm text-white/80 mb-2">
                      {item.reason}
                    </div>
                    
                    <div class="flex items-center justify-between text-xs">
                      <span class="text-yellow-400 font-mono">
                        {formatCost(item.cost)}
                      </span>
                      <span class="text-white/60">
                        vs {item.enemyThreat}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            {/each}
          </div>

          <!-- Quick Stats -->
          <div class="pt-2 border-t border-white/10">
            <div class="flex items-center justify-between text-xs text-white/60">
              <span>Total Suggestions: {counterData.length}</span>
              <span>
                High Priority: {counterData.filter(item => item.priority === 'high').length}
              </span>
            </div>
          </div>

          <!-- Actions -->
          <div class="flex gap-2">
            <button 
              class="flex-1 px-3 py-1 bg-green-500/20 hover:bg-green-500/30 text-green-400 text-xs rounded transition-colors"
              onclick={() => console.log('View in shop')}
            >
              View in Shop
            </button>
            <button 
              class="flex-1 px-3 py-1 bg-purple-500/20 hover:bg-purple-500/30 text-purple-400 text-xs rounded transition-colors"
              onclick={() => console.log('Refresh suggestions')}
            >
              Refresh
            </button>
          </div>
        </div>
      </CardContent>
    </Card>
  </div>
{/if}
