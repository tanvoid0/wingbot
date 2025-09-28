<script lang="ts">
  import { Gamepad2, Eye } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { 
    gameData, 
    gameTime, 
    gameMode, 
    gamePhase, 
    isSpectatorMode,
    liveGameActions 
  } from '$lib/stores/liveGameStore';
</script>

<Card>
  <CardHeader>
    <CardTitle class="flex items-center gap-2">
      <Gamepad2 class="w-5 h-5" />
      Game Status
    </CardTitle>
  </CardHeader>
  <CardContent>
    <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
      <div class="text-center p-4 border rounded-lg">
        <div class="text-2xl font-bold text-foreground flex items-center justify-center gap-2">
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
      <div class="text-center p-4 border rounded-lg">
        <Badge variant="outline" class={liveGameActions.getGameModeColor()}>
          {$gameMode}
        </Badge>
        <div class="text-sm text-muted-foreground mt-1">Game Mode</div>
      </div>
      <div class="text-center p-4 border rounded-lg">
        <Badge variant="outline" class={liveGameActions.getGamePhaseColor()}>
          {$gamePhase}
        </Badge>
        <div class="text-sm text-muted-foreground mt-1">Game Phase</div>
      </div>
      <div class="text-center p-4 border rounded-lg">
        <div class="text-2xl font-bold text-foreground">
          {#if $isSpectatorMode}
            <Eye class="w-6 h-6 mx-auto text-orange-500" />
          {:else}
            {$gameData?.activePlayer?.level || 0}
          {/if}
        </div>
        <div class="text-sm text-muted-foreground">
          {#if $isSpectatorMode}
            Spectating
          {:else}
            Your Level
          {/if}
        </div>
      </div>
    </div>
  </CardContent>
</Card>
