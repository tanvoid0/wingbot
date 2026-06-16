<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { 
    Home, 
    Gamepad2, 
    Settings, 
    Play,
    Layers,
    Trophy
  } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader } from '$lib/components/ui/card';
  import AppModals from '$lib/components/AppModals.svelte';
  import { initializeImageService } from '$lib/services/imageService';
  import { startWebSocketListening } from '$lib/stores/websocketStore';
  import { matchHistoryActions } from '$lib/stores/matchHistoryStore';
  import { openLauncherModal, openOverlaysModal } from '$lib/stores/appModalsStore';
  import AppStatusBar from '$lib/components/AppStatusBar.svelte';
  import '../app.css';

  let { children }: { children: any } = $props();

  let systemInfo = $state<any>(null);
  let isExpanded = $state(false);

  const navigationItems = [
    { href: '/', label: 'Dashboard', icon: Home, description: 'Overview and status' },
    { href: '/game', label: 'Game', icon: Gamepad2, description: 'Session assistant — queue, champ select, live game' },
    { href: '/match-history', label: 'Match History', icon: Trophy, description: 'Performance analysis and trends' },
    { href: '/settings', label: 'Settings', icon: Settings, description: 'Configuration' },
  ];

  $effect(() => {
    const open = $page.url.searchParams.get('open');
    if (open === 'launcher') {
      openLauncherModal();
      void goto('/', { replaceState: true, keepFocus: true });
    } else if (open === 'overlays') {
      openOverlaysModal();
      void goto('/', { replaceState: true, keepFocus: true });
    }
  });

  onMount(() => {
    let poll: ReturnType<typeof setInterval> | undefined;

    startWebSocketListening();

    async function refreshSystemInfo() {
      try {
        systemInfo = await invoke('get_system_info');
      } catch (error) {
        console.error('Failed to get system info:', error);
      }
    }

    void (async () => {
      try {
        await initializeImageService();
        await refreshSystemInfo();
        try {
          await matchHistoryActions.fetchAnalysis();
        } catch (err) {
          console.warn('[MatchHistory] Initial fetch failed:', err);
        }
      } catch (error) {
        console.error('Failed to initialize:', error);
      }
      poll = setInterval(refreshSystemInfo, 25_000);
    })();

    return () => {
      if (poll !== undefined) clearInterval(poll);
    };
  });

  function toggleNavigation() {
    isExpanded = !isExpanded;
  }
</script>

<div class="flex h-screen bg-background">
  <!-- Navigation Sidebar -->
  <Card class="flex flex-col border-r border-border transition-all duration-300 shrink-0 {isExpanded ? 'w-44' : 'w-14'}">
    <CardHeader class="border-b border-border p-2">
      <div class="flex items-center justify-between">
        {#if isExpanded}
          <div class="flex items-center gap-1.5 min-w-0">
            <div class="w-7 h-7 rounded-md bg-primary flex items-center justify-center shrink-0">
              <span class="text-primary-foreground font-bold text-xs">WB</span>
            </div>
            <h1 class="font-semibold text-sm text-foreground truncate">WingBot</h1>
          </div>
        {/if}
        <Button
          variant="ghost"
          size="sm"
          onclick={toggleNavigation}
          class="p-1 h-7 w-7 rounded-md hover:bg-accent hover:text-accent-foreground transition-colors"
          title={isExpanded ? 'Collapse sidebar' : 'Expand sidebar'}
        >
          {#if isExpanded}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
          {:else}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          {/if}
        </Button>
      </div>
    </CardHeader>

    <CardContent class="flex-1 flex flex-col py-2 px-1.5">
      <nav class="space-y-0.5">
        {#each navigationItems as item}
          <a
            href={item.href}
            title={item.label}
            class="flex items-center px-2 py-1.5 text-sm font-medium rounded-md transition-colors hover:bg-accent hover:text-accent-foreground {$page.url.pathname === item.href ? 'bg-accent text-accent-foreground' : 'text-muted-foreground'}"
          >
            {#if item.icon}
              {@const IconComponent = item.icon}
              <IconComponent class="w-4 h-4 flex-shrink-0" />
            {/if}
            {#if isExpanded}
              <span class="ml-2 truncate">{item.label}</span>
            {/if}
          </a>
        {/each}
      </nav>

      <div class="mt-auto pt-2 border-t border-border space-y-1.5">
        <div class="flex {isExpanded ? 'flex-row gap-1' : 'flex-col gap-1'}">
          <Button
            variant="outline"
            size="sm"
            class="{isExpanded ? 'flex-1' : 'w-full'} h-7 px-2"
            onclick={openLauncherModal}
            title="Launch League"
          >
            <Play class="w-3.5 h-3.5 {isExpanded ? 'mr-1' : ''}" />
            {#if isExpanded}<span class="text-xs">Launch</span>{/if}
          </Button>
          <Button
            variant="outline"
            size="sm"
            class="{isExpanded ? 'flex-1' : 'w-full'} h-7 px-2"
            onclick={openOverlaysModal}
            title="Overlay config"
          >
            <Layers class="w-3.5 h-3.5 {isExpanded ? 'mr-1' : ''}" />
            {#if isExpanded}<span class="text-xs">Overlays</span>{/if}
          </Button>
        </div>
        {#if isExpanded}
          <div class="text-[10px] text-muted-foreground px-1">
            Status shown in top bar →
          </div>
        {/if}
      </div>
    </CardContent>
  </Card>

  <!-- Main Content -->
  <main class="flex-1 flex flex-col overflow-hidden">
    <AppStatusBar />
    <div class="flex-1 overflow-auto">
      {@render children?.()}
    </div>
  </main>
</div>

<AppModals />
