<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { wsConnectionHealth, wsLCUWebSocketConnected, wsLCUWebSocketEvents } from '$lib/stores/websocketStore';
  import { wsActions } from '$lib/stores/websocketStore';

  let healthInfo: any = null;
  let lcuWebSocketConnected = false;
  let lcuEvents: any[] = [];
  let refreshInterval: number | null = null;

  // Subscribe to stores
  const unsubscribeHealth = wsConnectionHealth.subscribe(value => {
    healthInfo = value;
  });

  const unsubscribeLCUConnected = wsLCUWebSocketConnected.subscribe(value => {
    lcuWebSocketConnected = value;
  });

  const unsubscribeLCUEvents = wsLCUWebSocketEvents.subscribe(value => {
    lcuEvents = value;
  });

  onMount(async () => {
    // Get initial health info
    try {
      await wsActions.getConnectionHealthInfo();
    } catch (error) {
      console.error('Failed to get initial health info:', error);
    }

    // Set up periodic health checks
    refreshInterval = setInterval(async () => {
      try {
        await wsActions.checkConnectionHealth();
        await wsActions.getConnectionHealthInfo();
      } catch (error) {
        console.error('Health check failed:', error);
      }
    }, 10000); // Check every 10 seconds
  });

  onDestroy(() => {
    unsubscribeHealth();
    unsubscribeLCUConnected();
    unsubscribeLCUEvents();
    
    if (refreshInterval) {
      clearInterval(refreshInterval);
    }
  });

  /** Align Tauri `check_connection_health` payload with WebSocket `ConnectionHealthUpdate`. */
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

  function getHealthColor() {
    const status = getHealthStatus(asNormHealth(healthInfo));
    switch (status) {
      case 'Excellent': return 'text-green-500';
      case 'Good': return 'text-blue-500';
      case 'Degraded': return 'text-yellow-500';
      case 'Poor': return 'text-red-500';
      default: return 'text-gray-500';
    }
  }

  function formatTimestamp(timestamp: unknown) {
    if (timestamp == null || timestamp === '') return '—';
    const n = Number(timestamp);
    if (Number.isFinite(n) && n > 0) {
      const ms = n > 1_000_000_000_000 ? n : n * 1000;
      const d = new Date(ms);
      return Number.isNaN(d.getTime()) ? '—' : d.toLocaleTimeString();
    }
    const d = new Date(String(timestamp));
    return Number.isNaN(d.getTime()) ? '—' : d.toLocaleTimeString();
  }

  function getEventTypeColor(eventType: string) {
    switch (eventType.toLowerCase()) {
      case 'create': return 'text-green-500';
      case 'update': return 'text-blue-500';
      case 'delete': return 'text-red-500';
      default: return 'text-gray-500';
    }
  }
</script>

<div class="connection-health-card bg-gray-800 rounded-lg p-4 mb-4">
  <div class="flex items-center justify-between mb-3">
    <h3 class="text-lg font-semibold text-white">Connection Health</h3>
    <div class="flex items-center space-x-2">
      <div class="flex items-center space-x-1">
        <div class="w-2 h-2 rounded-full {lcuWebSocketConnected ? 'bg-green-500' : 'bg-red-500'}"></div>
        <span class="text-sm text-gray-300">
          LCU WebSocket {lcuWebSocketConnected ? 'Connected' : 'Disconnected'}
        </span>
      </div>
      <button 
        on:click={() => wsActions.getConnectionHealthInfo()}
        class="px-2 py-1 bg-blue-600 text-white text-xs rounded hover:bg-blue-700"
        title="Refresh Health Info"
      >
        Refresh
      </button>
    </div>
  </div>

  {#if healthInfo}
    {@const h = asNormHealth(healthInfo)}
    {@const wsOk = lcuWebSocketConnected || h.websocket_connected}
    <div class="grid grid-cols-2 gap-4 mb-4">
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <span class="text-sm text-gray-400">Status:</span>
          <span class="text-sm font-medium {getHealthColor()}">{getHealthStatus(h)}</span>
        </div>
        <div class="flex items-center justify-between">
          <span class="text-sm text-gray-400" title="True if LCU HTTPS or Live Client API (2999) responds">
            LCU / live client:
          </span>
          <span class="text-sm {h.is_healthy ? 'text-green-500' : 'text-red-500'}">
            {h.is_healthy ? 'Reachable' : 'Unreachable'}
          </span>
        </div>
        {#if h.lcu_connected !== undefined}
          <div class="flex items-center justify-between">
            <span class="text-sm text-gray-400">LCU session:</span>
            <span class="text-sm {h.lcu_connected ? 'text-green-500' : 'text-red-500'}">
              {h.lcu_connected ? 'Active' : 'None'}
            </span>
          </div>
        {/if}
        {#if h.live_client_available !== undefined}
          <div class="flex items-center justify-between">
            <span class="text-sm text-gray-400">Live client API:</span>
            <span class="text-sm {h.live_client_available ? 'text-green-500' : 'text-red-500'}">
              {h.live_client_available ? 'Reachable' : 'Unreachable'}
            </span>
          </div>
        {/if}
        <div class="flex items-center justify-between">
          <span class="text-sm text-gray-400">WebSocket:</span>
          <span class="text-sm {wsOk ? 'text-green-500' : 'text-red-500'}">
            {wsOk ? 'Connected' : 'Disconnected'}
          </span>
        </div>
      </div>
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <span class="text-sm text-gray-400">Failures:</span>
          <span class="text-sm {h.consecutive_failures > 0 ? 'text-red-500' : 'text-green-500'}">
            {h.consecutive_failures}
          </span>
        </div>
        <div class="flex items-center justify-between">
          <span class="text-sm text-gray-400">Last Check:</span>
          <span class="text-sm text-gray-300">
            {formatTimestamp(h.last_check)}
          </span>
        </div>
        {#if h.last_error}
          <div class="flex items-center justify-between">
            <span class="text-sm text-gray-400">Last Error:</span>
            <span class="text-sm text-red-500 truncate max-w-32" title={h.last_error}>
              {h.last_error}
            </span>
          </div>
        {/if}
      </div>
    </div>
  {:else}
    <div class="text-center py-4">
      <div class="text-gray-400">Loading health information...</div>
    </div>
  {/if}

  <!-- LCU WebSocket Events -->
  <div class="mt-4">
    <div class="flex items-center justify-between mb-2">
      <h4 class="text-sm font-medium text-white">Recent LCU Events</h4>
      <span class="text-xs text-gray-400">{lcuEvents.length} events</span>
    </div>
    
    {#if lcuEvents.length > 0}
      <div class="max-h-32 overflow-y-auto space-y-1">
        {#each lcuEvents.slice(0, 10) as event}
          <div class="flex items-center justify-between text-xs bg-gray-700 rounded px-2 py-1">
            <div class="flex items-center space-x-2">
              <span class="{getEventTypeColor(event.event_type || 'unknown')}">
                {event.event_type || 'Unknown'}
              </span>
              <span class="text-gray-400 truncate max-w-32" title={event.uri}>
                {event.uri?.split('/').pop() || 'Unknown URI'}
              </span>
            </div>
            <span class="text-gray-500">
              {new Date(event.timestamp).toLocaleTimeString()}
            </span>
          </div>
        {/each}
      </div>
    {:else}
      <div class="text-center py-2 text-gray-400 text-sm">
        No LCU WebSocket events received
      </div>
    {/if}
  </div>
</div>
