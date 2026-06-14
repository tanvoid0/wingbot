<script lang="ts">
  import { Package, TrendingUp, Target, AlertCircle, CheckCircle, Clock } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Progress } from '$lib/components/ui/progress';
  import { 
    wsAllPlayersItems, 
    wsBuildAnalysis, 
    wsItemRecommendations
  } from '$lib/stores/websocketStore';
  import { onMount } from 'svelte';

  // Local state
  let selectedPlayer = $state<string | null>(null);
  let showRecommendations = $state(true);

  // Get player list from items data
  let playerList = $derived(Object.keys($wsAllPlayersItems));
  
  // Set first player as default selected
  $effect(() => {
    if (playerList.length > 0 && !selectedPlayer) {
      selectedPlayer = playerList[0];
    }
  });

  // Get selected player's analysis
  let selectedPlayerAnalysis = $derived(selectedPlayer ? $wsBuildAnalysis[selectedPlayer] : null);

  // Calculate summary data
  let itemTrackingSummary = $derived({
    totalPlayers: Object.keys($wsAllPlayersItems).length,
    playersWithItems: Object.values($wsBuildAnalysis).filter((a: any) => a && a.completedItems > 0).length,
    averageEfficiency: Object.keys($wsAllPlayersItems).length > 0 ? 
      Math.round(Object.values($wsBuildAnalysis).reduce((sum: number, a: any) => sum + (a?.goldEfficiency || 0), 0) / Object.keys($wsAllPlayersItems).length) : 0
  });

  function getBuildStageColor(stage: string) {
    switch (stage) {
      case 'early': return 'text-green-600';
      case 'mid': return 'text-yellow-600';
      case 'late': return 'text-purple-600';
      default: return 'text-gray-600';
    }
  }

  function getBuildStageIcon(stage: string) {
    switch (stage) {
      case 'early': return Clock;
      case 'mid': return TrendingUp;
      case 'late': return Target;
      default: return Package;
    }
  }

  function getPriorityColor(priority: string) {
    switch (priority) {
      case 'high': return 'text-red-600';
      case 'medium': return 'text-yellow-600';
      case 'low': return 'text-green-600';
      default: return 'text-gray-600';
    }
  }
</script>

{#if $wsAllPlayersItems && Object.keys($wsAllPlayersItems).length > 0}
  <Card class="bg-gradient-to-br from-card/50 to-card/30 backdrop-blur-sm border-border/50 shadow-lg">
    <CardHeader class="pb-4">
      <CardTitle class="flex items-center gap-3 text-lg">
        <div class="p-2 rounded-lg bg-primary/10">
          <Package class="w-5 h-5 text-primary" />
        </div>
        <div class="flex-1">
          <div class="font-bold">Item Tracking</div>
          <div class="text-sm text-muted-foreground font-normal">
            Real-time build analysis and recommendations
          </div>
        </div>
        <Badge variant="outline" class="text-xs">
          {itemTrackingSummary.totalPlayers} Players
        </Badge>
      </CardTitle>
    </CardHeader>
    <CardContent>
      <div class="space-y-6">
        <!-- Overall Summary -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div class="p-3 bg-muted/30 rounded-lg">
            <div class="flex items-center gap-2 mb-2">
              <Package class="w-4 h-4 text-blue-500" />
              <span class="text-sm font-medium">Total Players</span>
            </div>
            <div class="text-2xl font-bold text-blue-600">
              {itemTrackingSummary.totalPlayers}
            </div>
          </div>
          
          <div class="p-3 bg-muted/30 rounded-lg">
            <div class="flex items-center gap-2 mb-2">
              <CheckCircle class="w-4 h-4 text-green-500" />
              <span class="text-sm font-medium">With Items</span>
            </div>
            <div class="text-2xl font-bold text-green-600">
              {itemTrackingSummary.playersWithItems}
            </div>
          </div>
          
          <div class="p-3 bg-muted/30 rounded-lg">
            <div class="flex items-center gap-2 mb-2">
              <TrendingUp class="w-4 h-4 text-purple-500" />
              <span class="text-sm font-medium">Avg Efficiency</span>
            </div>
            <div class="text-2xl font-bold text-purple-600">
              {itemTrackingSummary.averageEfficiency}%
            </div>
          </div>
        </div>

        <!-- Player Selection -->
        {#if playerList.length > 1}
          <div class="space-y-2">
            <label for="player-select" class="text-sm font-medium">Select Player:</label>
            <div id="player-select" class="flex flex-wrap gap-2" role="group" aria-labelledby="player-select">
              {#each playerList as playerId}
                <button
                  onclick={() => selectedPlayer = playerId}
                  class="px-3 py-1 text-xs rounded-full border transition-colors
                    {selectedPlayer === playerId 
                      ? 'bg-primary text-primary-foreground border-primary' 
                      : 'bg-muted/50 hover:bg-muted border-border'}"
                >
                  Player {playerId.slice(-4)}
                </button>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Selected Player Analysis -->
        {#if selectedPlayer && selectedPlayerAnalysis}
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <h3 class="text-lg font-semibold">
                Player {selectedPlayer.slice(-4)} Build Analysis
              </h3>
              <Badge variant="outline" class={getBuildStageColor(selectedPlayerAnalysis.buildStage || 'early')}>
                {selectedPlayerAnalysis.buildStage || 'early'} game
              </Badge>
            </div>

            <!-- Build Progress -->
            <div class="space-y-3">
              <div class="flex items-center justify-between text-sm">
                <span>Build Completion</span>
                <span>{selectedPlayerAnalysis.completedItems}/{selectedPlayerAnalysis.totalItems} items</span>
              </div>
              <Progress 
                value={selectedPlayerAnalysis.totalItems > 0 ? (selectedPlayerAnalysis.completedItems / selectedPlayerAnalysis.totalItems) * 100 : 0} 
                class="h-2"
              />
            </div>

            <!-- Gold Efficiency -->
            <div class="space-y-3">
              <div class="flex items-center justify-between text-sm">
                <span>Gold Efficiency</span>
                <span>{selectedPlayerAnalysis.goldEfficiency}%</span>
              </div>
              <Progress 
                value={selectedPlayerAnalysis.goldEfficiency} 
                class="h-2"
              />
            </div>

            <!-- Current Items -->
            {#if selectedPlayerAnalysis.buildPath && selectedPlayerAnalysis.buildPath.length > 0}
              <div class="space-y-2">
                <h4 class="text-sm font-medium">Current Items:</h4>
                <div class="grid grid-cols-2 md:grid-cols-3 gap-2">
                  {#each selectedPlayerAnalysis.buildPath as item}
                    <div class="p-2 bg-muted/50 rounded-lg text-xs">
                      <div class="font-medium truncate">{item.name}</div>
                      <div class="text-muted-foreground">{item.cost} gold</div>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}

            <!-- Recommendations -->
            {#if showRecommendations && selectedPlayerAnalysis.recommendations && selectedPlayerAnalysis.recommendations.length > 0}
              <div class="space-y-2">
                <div class="flex items-center justify-between">
                  <h4 class="text-sm font-medium">Recommendations:</h4>
                  <button
                    onclick={() => showRecommendations = !showRecommendations}
                    class="text-xs text-muted-foreground hover:text-foreground"
                  >
                    Hide
                  </button>
                </div>
                <div class="space-y-2">
                  {#each selectedPlayerAnalysis.recommendations as rec}
                    <div class="p-3 bg-muted/30 rounded-lg">
                      <div class="flex items-start gap-2">
                        <AlertCircle class="w-4 h-4 mt-0.5 {getPriorityColor(rec.priority)}" />
                        <div class="flex-1">
                          <div class="text-sm font-medium {getPriorityColor(rec.priority)}">
                            {rec.message}
                          </div>
                          {#if rec.items && rec.items.length > 0}
                            <div class="text-xs text-muted-foreground mt-1">
                              Consider: {rec.items.join(', ')}
                            </div>
                          {/if}
                        </div>
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            {:else if !showRecommendations}
              <button
                onclick={() => showRecommendations = true}
                class="text-xs text-muted-foreground hover:text-foreground"
              >
                Show Recommendations
              </button>
            {/if}
          </div>
        {/if}
      </div>
    </CardContent>
  </Card>
{:else}
  <Card class="bg-gradient-to-br from-card/50 to-card/30 backdrop-blur-sm border-border/50 shadow-lg opacity-60">
    <CardHeader class="pb-4">
      <CardTitle class="flex items-center gap-3 text-lg">
        <div class="p-2 rounded-lg bg-muted/50">
          <Package class="w-5 h-5 text-muted-foreground" />
        </div>
        <div class="flex-1">
          <div class="font-bold text-muted-foreground">Item Tracking</div>
          <div class="text-sm text-muted-foreground font-normal">
            No item data available
          </div>
        </div>
        <Badge variant="outline" class="text-muted-foreground">
          Inactive
        </Badge>
      </CardTitle>
    </CardHeader>
    <CardContent>
      <div class="text-center py-8 text-muted-foreground">
        <div class="p-4 rounded-full bg-muted/50 w-fit mx-auto mb-4">
          <Package class="w-8 h-8 text-muted-foreground/50" />
        </div>
        <p class="text-sm">Waiting for live game data...</p>
      </div>
    </CardContent>
  </Card>
{/if}
