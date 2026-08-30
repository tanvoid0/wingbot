<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { ChevronDown, ChevronUp, RefreshCw } from '@lucide/svelte';
  import { wsConnectionHealth, wsLCUWebSocketConnected, wsLCUWebSocketEvents } from '$lib/stores/websocketStore';
  import { wsActions } from '$lib/stores/websocketStore';
  import { formatTimestamp } from '$lib/utils/timestamp';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';

  // $state, or the store subscriptions below mutate plain locals in runes mode
  // and the card renders its initial values forever. AppStatusBar does this right.
  let healthInfo: any = $state(null);
  let lcuWebSocketConnected = $state(false);
  let lcuEvents: any[] = $state([]);
  let refreshInterval: ReturnType<typeof setInterval> | null = null;
  let expanded = $state(false);

  const unsubscribeHealth = wsConnectionHealth.subscribe((value) => {
    healthInfo = value;
  });

  const unsubscribeLCUConnected = wsLCUWebSocketConnected.subscribe((value) => {
    lcuWebSocketConnected = value;
  });

  const unsubscribeLCUEvents = wsLCUWebSocketEvents.subscribe((value) => {
    lcuEvents = value;
  });

  onMount(async () => {
    try {
      await wsActions.getConnectionHealthInfo();
    } catch (error) {
      console.error('Failed to get initial health info:', error);
    }

    refreshInterval = setInterval(async () => {
      try {
        await wsActions.checkConnectionHealth();
        await wsActions.getConnectionHealthInfo();
      } catch (error) {
        console.error('Health check failed:', error);
      }
    }, 10000);
  });

  onDestroy(() => {
    unsubscribeHealth();
    unsubscribeLCUConnected();
    unsubscribeLCUEvents();
    if (refreshInterval) clearInterval(refreshInterval);
  });

  function normalizedHealth(h: any) {
    if (!h) return null;
    const isHealthy = h.is_healthy ?? h.healthy ?? false;
    const failures = Number(h.consecutive_failures ?? 0);
    return {
      is_healthy: isHealthy,
      websocket_connected: h.websocket_connected ?? false,
      consecutive_failures: Number.isFinite(failures) ? failures : 0,
      last_check: h.last_check,
      last_error: h.last_error ?? null,
      lcu_connected: h.lcu_connected,
      live_client_available: h.live_client_available,
    };
  }

  type NormHealth = NonNullable<ReturnType<typeof normalizedHealth>>;

  function asNormHealth(h: any): NormHealth {
    return (
      normalizedHealth(h) ?? {
        is_healthy: false,
        websocket_connected: false,
        consecutive_failures: 0,
        last_check: undefined,
        last_error: null,
        lcu_connected: undefined,
        live_client_available: undefined,
      }
    );
  }

  function getHealthStatus(h: NormHealth) {
    const wsLine = lcuWebSocketConnected || h.websocket_connected;
    if (h.is_healthy && wsLine) return 'Excellent';
    if (h.is_healthy) return 'Good';
    if (h.consecutive_failures < 3) return 'Degraded';
    return 'Poor';
  }

  function getHealthVariant(h: NormHealth): 'default' | 'secondary' | 'destructive' | 'outline' {
    const status = getHealthStatus(h);
    if (status === 'Excellent' || status === 'Good') return 'default';
    if (status === 'Degraded') return 'secondary';
    return 'destructive';
  }

  function getEventTypeColor(eventType: string) {
    switch (eventType.toLowerCase()) {
      case 'create':
        return 'text-green-500';
      case 'update':
        return 'text-blue-500';
      case 'delete':
        return 'text-red-500';
      default:
        return 'text-muted-foreground';
    }
  }

  function dot(ok: boolean | undefined) {
    return ok ? 'bg-green-500' : 'bg-red-500';
  }
</script>

<div class="border-b border-border bg-muted/30 px-3 py-1.5 text-xs">
  {#if healthInfo}
    {@const h = asNormHealth(healthInfo)}
    {@const wsOk = lcuWebSocketConnected || h.websocket_connected}
    <div class="flex items-center justify-between gap-2">
      <div class="flex flex-wrap items-center gap-x-3 gap-y-1 min-w-0">
        <span class="text-muted-foreground font-medium">Health</span>
        <Badge variant={getHealthVariant(h)} class="text-[10px] px-1.5 py-0 h-5">
          {getHealthStatus(h)}
        </Badge>
        <span class="inline-flex items-center gap-1">
          <span class="w-1.5 h-1.5 rounded-full {dot(h.is_healthy)}"></span>
          LCU
        </span>
        <span class="inline-flex items-center gap-1">
          <span class="w-1.5 h-1.5 rounded-full {dot(wsOk)}"></span>
          WS
        </span>
        {#if h.live_client_available !== undefined}
          <span class="inline-flex items-center gap-1">
            <span class="w-1.5 h-1.5 rounded-full {dot(h.live_client_available)}"></span>
            Live
          </span>
        {/if}
        {#if h.consecutive_failures > 0}
          <span class="text-destructive">{h.consecutive_failures} failures</span>
        {/if}
      </div>
      <div class="flex items-center gap-1 shrink-0">
        <Button
          variant="ghost"
          size="sm"
          class="h-6 w-6 p-0"
          onclick={() => wsActions.getConnectionHealthInfo()}
          title="Refresh health"
        >
          <RefreshCw class="w-3 h-3" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          class="h-6 w-6 p-0"
          onclick={() => (expanded = !expanded)}
          title={expanded ? 'Collapse' : 'Expand details'}
        >
          {#if expanded}
            <ChevronUp class="w-3 h-3" />
          {:else}
            <ChevronDown class="w-3 h-3" />
          {/if}
        </Button>
      </div>
    </div>

    {#if expanded}
      <div class="mt-2 pt-2 border-t border-border/60 grid grid-cols-2 sm:grid-cols-4 gap-x-4 gap-y-1 text-muted-foreground">
        <div>LCU session: <span class={h.lcu_connected ? 'text-green-600' : 'text-red-500'}>{h.lcu_connected ? 'Active' : 'None'}</span></div>
        <div>Last check: {formatTimestamp(h.last_check)}</div>
        {#if h.last_error}
          <div class="col-span-2 truncate text-red-500" title={h.last_error}>Error: {h.last_error}</div>
        {/if}
      </div>

      <div class="mt-2">
        <div class="flex items-center justify-between mb-1">
          <span class="text-muted-foreground">Recent LCU events</span>
          <span class="text-muted-foreground">{lcuEvents.length}</span>
        </div>
        {#if lcuEvents.length > 0}
          <div class="max-h-20 overflow-y-auto space-y-0.5">
            {#each lcuEvents.slice(0, 8) as event}
              <div class="flex items-center justify-between gap-2 bg-background/60 rounded px-1.5 py-0.5">
                <div class="flex items-center gap-1.5 min-w-0">
                  <span class="{getEventTypeColor(event.event_type || 'unknown')} shrink-0">
                    {event.event_type || '?'}
                  </span>
                  <span class="truncate text-muted-foreground" title={event.uri}>
                    {event.uri?.split('/').pop() || 'Unknown'}
                  </span>
                </div>
                <span class="text-muted-foreground shrink-0">{formatTimestamp(event.timestamp)}</span>
              </div>
            {/each}
          </div>
        {:else}
          <div class="text-muted-foreground py-1">No events yet</div>
        {/if}
      </div>
    {/if}
  {:else}
    <div class="text-muted-foreground">Loading connection health…</div>
  {/if}
</div>
