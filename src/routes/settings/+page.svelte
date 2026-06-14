<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
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
    Settings, 
    Monitor, 
    Bot, 
    Gamepad2, 
    Bell, 
    RefreshCw,
    Save,
    RotateCcw,
    Wifi,
    AlertCircle
  } from '@lucide/svelte';

  let config = $state<any>(null);
  let systemInfo = $state<any>(null);
  let isLoading = $state(false);
  let hasChanges = $state(false);

  // Form state
  let aiServiceUrl = $state('');
  let aiServiceTimeout = $state(60);
  let autoDetectLeague = $state(true);
  let manualPort = $state<number | null>(null);
  let manualPassword = $state('');
  let theme = $state('dark');
  let language = $state('en');
  let notificationsEnabled = $state(true);
  let soundEnabled = $state(true);
  let autoRefreshEnabled = $state(true);
  let refreshInterval = $state(5);

  onMount(async () => {
    await loadConfig();
    await loadSystemInfo();
  });

  async function loadConfig() {
    try {
      config = await invoke('get_config');
      if (config) {
        aiServiceUrl = config.ai_service?.base_url || 'http://127.0.0.1:8000';
        aiServiceTimeout = config.ai_service?.timeout_seconds || 60;
        autoDetectLeague = config.league_client?.auto_detect ?? true;
        manualPort = config.league_client?.manual_port || null;
        manualPassword = config.league_client?.manual_password || '';
        theme = config.ui?.theme || 'dark';
        language = config.ui?.language || 'en';
        notificationsEnabled = config.notifications?.enabled ?? true;
        soundEnabled = config.notifications?.sound_enabled ?? true;
        autoRefreshEnabled = config.auto_refresh?.enabled ?? true;
        refreshInterval = config.auto_refresh?.interval_seconds || 5;
      }
    } catch (error) {
      console.error('Failed to load config:', error);
    }
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
        ai_service: {
          base_url: aiServiceUrl,
          timeout_seconds: aiServiceTimeout,
          model_name: 'gpt-3.5-turbo',
          max_tokens: 1000
        },
        league_client: {
          auto_detect: autoDetectLeague,
          manual_port: manualPort,
          manual_password: manualPassword,
          connection_timeout: 30
        },
        ui: {
          theme: theme,
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
        }
      };

      await invoke('update_config', { config: updatedConfig });
      hasChanges = false;
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
        aiServiceUrl = 'http://127.0.0.1:8000';
        aiServiceTimeout = 60;
        autoDetectLeague = true;
        manualPort = null;
        manualPassword = '';
        theme = 'dark';
        language = 'en';
        notificationsEnabled = true;
        soundEnabled = true;
        autoRefreshEnabled = true;
        refreshInterval = 5;
        hasChanges = true;
      } catch (error) {
        console.error('Failed to reset config:', error);
      }
    }
  }

  function markAsChanged() {
    hasChanges = true;
  }
</script>

<div class="flex flex-col h-full bg-background">
  <!-- Header -->
  <Card class="border-b border-border rounded-none">
    <CardHeader class="pb-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg bg-primary flex items-center justify-center">
            <Settings class="w-5 h-5 text-primary-foreground" />
          </div>
          <div>
            <CardTitle class="text-xl">Settings</CardTitle>
            <p class="text-sm text-muted-foreground">Configure WingBot preferences</p>
          </div>
        </div>
        <div class="flex gap-2">
          {#if hasChanges}
            <Badge variant="outline" class="text-orange-600 border-orange-600">
              <AlertCircle class="w-3 h-3 mr-1" />
              Unsaved changes
            </Badge>
          {/if}
          <Button variant="outline" size="sm" onclick={resetConfig}>
            <RotateCcw class="w-4 h-4 mr-2" />
            Reset
          </Button>
          <Button onclick={saveConfig} disabled={!hasChanges || isLoading}>
            {#if isLoading}
              <RefreshCw class="w-4 h-4 mr-2 animate-spin" />
            {:else}
              <Save class="w-4 h-4 mr-2" />
            {/if}
            Save
          </Button>
        </div>
      </div>
    </CardHeader>
  </Card>

  <!-- Settings Content -->
  <ScrollArea class="flex-1 p-6">
    <div class="max-w-4xl mx-auto space-y-8">
      <!-- AI Service Settings -->
      <Card>
        <CardHeader>
          <CardTitle class="flex items-center gap-2">
            <Bot class="w-5 h-5" />
            AI Service Configuration
          </CardTitle>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div class="space-y-2">
              <Label for="ai-url">AI Service URL</Label>
              <Input
                id="ai-url"
                bind:value={aiServiceUrl}
                oninput={markAsChanged}
                placeholder="http://127.0.0.1:8000"
              />
            </div>
            <div class="space-y-2">
              <Label for="ai-timeout">Timeout (seconds)</Label>
              <Input
                id="ai-timeout"
                type="number"
                bind:value={aiServiceTimeout}
                oninput={markAsChanged}
                min="10"
                max="300"
              />
            </div>
          </div>
          {#if systemInfo?.ai_service_status}
            <div class="flex items-center gap-2">
              <span class="text-sm text-muted-foreground">Status:</span>
              <Badge variant={systemInfo.ai_service_status === 'Connected' ? 'default' : 'destructive'}>
                {systemInfo.ai_service_status}
              </Badge>
            </div>
          {/if}
        </CardContent>
      </Card>

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
            <div class="flex items-center gap-2">
              <span class="text-sm text-muted-foreground">League Status:</span>
              <Badge variant={systemInfo.league_running ? 'default' : 'secondary'}>
                {systemInfo.league_running ? 'Running' : 'Not Running'}
              </Badge>
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
              <Select bind:value={theme} onValueChange={markAsChanged} multiple={false}>
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
              <Select bind:value={language} onValueChange={markAsChanged} multiple={false}>
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
