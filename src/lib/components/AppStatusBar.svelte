<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import {
    Brain,
    ChevronDown,
    ChevronUp,
    RotateCcw,
    Save,
    AlertCircle,
  } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import {
    wsConnectionHealth,
    wsLCUWebSocketConnected,
    wsLCUWebSocketEvents,
    wsActions,
    wsConnected,
  } from '$lib/stores/websocketStore';
  import {
    appStatusBarContext,
    appStatusBarToolbar,
  } from '$lib/stores/appStatusBarStore';
  import {
    ollamaEnabled,
    ollamaStatus,
    ollamaModelName,
    ollamaLastError,
    refreshOllamaStatus,
    startOllamaStatusPolling,
    stopOllamaStatusPolling,
  } from '$lib/stores/ollamaStatusStore';
  import { formatTimestamp } from '$lib/utils/timestamp';

  let healthInfo: any = $state(null);
  let lcuWebSocketConnected = $state(false);
  let lcuEvents: any[] = $state([]);
  let wsAppConnected = $state(false);
  let leagueRunning = $state(false);
  let lcuConnected = $state(false);
  let expanded = $state(false);
  let healthPoll: ReturnType<typeof setInterval> | undefined;

  const routeTitles: Record<string, string> = {
    '/': 'Dashboard',
    '/game': 'Game',
    '/match-history': 'Match History',
    '/settings': 'Settings',
  };

  const pageContext = $derived($appStatusBarContext);
  const toolbar = $derived($appStatusBarToolbar);
  const routeTitle = $derived(
    pageContext?.title ?? routeTitles[$page.url.pathname] ?? 'WingBot',
  );

  const unsubscribeHealth = wsConnectionHealth.subscribe((v) => {
    healthInfo = v;
  });
  const unsubscribeLcuWs = wsLCUWebSocketConnected.subscribe((v) => {
    lcuWebSocketConnected = v;
  });
  const unsubscribeLcuEvents = wsLCUWebSocketEvents.subscribe((v) => {
    lcuEvents = v;
  });
  const unsubscribeWs = wsConnected.subscribe((v) => {
    wsAppConnected = v;
  });

  function normalizedHealth(h: any) {
    if (!h) return null;
    return {
      is_healthy: h.is_healthy ?? h.healthy ?? false,
      websocket_connected: h.websocket_connected ?? false,
      consecutive_failures: Number(h.consecutive_failures ?? 0),
      last_check: h.last_check,
      last_error: h.last_error ?? null,
      lcu_connected: h.lcu_connected,
      live_client_available: h.live_client_available,
    };
  }

  function dot(ok: boolean | undefined) {
    return ok ? 'bg-green-500' : 'bg-red-500';
  }

  function ollamaDotClass(status: string) {
    switch (status) {
      case 'online':
        return 'bg-green-500';
      case 'checking':
        return 'bg-yellow-500 animate-pulse';
      case 'offline':
        return 'bg-red-500';
      default:
        return 'bg-muted-foreground/40';
    }
  }

  function ollamaLabel(status: string, enabled: boolean) {
    if (!enabled) return 'Off';
    if (status === 'checking') return '…';
    if (status === 'online') return 'OK';
    return 'Down';
  }

  function inGameStatusLabel(inMatch: boolean | undefined): string {
    if (inMatch === undefined) return 'Match';
    return inMatch ? 'In game' : 'Not in match';
  }

  function inGameStatusTitle(inMatch: boolean | undefined): string {
    if (inMatch === undefined) return 'Whether you are in an active match';
    return inMatch
      ? 'In an active match — live stats, timers, and overlays are available'
      : 'Not in a match — lobby, champ select, queue, or post-game (LCU still works)';
  }

  async function refreshSystemInfo() {
    try {
      const info = await invoke<{
        league_running?: boolean;
        lcu_connected?: boolean;
      }>('get_system_info');
      leagueRunning = info.league_running ?? false;
      lcuConnected = info.lcu_connected ?? false;
    } catch {
      leagueRunning = false;
      lcuConnected = false;
    }
  }

  async function refreshHealth() {
    try {
      await wsActions.getConnectionHealthInfo();
      await wsActions.checkConnectionHealth();
    } catch (error) {
      console.error('Health refresh failed:', error);
    }
  }

  async function refreshAll() {
    await Promise.all([refreshSystemInfo(), refreshHealth(), refreshOllamaStatus()]);
  }

  onMount(() => {
    startOllamaStatusPolling();
    void refreshAll();

    healthPoll = setInterval(() => {
      void refreshSystemInfo();
      void refreshHealth();
    }, 15_000);
  });

  onDestroy(() => {
    unsubscribeHealth();
    unsubscribeLcuWs();
    unsubscribeLcuEvents();
    unsubscribeWs();
    stopOllamaStatusPolling();
    if (healthPoll !== undefined) clearInterval(healthPoll);
  });

  const h = $derived(normalizedHealth(healthInfo));
  const wsOk = $derived(lcuWebSocketConnected || (h?.websocket_connected ?? false));
</script>

<div class="border-b border-border bg-muted/30 shrink-0 text-xs">
  <div class="flex items-center justify-between gap-2 px-3 py-1.5">
    <div class="flex items-center gap-2 min-w-0">
      <span class="font-medium text-sm text-foreground truncate">{routeTitle}</span>
      {#if pageContext?.badge}
        <Badge variant="outline" class="{pageContext.badgeClass ?? ''} text-[10px] px-1.5 py-0 h-5 shrink-0">
          {pageContext.badge}
        </Badge>
      {/if}
    </div>

    <div class="flex flex-wrap items-center justify-end gap-x-3 gap-y-1 min-w-0">
      <span class="inline-flex items-center gap-1 text-muted-foreground" title="League client">
        <span class="w-1.5 h-1.5 rounded-full {dot(leagueRunning)}"></span>
        League
      </span>
      <span class="inline-flex items-center gap-1 text-muted-foreground" title="LCU REST">
        <span class="w-1.5 h-1.5 rounded-full {dot(lcuConnected)}"></span>
        LCU
      </span>
      <span class="inline-flex items-center gap-1 text-muted-foreground" title="App WebSocket">
        <span class="w-1.5 h-1.5 rounded-full {dot(wsAppConnected)}"></span>
        WS
      </span>
      {#if h?.live_client_available !== undefined}
        <span
          class="inline-flex items-center gap-1 text-muted-foreground"
          title={inGameStatusTitle(h.live_client_available)}
        >
          <span class="w-1.5 h-1.5 rounded-full {dot(h.live_client_available)}"></span>
          {inGameStatusLabel(h.live_client_available)}
        </span>
      {/if}
      <button
        type="button"
        class="inline-flex items-center gap-1 text-muted-foreground hover:text-foreground transition-colors"
        title={$ollamaEnabled
          ? `Ollama — ${$ollamaModelName} (${$ollamaStatus})`
          : 'Ollama disabled — enable in Settings'}
        onclick={() => goto('/settings')}
      >
        <Brain class="w-3 h-3" />
        <span class="w-1.5 h-1.5 rounded-full {ollamaDotClass($ollamaStatus)}"></span>
        AI {ollamaLabel($ollamaStatus, $ollamaEnabled)}
      </button>
    </div>

    <div class="flex items-center gap-0.5 shrink-0">
      {#if toolbar?.onSave}
        {#if toolbar.hasChanges}
          <Badge variant="outline" class="text-orange-600 border-orange-600 text-[10px] px-1.5 py-0 h-5 mr-1">
            Unsaved
          </Badge>
        {/if}
        {#if toolbar.onReset}
          <Button variant="ghost" size="sm" class="h-7 px-2" onclick={() => toolbar.onReset?.()} title="Reset settings">
            <RotateCcw class="w-3.5 h-3.5" />
          </Button>
        {/if}
        <Button
          variant="ghost"
          size="sm"
          class="h-7 px-2"
          onclick={() => toolbar.onSave?.()}
          disabled={!toolbar.hasChanges || toolbar.isLoading}
          title="Save settings"
        >
          <Save class="w-3.5 h-3.5 {toolbar.isLoading ? 'animate-pulse' : ''}" />
        </Button>
      {/if}

      <Button
        variant="ghost"
        size="sm"
        class="h-7 w-7 p-0"
        onclick={() => (expanded = !expanded)}
        title={expanded ? 'Collapse details' : 'Expand details'}
      >
        {#if expanded}
          <ChevronUp class="w-3.5 h-3.5" />
        {:else}
          <ChevronDown class="w-3.5 h-3.5" />
        {/if}
      </Button>
    </div>
  </div>

  {#if expanded}
    <div class="border-t border-border/60 px-3 py-2 space-y-2 text-muted-foreground">
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-x-4 gap-y-1">
        <div>League: <span class={leagueRunning ? 'text-green-600' : 'text-red-500'}>{leagueRunning ? 'Running' : 'Off'}</span></div>
        <div>LCU REST: <span class={lcuConnected ? 'text-green-600' : 'text-red-500'}>{lcuConnected ? 'Connected' : 'Disconnected'}</span></div>
        <div>LCU WS: <span class={wsOk ? 'text-green-600' : 'text-red-500'}>{wsOk ? 'Connected' : 'Disconnected'}</span></div>
        <div>App WS: <span class={wsAppConnected ? 'text-green-600' : 'text-red-500'}>{wsAppConnected ? 'Connected' : 'Disconnected'}</span></div>
        {#if h?.live_client_available !== undefined}
          <div>
            Match:
            <span class={h.live_client_available ? 'text-green-600' : 'text-muted-foreground'}>
              {h.live_client_available ? 'In game' : 'Not in match'}
            </span>
          </div>
        {/if}
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-x-4 gap-y-1">
        <div class="flex items-start gap-1.5">
          <Brain class="w-3 h-3 mt-0.5 shrink-0" />
          <div>
            <span class="font-medium text-foreground">Ollama</span>
            {#if $ollamaEnabled}
              — {$ollamaModelName}
              <span class={$ollamaStatus === 'online' ? 'text-green-600' : $ollamaStatus === 'checking' ? 'text-yellow-600' : 'text-red-500'}>
                ({$ollamaStatus})
              </span>
            {:else}
              — disabled
            {/if}
            {#if $ollamaLastError}
              <div class="text-red-500 truncate" title={$ollamaLastError}>{$ollamaLastError}</div>
            {/if}
          </div>
        </div>

        {#if h}
          <div>
            Health check: {formatTimestamp(h.last_check)}
            {#if h.consecutive_failures > 0}
              <span class="text-destructive ml-1">({h.consecutive_failures} failures)</span>
            {/if}
            {#if h.last_error}
              <div class="text-red-500 truncate" title={h.last_error}>{h.last_error}</div>
            {/if}
          </div>
        {/if}
      </div>

      {#if lcuEvents.length > 0}
        <div>
          <div class="mb-1 font-medium text-foreground">Recent LCU events ({lcuEvents.length})</div>
          <div class="max-h-20 overflow-y-auto space-y-0.5">
            {#each lcuEvents.slice(0, 6) as event}
              <div class="flex items-center justify-between gap-2 bg-background/60 rounded px-1.5 py-0.5">
                <span class="truncate">{event.event_type ?? '?'} — {event.uri?.split('/').pop() ?? 'Unknown'}</span>
                <span class="shrink-0">{formatTimestamp(event.timestamp)}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      {#if !$ollamaEnabled}
        <div class="flex items-center gap-1.5 text-muted-foreground">
          <AlertCircle class="w-3 h-3" />
          Enable AI Coach in Settings to use live recommendations.
        </div>
      {/if}
    </div>
  {/if}
</div>
