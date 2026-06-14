<script lang="ts">
  import { User, Target, Zap, Shield } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { wsGameData, wsLiveGameData, wsIsSpectatorMode } from '$lib/stores/websocketStore';
  import type { LiveClientActivePlayer } from '$lib/types/game';

  // Get active player data with proper typing
  $: gameData = $wsGameData || $wsLiveGameData;
  $: activePlayer = gameData?.activePlayer as LiveClientActivePlayer | undefined;
  $: isSpectator = $wsIsSpectatorMode;
</script>

{#if gameData?.activePlayer && !$wsIsSpectatorMode}
<Card>
  <CardHeader>
    <CardTitle class="flex items-center gap-2">
      <User class="w-5 h-5" />
      Active Player
    </CardTitle>
  </CardHeader>
  <CardContent>
    {#if isSpectator}
      <div class="text-center py-8 text-muted-foreground">
        <User class="w-12 h-12 mx-auto mb-4 opacity-50" />
        <p>Active player data not available in spectator mode</p>
      </div>
    {:else if activePlayer}
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- Player Info -->
        <div class="space-y-4">
          <div class="flex items-center gap-3">
            <div class="w-12 h-12 bg-primary/10 rounded-lg flex items-center justify-center">
              <User class="w-6 h-6 text-primary" />
            </div>
            <div>
              <h3 class="text-lg font-semibold">{activePlayer.summonerName}</h3>
              <p class="text-sm text-muted-foreground">{activePlayer.championName}</p>
            </div>
          </div>
          
          <div class="grid grid-cols-2 gap-4 text-sm">
            <div class="flex justify-between">
              <span class="text-muted-foreground">Level:</span>
              <span class="font-medium">{activePlayer.level}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-muted-foreground">Gold:</span>
              <span class="font-medium">{activePlayer.currentGold}</span>
            </div>
          </div>
        </div>
        
        <!-- Stats -->
        <div class="space-y-4">
          <h4 class="text-sm font-medium text-muted-foreground">Current Stats</h4>
          <div class="grid grid-cols-2 gap-4 text-sm">
            <div class="flex justify-between">
              <span class="text-muted-foreground">Kills:</span>
              <span class="font-medium">{activePlayer?.scores?.kills || 0}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-muted-foreground">Deaths:</span>
              <span class="font-medium">{activePlayer?.scores?.deaths || 0}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-muted-foreground">Assists:</span>
              <span class="font-medium">{activePlayer?.scores?.assists || 0}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-muted-foreground">CS:</span>
              <span class="font-medium">{activePlayer?.scores?.creepScore || 0}</span>
            </div>
          </div>
        </div>
      </div>
      
      <!-- Items -->
      {#if activePlayer?.items && Array.isArray(activePlayer.items) && activePlayer.items.length > 0}
        <div class="mt-6">
          <h4 class="text-sm font-medium text-muted-foreground mb-3">Items</h4>
          <div class="grid grid-cols-6 gap-2">
            {#each activePlayer.items as item, index}
              <div class="w-12 h-12 bg-muted rounded-lg flex items-center justify-center text-xs font-medium">
                {item || index + 1}
              </div>
            {/each}
          </div>
        </div>
      {/if}
    {:else}
      <div class="text-center py-8 text-muted-foreground">
        <User class="w-12 h-12 mx-auto mb-4 opacity-50" />
        <p>No active player data available</p>
      </div>
    {/if}
  </CardContent>
</Card>
{/if}
