<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { 
    Play, 
    Settings, 
    RefreshCw, 
    AlertCircle,
    CheckCircle,
    Clock,
    Gamepad2,
    Layers,
    Trophy
  } from '@lucide/svelte';
  import StatusCard from '$lib/components/StatusCard.svelte';
  import QuickActionCard from '$lib/components/QuickActionCard.svelte';
  import SectionCard from '$lib/components/SectionCard.svelte';
  import { Button } from '$lib/components/ui/button';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import { openLauncherModal, openOverlaysModal } from '$lib/stores/appModalsStore';
  import { formatTimestamp } from '$lib/utils/timestamp';

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
</script>

<div class="p-4 space-y-4">
  <div class="flex items-center justify-end">
    <Button
      onclick={loadData}
      disabled={isLoading}
      size="sm"
      variant="outline"
      class="flex items-center gap-1.5"
    >
      <RefreshCw class="w-3.5 h-3.5 {isLoading ? 'animate-spin' : ''}" />
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
  <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
    <StatusCard
      title="League Client"
      value={getLeagueStatus()}
      icon={Play}
      status={getLeagueStatusType()}
      actionLabel="Detect Client"
      onAction={detectLeagueClient}
    />

    <StatusCard
      title="LCU Connection"
      value={systemInfo?.lcu_connected ? 'Connected' : 'Disconnected'}
      icon={Gamepad2}
      status={systemInfo?.lcu_connected ? 'success' : 'error'}
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
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3">
      <QuickActionCard
        title="Game Session"
        description="Queue, champion select, and live game"
        icon={Gamepad2}
        href="/game"
        variant="green"
      />

      <QuickActionCard
        title="Match History"
        description="Stats, champions, and recent games"
        icon={Trophy}
        href="/match-history"
        variant="purple"
      />

      <QuickActionCard
        title="Launch League"
        description="Start the League client"
        icon={Play}
        onclick={openLauncherModal}
        variant="blue"
      />

      <QuickActionCard
        title="Overlay Config"
        description="Configure in-game HUD overlays"
        icon={Layers}
        onclick={openOverlaysModal}
        variant="blue"
      />
    </div>
  </SectionCard>

  <!-- Recent Activity -->
  <SectionCard title="Activity">
    <div class="space-y-2 text-sm">
      <div class="flex items-center text-sm text-muted-foreground">
        <Clock class="w-4 h-4 mr-2" />
        <span>Last updated: {formatTimestamp(Date.now())}</span>
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
