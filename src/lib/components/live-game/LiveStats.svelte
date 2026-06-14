<script lang="ts">
  import { TrendingUp } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { wsLiveGameStats, wsGameMode } from '$lib/stores/websocketStore';
  import type { GameStats } from '$lib/types/game';

  // Use WebSocket data instead of props
  let liveGameStats = $wsLiveGameStats;
  let gameMode = $wsGameMode;

  function isArenaMode() {
    return gameMode === 'Arena';
  }
</script>

<Card>
  <CardHeader>
    <CardTitle class="flex items-center gap-2">
      <TrendingUp class="w-5 h-5" />
      Live Statistics
    </CardTitle>
  </CardHeader>
  <CardContent>
    {#if liveGameStats}
      {#if isArenaMode()}
        <!-- Arena-specific stats -->
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div class="text-center p-4 border rounded-lg">
            <div class="text-2xl font-bold text-purple-600">{liveGameStats.team_kills || 0}</div>
            <div class="text-sm text-muted-foreground">Team Kills</div>
          </div>
          <div class="text-center p-4 border rounded-lg">
            <div class="text-2xl font-bold text-red-600">{liveGameStats.enemy_kills || 0}</div>
            <div class="text-sm text-muted-foreground">Enemy Kills</div>
          </div>
          <div class="text-center p-4 border rounded-lg">
            <div class="text-2xl font-bold text-yellow-600">{liveGameStats.round_number || 1}</div>
            <div class="text-sm text-muted-foreground">Round</div>
          </div>
          <div class="text-center p-4 border rounded-lg">
            <div class="text-2xl font-bold text-green-600">{liveGameStats.team_wins || 0}</div>
            <div class="text-sm text-muted-foreground">Wins</div>
          </div>
        </div>
      {:else}
        <!-- Classic game stats -->
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div class="text-center p-4 border rounded-lg">
            <div class="text-2xl font-bold text-blue-600">{liveGameStats.team_kills || 0}</div>
            <div class="text-sm text-muted-foreground">Team Kills</div>
          </div>
          <div class="text-center p-4 border rounded-lg">
            <div class="text-2xl font-bold text-red-600">{liveGameStats.enemy_kills || 0}</div>
            <div class="text-sm text-muted-foreground">Enemy Kills</div>
          </div>
          <div class="text-center p-4 border rounded-lg">
            <div class="text-2xl font-bold text-green-600">{liveGameStats.team_towers || 0}</div>
            <div class="text-sm text-muted-foreground">Towers</div>
          </div>
          <div class="text-center p-4 border rounded-lg">
            <div class="text-2xl font-bold text-purple-600">{liveGameStats.team_dragons || 0}</div>
            <div class="text-sm text-muted-foreground">Dragons</div>
          </div>
        </div>
      {/if}
    {:else}
      <div class="text-center py-8 text-muted-foreground">
        <TrendingUp class="w-12 h-12 mx-auto mb-4 opacity-50" />
        <p>No live statistics available</p>
      </div>
    {/if}
  </CardContent>
</Card>
