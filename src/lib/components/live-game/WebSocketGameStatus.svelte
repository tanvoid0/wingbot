<script lang="ts">
  import { Gamepad2, Eye } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { 
    wsGameData,
    wsLiveGameData, 
    wsGameTime, 
    wsGameMode, 
    wsGamePhase, 
    wsIsSpectatorMode,
    wsActions 
  } from '$lib/stores/websocketStore';
</script>

{#if $wsGameData || $wsLiveGameData || $wsGamePhase !== 'No Live Game'}
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
          {$wsGameTime}
          {#if wsActions.getSyncStatus() === 'synced'}
            <div class="w-2 h-2 bg-green-500 rounded-full animate-pulse" title="Live timer synced with API"></div>
          {:else if wsActions.getSyncStatus() === 'stale'}
            <div class="w-2 h-2 bg-yellow-500 rounded-full" title="API sync stale, using local timer"></div>
          {:else if wsActions.getSyncStatus() === 'local'}
            <div class="w-2 h-2 bg-blue-500 rounded-full" title="Local timer (no API sync)"></div>
          {:else}
            <div class="w-2 h-2 bg-gray-400 rounded-full" title="No live game detected"></div>
          {/if}
        </div>
        <div class="text-sm text-muted-foreground mt-1">Game Time</div>
      </div>
      
      <div class="text-center p-4 border rounded-lg">
        <div class="text-lg font-semibold text-foreground">
          {$wsGameMode}
        </div>
        <div class="text-sm text-muted-foreground mt-1">Game Mode</div>
      </div>
      
      <div class="text-center p-4 border rounded-lg">
        <div class="text-lg font-semibold text-foreground">
          {$wsGamePhase}
        </div>
        <div class="text-sm text-muted-foreground mt-1">Game Phase</div>
      </div>
      
      <div class="text-center p-4 border rounded-lg">
        <div class="flex items-center justify-center gap-2">
          {#if $wsIsSpectatorMode}
            <Eye class="w-5 h-5 text-orange-500" />
            <Badge variant="secondary" class="bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200">
              Spectator
            </Badge>
          {:else}
            <Gamepad2 class="w-5 h-5 text-green-500" />
            <Badge variant="secondary" class="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
              Playing
            </Badge>
          {/if}
        </div>
        <div class="text-sm text-muted-foreground mt-1">Mode</div>
      </div>
    </div>
  </CardContent>
</Card>
{/if}
