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
      An error occurred while connecting to the game data. Please check the debug information below for details.
    {:else}
      No live game data is currently available. Make sure the League client is running and you're in a game or spectating a match.
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
