<script lang="ts">
  import { Gamepad2, RefreshCw, AlertCircle, Play } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';

  interface Props {
    error: string | null;
    isLoading: boolean;
    isRefreshing: boolean;
    onRetry: () => void;
    onViewDebug: () => void;
    onLaunchLeague?: () => void;
  }

  let { error, isLoading, isRefreshing, onRetry, onViewDebug, onLaunchLeague }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center py-8 text-center max-w-md mx-auto">
  <div class="w-12 h-12 bg-muted rounded-full flex items-center justify-center mb-3">
    <Gamepad2 class="w-6 h-6 text-muted-foreground" />
  </div>
  <h2 class="text-lg font-semibold text-foreground mb-1">No active session</h2>
  <p class="text-sm text-muted-foreground mb-4">
    {#if error}
      {#if error.includes('LCU connection unhealthy') || error.toLowerCase().includes('no lcu connection')}
        LCU not connected. Open the League client, then retry.
      {:else if error.includes('Spectator mode')}
        Live Client API unavailable in spectator mode.
      {:else if error.includes('Live Client Data API')}
        {error}
      {:else}
        Connection error — use debug info for details.
      {/if}
    {:else}
      Start League or connect to an active game.
    {/if}
  </p>
  <div class="flex flex-wrap items-center justify-center gap-2">
    <Button size="sm" onclick={onRetry} disabled={isLoading || isRefreshing} class="flex items-center gap-1.5">
      <RefreshCw class="w-3.5 h-3.5 {(isLoading || isRefreshing) ? 'animate-spin' : ''}" />
      {(isLoading || isRefreshing) ? 'Connecting…' : 'Retry'}
    </Button>
    {#if onLaunchLeague}
      <Button size="sm" variant="secondary" onclick={onLaunchLeague} class="flex items-center gap-1.5">
        <Play class="w-3.5 h-3.5" />
        Launch
      </Button>
    {/if}
    <Button size="sm" variant="outline" onclick={onViewDebug} class="flex items-center gap-1.5">
      <AlertCircle class="w-3.5 h-3.5" />
      Debug
    </Button>
  </div>
</div>
