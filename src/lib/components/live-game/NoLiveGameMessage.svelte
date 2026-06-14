<script lang="ts">
  import { Gamepad2, RefreshCw, AlertCircle } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';

  interface Props {
    error: string | null;
    isLoading: boolean;
    isRefreshing: boolean;
    onRetry: () => void;
    onViewDebug: () => void;
  }

  let { error, isLoading, isRefreshing, onRetry, onViewDebug }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center py-20 text-center">
  <div class="w-24 h-24 bg-muted rounded-full flex items-center justify-center mb-6">
    <Gamepad2 class="w-12 h-12 text-muted-foreground" />
  </div>
  <h2 class="text-2xl font-semibold text-foreground mb-2">No Live Game Detected</h2>
  <p class="text-muted-foreground mb-6 max-w-md">
    {#if error}
      {#if error.includes('LCU connection unhealthy') || error.toLowerCase().includes('no lcu connection')}
        <strong>League Client API (LCU) is not connected.</strong>
        The app cannot talk to the client until LCU credentials are detected. Open the League client (keep it running, not only the game), then click
        <strong>Detect LCU</strong> above. If problems persist, use <strong>View Debug Info</strong>.
      {:else if error.includes('Spectator mode')}
        <strong>Spectator Mode Detected</strong><br/>
        The Live Client Data API is not available when spectating. This is a Riot Games API limitation, not a bug.<br/><br/>
        <span class="text-sm">Note: Apps like Mobalytics use different APIs (Riot's server-side Spectator API) which require special API keys. Live data will work when you're actively playing a game.</span>
      {:else if error.includes('Live Client Data API')}
        {error}
      {:else}
        An error occurred while connecting to the game data. Please check the debug information below for details.
      {/if}
    {:else}
      No live game data is currently available. Make sure the League client is running and you're in an active game.
    {/if}
  </p>
  <div class="flex items-center gap-4">
    <Button onclick={onRetry} disabled={isLoading || isRefreshing} class="flex items-center gap-2">
      <RefreshCw class="w-4 h-4 {(isLoading || isRefreshing) ? 'animate-spin' : ''}" />
      {(isLoading || isRefreshing) ? 'Connecting...' : 'Retry Connection'}
    </Button>
    <Button variant="outline" onclick={onViewDebug} class="flex items-center gap-2">
      <AlertCircle class="w-4 h-4" />
      View Debug Info
    </Button>
  </div>
</div>
