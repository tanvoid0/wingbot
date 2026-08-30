<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Switch } from '$lib/components/ui/switch';
  import { Select, SelectContent, SelectItem, SelectTrigger } from '$lib/components/ui/select';
  import { Separator } from '$lib/components/ui/separator';
  import { Badge } from '$lib/components/ui/badge';
  import { ScrollArea } from '$lib/components/ui/scroll-area';
  import { 
    Monitor, 
    Gamepad2, 
    Bell, 
    RefreshCw,
    Wifi,
    AlertCircle,
    Layers,
    Play,
    Key,
    CheckCircle,
    ExternalLink,
    Eye,
    EyeOff,
    Brain
  } from '@lucide/svelte';
  import { openLauncherModal, openOverlaysModal } from '$lib/stores/appModalsStore';
  import {
    appStatusBarToolbar,
    clearAppStatusBarPageState,
  } from '$lib/stores/appStatusBarStore';
  import { refreshOllamaStatus } from '$lib/stores/ollamaStatusStore';

  type ThemeOption = 'light' | 'dark' | 'system';

  function themeFromBackend(value: string): ThemeOption {
    const map: Record<string, ThemeOption> = {
      Light: 'light',
      Dark: 'dark',
      System: 'system',
    };
    return map[value] ?? 'dark';
  }

  function themeToBackend(value: ThemeOption): string {
    const map: Record<ThemeOption, string> = {
      light: 'Light',
      dark: 'Dark',
      system: 'System',
    };
    return map[value] ?? 'Dark';
  }

  let config = $state<any>(null);
  let systemInfo = $state<any>(null);
  let isLoading = $state(false);
  let hasChanges = $state(false);

  // Form state
  let autoDetectLeague = $state(true);
  let manualPort = $state<number | null>(null);
  let manualPassword = $state('');
  let theme = $state<ThemeOption>('dark');
  let language = $state('en');
  let notificationsEnabled = $state(true);
  let soundEnabled = $state(true);
  let autoRefreshEnabled = $state(true);
  let refreshInterval = $state(5);
  let riotApiEnabled = $state(false);
  let riotApiKey = $state('');
  let riotPlatform = $state('na1');
  let riotTestStatus = $state<string | null>(null);
  let riotTestLoading = $state(false);
  let matchDataStatus = $state<any>(null);
  let showRiotApiKey = $state(false);
  let ollamaEnabled = $state(false);
  let ollamaBaseUrl = $state('http://127.0.0.1:11434');
  let ollamaModelName = $state('wingbot-lol');
  let ollamaPollInterval = $state(30);
  let ollamaLiveCoach = $state(true);
  let ollamaInsightsMatchHistory = $state(true);
  let ollamaInsightsScout = $state(true);
  let ollamaInsightsRune = $state(true);
  let ollamaInsightsLobby = $state(true);
  let ollamaTestStatus = $state<string | null>(null);
  let ollamaTestLoading = $state(false);

  const platformOptions = [
    { value: 'na1', label: 'NA' },
    { value: 'euw1', label: 'EUW' },
    { value: 'eun1', label: 'EUNE' },
    { value: 'kr', label: 'KR' },
    { value: 'br1', label: 'BR' },
    { value: 'jp1', label: 'JP' },
    { value: 'oc1', label: 'OCE' },
  ];

  onMount(() => {
    void (async () => {
      await loadConfig();
      await Promise.all([loadSystemInfo(), loadMatchDataStatus()]);
    })();

    return () => clearAppStatusBarPageState();
  });

  $effect(() => {
    appStatusBarToolbar.set({
      hasChanges,
      isLoading,
      onSave: saveConfig,
      onReset: resetConfig,
    });
  });

  async function loadConfig() {
    try {
      config = await invoke('get_config');
      if (config) {
        autoDetectLeague = config.league_client?.auto_detect ?? true;
        manualPort = config.league_client?.manual_port || null;
        manualPassword = config.league_client?.manual_password || '';
        theme = themeFromBackend(config.ui?.theme || 'Dark');
        language = config.ui?.language || 'en';
        notificationsEnabled = config.notifications?.enabled ?? true;
        soundEnabled = config.notifications?.sound_enabled ?? true;
        autoRefreshEnabled = config.auto_refresh?.enabled ?? true;
        refreshInterval = config.auto_refresh?.interval_seconds || 5;
        riotApiEnabled = config.riot_api?.enabled ?? false;
        riotApiKey = config.riot_api?.api_key || '';
        riotPlatform = config.riot_api?.platform || 'na1';
        ollamaEnabled = config.ollama?.enabled ?? false;
        ollamaBaseUrl = config.ollama?.base_url || 'http://127.0.0.1:11434';
        ollamaModelName = config.ollama?.model_name || 'wingbot-lol';
        ollamaPollInterval = config.ollama?.poll_interval_sec ?? 30;
        ollamaLiveCoach = config.ollama?.live_coach ?? true;
        ollamaInsightsMatchHistory = config.ollama?.insights_match_history ?? true;
        ollamaInsightsScout = config.ollama?.insights_scout ?? true;
        ollamaInsightsRune = config.ollama?.insights_rune ?? true;
        ollamaInsightsLobby = config.ollama?.insights_lobby ?? true;
      }
    } catch (error) {
      console.error('Failed to load config:', error);
    }
  }

  async function loadMatchDataStatus() {
    try {
      matchDataStatus = await invoke('get_match_data_status');
    } catch (error) {
      console.error('Failed to load match data status:', error);
    }
  }

  async function openDeveloperPortal() {
    await openUrl('https://developer.riotgames.com/');
  }

  async function loadSystemInfo() {
    try {
      systemInfo = await invoke('get_system_info');
    } catch (error) {
      console.error('Failed to load system info:', error);
    }
  }

  async function saveConfig() {
    isLoading = true;
    try {
      const updatedConfig = {
        league_client: {
          auto_detect: autoDetectLeague,
          manual_port: manualPort,
          manual_password: manualPassword,
          connection_timeout: 30
        },
        ui: {
          theme: themeToBackend(theme),
          language: language,
          window_size: { width: 1200, height: 800 },
          navigation_rail_expanded: true
        },
        notifications: {
          enabled: notificationsEnabled,
          sound_enabled: soundEnabled,
          champion_suggestions: true,
          ability_recommendations: true,
          game_events: true
        },
        auto_refresh: {
          enabled: autoRefreshEnabled,
          interval_seconds: refreshInterval,
          champion_select: true,
          live_game: true
        },
        riot_api: {
          enabled: riotApiEnabled || !!riotApiKey.trim(),
          api_key: riotApiKey.trim() || null,
          platform: riotPlatform,
          routing_region: config?.riot_api?.routing_region || '',
        },
        ollama: currentOllamaConfig(),
      };

      await invoke('update_config', { config: updatedConfig });
      config = updatedConfig;
      hasChanges = false;

      if (riotApiKey.trim()) {
        await runRiotApiTest();
      }
      await loadMatchDataStatus();
      await refreshOllamaStatus();
    } catch (error) {
      console.error('Failed to save config:', error);
    } finally {
      isLoading = false;
    }
  }

  async function resetConfig() {
    if (confirm('Are you sure you want to reset all settings to default?')) {
      try {
        // Reset to default values
        autoDetectLeague = true;
        manualPort = null;
        manualPassword = '';
        theme = 'dark';
        language = 'en';
        notificationsEnabled = true;
        soundEnabled = true;
        autoRefreshEnabled = true;
        refreshInterval = 5;
        riotApiEnabled = false;
        riotApiKey = '';
        riotPlatform = 'na1';
        ollamaEnabled = false;
        ollamaBaseUrl = 'http://127.0.0.1:11434';
        ollamaModelName = 'wingbot-lol';
        ollamaPollInterval = 30;
        ollamaLiveCoach = true;
        ollamaInsightsMatchHistory = true;
        ollamaInsightsScout = true;
        ollamaInsightsRune = true;
        ollamaInsightsLobby = true;
        hasChanges = true;
      } catch (error) {
        console.error('Failed to reset config:', error);
      }
    }
  }

  function markAsChanged() {
    hasChanges = true;
  }

  async function testRiotApi() {
    if (hasChanges) {
      await saveConfig();
      return;
    }
    await runRiotApiTest();
  }

  async function runRiotApiTest() {
    riotTestLoading = true;
    riotTestStatus = null;
    try {
      const result = await invoke<{ ok: boolean; routing: string }>('test_riot_api_connection', {
        apiKey: riotApiKey.trim() || null,
        platform: riotPlatform,
      });
      riotTestStatus = `Connected (${result.routing} routing)`;
      await loadMatchDataStatus();
      await loadConfig();
    } catch (error) {
      riotTestStatus = error instanceof Error ? error.message : String(error);
      await loadMatchDataStatus();
    } finally {
      riotTestLoading = false;
    }
  }

  function currentOllamaConfig() {
    return {
      enabled: ollamaEnabled,
      base_url: ollamaBaseUrl.trim() || 'http://127.0.0.1:11434',
      model_name: ollamaModelName.trim() || 'wingbot-lol',
      poll_interval_sec: Math.max(5, Number(ollamaPollInterval) || 30),
      live_coach: ollamaLiveCoach,
      insights_match_history: ollamaInsightsMatchHistory,
      insights_scout: ollamaInsightsScout,
      insights_rune: ollamaInsightsRune,
      insights_lobby: ollamaInsightsLobby,
    };
  }

  async function testOllama() {
    ollamaTestLoading = true;
    ollamaTestStatus = null;
    try {
      const result = await invoke<{ ok: boolean; model: string }>('test_ollama_connection', {
        config: currentOllamaConfig(),
      });
      ollamaTestStatus = `Connected — model ${result.model} available`;
      await refreshOllamaStatus();
    } catch (error) {
      ollamaTestStatus = error instanceof Error ? error.message : String(error);
    } finally {
      ollamaTestLoading = false;
    }
  }
</script>

<div class="flex flex-col h-full bg-background">
  <ScrollArea class="flex-1 p-6">
    <div class="max-w-4xl mx-auto space-y-8">
      <!-- League Client Settings -->
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <Gamepad2 class="w-5 h-5" />
            League Client Configuration
          </CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-center justify-between">
            <div class="space-y-0.5">
              <Label>Auto-detect League Client</Label>
              <p class="text-sm text-muted-foreground">
                Automatically detect League client connection
              </p>
            </div>
            <Switch
              bind:checked={autoDetectLeague}
              onCheckedChange={markAsChanged}
            />
          </div>

          {#if !autoDetectLeague}
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="space-y-2">
                <Label for="manual-port">Manual Port</Label>
                <Input
                  id="manual-port"
                  type="number"
                  bind:value={manualPort}
                  oninput={markAsChanged}
                  placeholder="e.g., 12345"
                />
              </div>
              <div class="space-y-2">
                <Label for="manual-password">Manual Password</Label>
                <Input
                  id="manual-password"
                  type="password"
                  bind:value={manualPassword}
                  oninput={markAsChanged}
                  placeholder="Enter password"
                />
              </div>
            </div>
          {/if}

          {#if systemInfo?.league_running !== undefined}
            <div class="flex flex-wrap items-center gap-3">
              <div class="flex items-center gap-2">
                <span class="text-sm text-muted-foreground">League Status:</span>
                <Badge variant={systemInfo.league_running ? 'default' : 'secondary'}>
                  {systemInfo.league_running ? 'Running' : 'Not Running'}
                </Badge>
              </div>
              <Button variant="outline" size="sm" onclick={openLauncherModal}>
                <Play class="w-4 h-4 mr-2" />
                Launch League
              </Button>
            </div>
          {/if}
        </CardContent>
      </Card>

      <!-- In-Game Overlays -->
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <Layers class="w-5 h-5" />
            In-Game Overlays
          </CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          <p class="text-sm text-muted-foreground">
            Configure HUD overlays such as CS tracker, jungle timers, and objective timers.
          </p>
          <Button variant="outline" onclick={openOverlaysModal}>
            <Layers class="w-4 h-4 mr-2" />
            Configure Overlays
          </Button>
        </CardContent>
      </Card>

      <!-- Riot API Settings -->
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <Key class="w-5 h-5" />
            Match History &amp; Stats
          </CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          {#if matchDataStatus}
            <div class="rounded-lg border p-4 space-y-2
              {matchDataStatus.active_source === 'lcu'
                ? 'border-green-500/30 bg-green-500/10'
                : matchDataStatus.riot_key_expired
                  ? 'border-orange-500/30 bg-orange-500/10'
                  : 'border-border bg-muted/20'}">
              <div class="flex items-start gap-2">
                {#if matchDataStatus.active_source === 'lcu'}
                  <CheckCircle class="w-4 h-4 text-green-600 mt-0.5 shrink-0" />
                {:else}
                  <AlertCircle class="w-4 h-4 text-orange-500 mt-0.5 shrink-0" />
                {/if}
                <div class="space-y-1">
                  <p class="text-sm font-medium">
                    {#if matchDataStatus.active_source === 'lcu'}
                      Using League Client data
                    {:else if matchDataStatus.riot_key_expired}
                      Riot API key expired
                    {:else if matchDataStatus.riot_key_configured}
                      Using saved Riot API key
                    {:else}
                      No stats source active
                    {/if}
                  </p>
                  <p class="text-sm text-muted-foreground">{matchDataStatus.recommendation}</p>
                  {#if matchDataStatus.riot_key_configured}
                    <p class="text-xs text-muted-foreground">
                      Saved key: {matchDataStatus.riot_key_masked}
                      {#if matchDataStatus.last_verified_at}
                        · Last verified {new Date(matchDataStatus.last_verified_at).toLocaleString()}
                      {/if}
                    </p>
                  {/if}
                  {#if matchDataStatus.riot_key_expired && matchDataStatus.last_error}
                    <p class="text-xs text-destructive">{matchDataStatus.last_error}</p>
                  {/if}
                </div>
              </div>
            </div>
          {/if}

          <div class="text-sm text-muted-foreground space-y-2">
            <p>
              <strong>You usually do not need a Riot API key.</strong> While the League client is open and you are logged in,
              WingBot reads match history directly from the client — no daily setup.
            </p>
            <p>
              Only save a Riot API key if you want stats when the client is closed. Development keys expire every
              <strong>24 hours</strong>; paste a new one here when prompted (Settings remembers it until it expires).
            </p>
            <p>
              For a non-expiring key, register WingBot as a product on the
              <a
                href="https://developer.riotgames.com/"
                target="_blank"
                rel="noopener noreferrer"
                class="text-primary underline"
              >Riot Developer Portal</a>
              and apply for a Production API key.
            </p>
          </div>

          <div class="flex items-center justify-between">
            <div class="space-y-0.5">
              <Label>Riot API fallback</Label>
              <p class="text-sm text-muted-foreground">
                Only needed when the League client is closed
              </p>
            </div>
            <Switch
              bind:checked={riotApiEnabled}
              onCheckedChange={markAsChanged}
            />
          </div>

          {#if riotApiEnabled}
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="space-y-2 md:col-span-2">
                <Label for="riot-api-key">API Key</Label>
                <div class="flex gap-2">
                  <Input
                    id="riot-api-key"
                    type={showRiotApiKey ? 'text' : 'password'}
                    bind:value={riotApiKey}
                    oninput={markAsChanged}
                    placeholder="Paste RGAPI-... key (saved until it expires)"
                    class="font-mono text-sm"
                  />
                  <Button
                    type="button"
                    variant="outline"
                    size="icon"
                    class="shrink-0"
                    onclick={() => showRiotApiKey = !showRiotApiKey}
                    title={showRiotApiKey ? 'Hide API key' : 'Show API key'}
                  >
                    {#if showRiotApiKey}
                      <EyeOff class="w-4 h-4" />
                    {:else}
                      <Eye class="w-4 h-4" />
                    {/if}
                  </Button>
                </div>
                <p class="text-xs text-muted-foreground">
                  This is the exact key WingBot will send to Riot after you Save. Compare it with the key from the developer portal.
                </p>
              </div>
              <div class="space-y-2">
                <Label for="riot-platform">Platform</Label>
                <Select type="single" bind:value={riotPlatform} onValueChange={markAsChanged}>
                  <SelectTrigger>
                    <span>{platformOptions.find(p => p.value === riotPlatform)?.label ?? riotPlatform}</span>
                  </SelectTrigger>
                  <SelectContent>
                    {#each platformOptions as option}
                      <SelectItem value={option.value}>{option.label}</SelectItem>
                    {/each}
                  </SelectContent>
                </Select>
              </div>
            </div>

            <div class="flex flex-wrap items-center gap-3">
              <Button variant="outline" size="sm" onclick={openDeveloperPortal}>
                <ExternalLink class="w-4 h-4 mr-2" />
                Regenerate Key
              </Button>
              <Button variant="outline" size="sm" onclick={testRiotApi} disabled={riotTestLoading || !riotApiKey}>
                {#if riotTestLoading}
                  <RefreshCw class="w-4 h-4 mr-2 animate-spin" />
                {:else}
                  <Wifi class="w-4 h-4 mr-2" />
                {/if}
                Test Connection
              </Button>
              {#if riotTestStatus}
                <span class="text-sm {riotTestStatus.startsWith('Connected') ? 'text-muted-foreground' : 'text-destructive'}">{riotTestStatus}</span>
              {/if}
            </div>
          {/if}
        </CardContent>
      </Card>

      <!-- Ollama / AI Settings -->
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <Brain class="w-5 h-5" />
            AI Coach (Ollama)
          </CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="text-sm text-muted-foreground space-y-2">
            <p>
              WingBot uses a local <strong>wingbot-lol</strong> model served by
              <a
                href="https://ollama.com"
                target="_blank"
                rel="noopener noreferrer"
                class="text-primary underline"
              >Ollama</a>.
              Start Ollama before enabling AI recommendations.
            </p>
            <p>
              Export the model with <code class="text-xs">python export_ollama.py --project wingbot-lol</code>
              from the <code class="text-xs">llm/core</code> directory if it is not installed yet.
            </p>
          </div>

          <div class="flex items-center justify-between">
            <div class="space-y-0.5">
              <Label>Enable AI recommendations</Label>
              <p class="text-sm text-muted-foreground">
                Show live coaching tips during champ select and in-game
              </p>
            </div>
            <Switch
              bind:checked={ollamaEnabled}
              onCheckedChange={markAsChanged}
            />
          </div>

          {#if ollamaEnabled}
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="space-y-2 md:col-span-2">
                <Label for="ollama-url">Ollama URL</Label>
                <Input
                  id="ollama-url"
                  bind:value={ollamaBaseUrl}
                  oninput={markAsChanged}
                  placeholder="http://127.0.0.1:11434"
                />
              </div>
              <div class="space-y-2">
                <Label for="ollama-model">Model tag</Label>
                <Input
                  id="ollama-model"
                  bind:value={ollamaModelName}
                  oninput={markAsChanged}
                  placeholder="wingbot-lol"
                />
              </div>
              <div class="space-y-2">
                <Label for="ollama-interval">Refresh interval (seconds)</Label>
                <Input
                  id="ollama-interval"
                  type="number"
                  bind:value={ollamaPollInterval}
                  oninput={markAsChanged}
                  min="5"
                  max="300"
                />
              </div>
            </div>

            <div class="space-y-3 rounded-lg border border-border/60 p-3">
              <p class="text-sm font-medium">AI features</p>
              <div class="space-y-3">
                <div class="flex items-center justify-between gap-3">
                  <div>
                    <Label>Live coach</Label>
                    <p class="text-xs text-muted-foreground">In-game and champ select advice cards</p>
                  </div>
                  <Switch bind:checked={ollamaLiveCoach} onCheckedChange={markAsChanged} />
                </div>
                <div class="flex items-center justify-between gap-3">
                  <div>
                    <Label>Match history insights</Label>
                    <p class="text-xs text-muted-foreground">Performance patterns from recent games</p>
                  </div>
                  <Switch bind:checked={ollamaInsightsMatchHistory} onCheckedChange={markAsChanged} />
                </div>
                <div class="flex items-center justify-between gap-3">
                  <div>
                    <Label>Enemy scout insights</Label>
                    <p class="text-xs text-muted-foreground">Draft notes from enemy rank and mastery</p>
                  </div>
                  <Switch bind:checked={ollamaInsightsScout} onCheckedChange={markAsChanged} />
                </div>
                <div class="flex items-center justify-between gap-3">
                  <div>
                    <Label>Rune insights</Label>
                    <p class="text-xs text-muted-foreground">Rune page optimization tips</p>
                  </div>
                  <Switch bind:checked={ollamaInsightsRune} onCheckedChange={markAsChanged} />
                </div>
                <div class="flex items-center justify-between gap-3">
                  <div>
                    <Label>Lobby insights</Label>
                    <p class="text-xs text-muted-foreground">Premade readiness and role coverage</p>
                  </div>
                  <Switch bind:checked={ollamaInsightsLobby} onCheckedChange={markAsChanged} />
                </div>
              </div>
            </div>

            <div class="flex flex-wrap items-center gap-3">
              <Button variant="outline" size="sm" onclick={testOllama} disabled={ollamaTestLoading}>
                {#if ollamaTestLoading}
                  <RefreshCw class="w-4 h-4 mr-2 animate-spin" />
                {:else}
                  <Wifi class="w-4 h-4 mr-2" />
                {/if}
                Test Connection
              </Button>
              {#if ollamaTestStatus}
                <span class="text-sm text-muted-foreground">{ollamaTestStatus}</span>
              {/if}
            </div>
          {/if}
        </CardContent>
      </Card>

      <!-- UI Settings -->
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <Monitor class="w-5 h-5" />
            User Interface
          </CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div class="space-y-2">
              <Label for="theme">Theme</Label>
              <Select type="single" bind:value={theme} onValueChange={markAsChanged}>
                <SelectTrigger>
                  <span>{theme === 'light' ? 'Light' : theme === 'dark' ? 'Dark' : 'System'}</span>
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="light">Light</SelectItem>
                  <SelectItem value="dark">Dark</SelectItem>
                  <SelectItem value="system">System</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div class="space-y-2">
              <Label for="language">Language</Label>
              <Select type="single" bind:value={language} onValueChange={markAsChanged}>
                <SelectTrigger>
                  <span>{language === 'en' ? 'English' : language === 'es' ? 'Español' : language === 'fr' ? 'Français' : 'Deutsch'}</span>
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="en">English</SelectItem>
                  <SelectItem value="es">Español</SelectItem>
                  <SelectItem value="fr">Français</SelectItem>
                  <SelectItem value="de">Deutsch</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- Notifications Settings -->
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <Bell class="w-5 h-5" />
            Notifications
          </CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-center justify-between">
            <div class="space-y-0.5">
              <Label>Enable Notifications</Label>
              <p class="text-sm text-muted-foreground">
                Show desktop notifications for important events
              </p>
            </div>
            <Switch
              bind:checked={notificationsEnabled}
              onCheckedChange={markAsChanged}
            />
          </div>

          {#if notificationsEnabled}
            <div class="flex items-center justify-between">
              <div class="space-y-0.5">
                <Label>Sound Notifications</Label>
                <p class="text-sm text-muted-foreground">
                  Play sounds for notifications
                </p>
              </div>
              <Switch
                bind:checked={soundEnabled}
                onCheckedChange={markAsChanged}
              />
            </div>
          {/if}
        </CardContent>
      </Card>

      <!-- Auto Refresh Settings -->
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <RefreshCw class="w-5 h-5" />
            Auto Refresh
          </CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex items-center justify-between">
            <div class="space-y-0.5">
              <Label>Enable Auto Refresh</Label>
              <p class="text-sm text-muted-foreground">
                Automatically refresh game data
              </p>
            </div>
            <Switch
              bind:checked={autoRefreshEnabled}
              onCheckedChange={markAsChanged}
            />
          </div>

          {#if autoRefreshEnabled}
            <div class="space-y-2">
              <Label for="refresh-interval">Refresh Interval (seconds)</Label>
              <Input
                id="refresh-interval"
                type="number"
                bind:value={refreshInterval}
                oninput={markAsChanged}
                min="1"
                max="60"
              />
            </div>
          {/if}
        </CardContent>
      </Card>

      <!-- System Information -->
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <Wifi class="w-5 h-5" />
            System Information
          </CardTitle>
        </CardHeader>
        <CardContent>
          {#if systemInfo}
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
              <div>
                <span class="font-medium">Operating System:</span>
                <span class="ml-2 text-muted-foreground">{systemInfo.os}</span>
              </div>
              <div>
                <span class="font-medium">Architecture:</span>
                <span class="ml-2 text-muted-foreground">{systemInfo.arch}</span>
              </div>
              <div>
                <span class="font-medium">Platform:</span>
                <span class="ml-2 text-muted-foreground">{systemInfo.platform}</span>
              </div>
              <div>
                <span class="font-medium">League Client:</span>
                <Badge variant={systemInfo.league_running ? 'default' : 'secondary'} class="ml-2">
                  {systemInfo.league_running ? 'Running' : 'Not Running'}
                </Badge>
              </div>
            </div>
          {:else}
            <p class="text-muted-foreground">Loading system information...</p>
          {/if}
        </CardContent>
      </Card>
    </div>
  </ScrollArea>
</div>
