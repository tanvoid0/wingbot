<script lang="ts">
  import { page } from '$app/stores';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { 
    Home, 
    Users, 
    Gamepad2, 
    Settings, 
    Bot,
    Play
  } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Card, CardContent, CardHeader } from '$lib/components/ui/card';
  import { initializeImageService } from '$lib/services/imageService';
  import '../app.css';

  // --- declare snippet prop for default children ---
  let { children }: { children: any } = $props();

  let systemInfo = $state<any>(null);
  let leagueStatus = $state<string>('Unknown');
  let aiServiceStatus = $state<string>('Unknown');
  let isExpanded = $state(true);

  const navigationItems = [
    { href: '/', label: 'Dashboard', icon: Home, description: 'Overview and status' },
    { href: '/league-launcher', label: 'League Launcher', icon: Play, description: 'Launch League of Legends' },
    { href: '/game', label: 'Game Assistant', icon: Gamepad2, description: 'Adaptive game interface' },
    { href: '/overlays', label: 'Overlays', icon: Users, description: 'In-game overlay configuration' },
    { href: '/ai-chat', label: 'AI Chat', icon: Bot, description: 'AI-powered advice' },
    { href: '/image-service-test', label: 'Image Service Test', icon: Settings, description: 'Test dynamic images' },
    { href: '/settings', label: 'Settings', icon: Settings, description: 'Configuration' },
  ];

  onMount(() => {
    let poll: ReturnType<typeof setInterval> | undefined;

    async function refreshSystemInfo() {
      try {
        systemInfo = await invoke('get_system_info');
        leagueStatus = systemInfo.league_running ? 'Running' : 'Not Running';
        aiServiceStatus = systemInfo.ai_service_status;
      } catch (error) {
        console.error('Failed to get system info:', error);
      }
    }

    void (async () => {
      try {
        await initializeImageService();
        await refreshSystemInfo();
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
  <Card class="flex flex-col border-r border-border transition-all duration-300 {isExpanded ? 'w-64' : 'w-16'}">
    <CardHeader class="border-b border-border p-4">
      <div class="flex items-center justify-between">
        {#if isExpanded}
          <div class="flex items-center gap-2">
            <div class="w-8 h-8 rounded-lg bg-primary flex items-center justify-center">
              <span class="text-primary-foreground font-bold text-sm">WB</span>
            </div>
            <div>
              <h1 class="font-semibold text-foreground">WingBot</h1>
              <p class="text-xs text-muted-foreground">AI League Companion</p>
            </div>
          </div>
        {/if}
        <Button
          variant="ghost"
          size="sm"
          onclick={toggleNavigation}
          class="p-2 h-auto rounded-md hover:bg-accent hover:text-accent-foreground transition-colors"
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

    <CardContent class="flex-1 py-4">
      <nav class="space-y-1">
        {#each navigationItems as item}
          <a
            href={item.href}
            class="flex items-center px-3 py-2 text-sm font-medium rounded-md transition-colors hover:bg-accent hover:text-accent-foreground {$page.url.pathname === item.href ? 'bg-accent text-accent-foreground' : 'text-muted-foreground'}"
          >
            {#if item.icon}
              {@const IconComponent = item.icon}
              <IconComponent class="w-4 h-4 flex-shrink-0" />
            {/if}
            {#if isExpanded}
              <span class="ml-3">{item.label}</span>
            {/if}
          </a>
        {/each}
      </nav>

      {#if isExpanded}
        <div class="mt-auto pt-4 border-t border-border space-y-2">
          <div class="flex items-center justify-between text-xs">
            <span class="text-muted-foreground">League:</span>
            <Badge variant={leagueStatus === 'Running' ? 'default' : 'destructive'} class="text-xs">
              {leagueStatus}
            </Badge>
          </div>
          <div class="flex items-center justify-between text-xs">
            <span class="text-muted-foreground">AI Service:</span>
            <Badge variant={aiServiceStatus === 'Connected' ? 'default' : 'destructive'} class="text-xs">
              {aiServiceStatus}
            </Badge>
          </div>
        </div>
      {/if}
    </CardContent>
  </Card>

  <!-- Main Content -->
  <main class="flex-1 overflow-auto">
    {@render children?.()}
  </main>
</div>
