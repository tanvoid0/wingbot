<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { 
    Play, 
    Users, 
    Bot, 
    Settings, 
    RefreshCw, 
    AlertCircle,
    CheckCircle,
    Clock,
    Gamepad2
  } from '@lucide/svelte';
  import StatusCard from '$lib/components/StatusCard.svelte';
  import QuickActionCard from '$lib/components/QuickActionCard.svelte';
  import SectionCard from '$lib/components/SectionCard.svelte';
  import { Button } from '$lib/components/ui/button';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';

  let systemInfo = $state<any>(null);
  let gameState = $state<any>(null);
  let isLoading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    try {
      isLoading = true;
      error = null;
      
      const [sysInfo, game] = await Promise.all([
        invoke('get_system_info'),
        invoke('get_game_state')
      ]);
      
      systemInfo = sysInfo;
      gameState = game;
    } catch (err) {
      error = err as string;
      console.error('Failed to load data:', err);
    } finally {
      isLoading = false;
    }
  }

  async function detectLeagueClient() {
    try {
      await invoke('detect_league_client');
      await loadData(); // Refresh data
    } catch (err) {
      error = err as string;
    }
  }

  function getGameStatusText() {
    if (!gameState) return 'Unknown';
    
    switch (gameState.status) {
      case 'Idle':
        return 'Not in game';
      case 'InChampionSelect':
        return 'In champion select';
      case 'InGame':
        return 'In game';
      case 'Loading':
        return 'Loading game';
      default:
        return 'Unknown';
    }
  }

  function getLeagueStatus() {
    return systemInfo?.league_running ? 'Running' : 'Not Running';
  }

  function getLeagueStatusType() {
    return systemInfo?.league_running ? 'success' : 'error';
  }

  function getAIStatusType() {
    return systemInfo?.ai_service_status === 'Connected' ? 'success' : 'error';
  }
</script>

<div class="p-6 space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold text-foreground">WingBot Dashboard</h1>
      <p class="text-muted-foreground mt-1">Your AI-powered League of Legends companion</p>
    </div>
    <Button
      onclick={loadData}
      disabled={isLoading}
      class="flex items-center gap-2"
    >
      <RefreshCw class="w-4 h-4 {isLoading ? 'animate-spin' : ''}" />
      Refresh
    </Button>
  </div>

  {#if error}
    <Alert variant="destructive">
      <AlertCircle class="h-4 w-4" />
      <AlertDescription>Error: {error}</AlertDescription>
    </Alert>
  {/if}

  <!-- Status Cards -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
    <StatusCard
      title="League Client"
      value={getLeagueStatus()}
      icon={Play}
      status={getLeagueStatusType()}
      actionLabel="Detect Client"
      onAction={detectLeagueClient}
    />

    <StatusCard
      title="AI Service"
      value={systemInfo?.ai_service_status || 'Unknown'}
      icon={Bot}
      status={getAIStatusType()}
    />

    <StatusCard
      title="Game Status"
      value={getGameStatusText()}
      icon={Gamepad2}
      status="neutral"
    />

    <StatusCard
      title="Platform"
      value={systemInfo?.os || 'Unknown'}
      icon={Settings}
      status="neutral"
    />
  </div>

  <!-- Quick Actions -->
  <SectionCard title="Quick Actions">
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <QuickActionCard
        title="Champion Select"
        description="Get pick recommendations"
        icon={Users}
        href="/champion-select"
        variant="blue"
      />

      <QuickActionCard
        title="Live Game"
        description="In-game assistance"
        icon={Play}
        href="/live-game"
        variant="green"
      />

      <QuickActionCard
        title="AI Chat"
        description="Get AI advice"
        icon={Bot}
        href="/ai-chat"
        variant="purple"
      />
    </div>
  </SectionCard>

  <!-- Recent Activity -->
  <SectionCard title="Recent Activity">
    <div class="space-y-3">
      <div class="flex items-center text-sm text-muted-foreground">
        <Clock class="w-4 h-4 mr-2" />
        <span>Last updated: {new Date().toLocaleTimeString()}</span>
      </div>
      {#if gameState?.status === 'InChampionSelect'}
        <div class="flex items-center text-sm text-blue-600 dark:text-blue-400">
          <CheckCircle class="w-4 h-4 mr-2" />
          <span>Currently in champion select phase</span>
        </div>
      {/if}
      {#if gameState?.status === 'InGame'}
        <div class="flex items-center text-sm text-green-600 dark:text-green-400">
          <CheckCircle class="w-4 h-4 mr-2" />
          <span>Currently in game</span>
        </div>
      {/if}
    </div>
  </SectionCard>
</div>
