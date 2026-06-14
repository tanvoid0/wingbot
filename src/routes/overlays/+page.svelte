<script lang="ts">
  import { onMount } from 'svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Switch } from '$lib/components/ui/switch';
  import { Slider } from '$lib/components/ui/slider';
  import { 
    overlayConfigs, 
    overlayActions, 
    csTrackerData, 
    jungleTimers, 
    objectiveTimers, 
    counterItems 
  } from '$lib/stores/overlayStore';
  
  // Import overlay components
  import CSTracker from '$lib/components/overlays/CSTracker.svelte';
  import JungleTimers from '$lib/components/overlays/JungleTimers.svelte';
  import ObjectiveTimers from '$lib/components/overlays/ObjectiveTimers.svelte';
  import CounterItems from '$lib/components/overlays/CounterItems.svelte';

  let selectedOverlay = $state('csTracker');
  let isPreviewMode = $state(false);

  onMount(() => {
    // Initialize overlay data
    console.log('Overlay page loaded');
  });

  function toggleOverlay(overlayName: string) {
    overlayActions.toggleOverlay(overlayName);
  }

  function resetOverlayPosition(overlayName: string) {
    overlayActions.resetPosition(overlayName);
  }

  function updateOverlayConfig(overlayName: string, key: string, value: any) {
    overlayActions.updateConfig(overlayName, { [key]: value });
  }

  const overlayList = [
    {
      id: 'csTracker',
      name: 'CS Tracker',
      description: 'Track your CS count, CS/min, and gold from CS',
      icon: '📊',
      component: CSTracker
    },
    {
      id: 'jungleTimers',
      name: 'Jungle Timers',
      description: 'Track jungle camp respawn timers',
      icon: '🌿',
      component: JungleTimers
    },
    {
      id: 'objectiveTimers',
      name: 'Objective Timers',
      description: 'Track Dragon, Baron, and Herald spawn timers',
      icon: '🐉',
      component: ObjectiveTimers
    },
    {
      id: 'counterItems',
      name: 'Counter Items',
      description: 'Get item suggestions based on enemy team',
      icon: '🛡️',
      component: CounterItems
    }
  ];
</script>

<svelte:head>
  <title>Overlay Configuration - Wingbot</title>
</svelte:head>

<div class="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
  <div class="container mx-auto px-4 py-8">
    <!-- Header -->
    <div class="mb-8">
      <h1 class="text-4xl font-bold text-white mb-2">Overlay Configuration</h1>
      <p class="text-slate-300">Configure and test in-game overlays for enhanced gameplay</p>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
      <!-- Overlay List -->
      <div class="lg:col-span-1">
        <Card class="bg-slate-800/50 border-slate-700">
          <CardHeader>
            <CardTitle class="text-white">Available Overlays</CardTitle>
          </CardHeader>
          <CardContent class="space-y-4">
            {#each overlayList as overlay}
              {@const config = $overlayConfigs[overlay.id]}
              <button 
                type="button"
                class="w-full p-4 rounded-lg border cursor-pointer transition-all text-left
                  {selectedOverlay === overlay.id ? 'bg-blue-500/20 border-blue-500' : 'bg-slate-700/50 border-slate-600 hover:bg-slate-700'}"
                onclick={() => selectedOverlay = overlay.id}
              >
                <div class="flex items-center gap-3 mb-2">
                  <span class="text-2xl">{overlay.icon}</span>
                  <div>
                    <h3 class="text-white font-medium">{overlay.name}</h3>
                    <p class="text-slate-400 text-sm">{overlay.description}</p>
                  </div>
                </div>
                
                <div class="flex items-center justify-between">
                  <Badge variant={config.enabled ? 'default' : 'secondary'}>
                    {config.enabled ? 'Enabled' : 'Disabled'}
                  </Badge>
                  <Button 
                    size="sm" 
                    variant="outline"
                    onclick={() => toggleOverlay(overlay.id)}
                  >
                    {config.enabled ? 'Disable' : 'Enable'}
                  </Button>
                </div>
              </button>
            {/each}
          </CardContent>
        </Card>
      </div>

      <!-- Configuration Panel -->
      <div class="lg:col-span-2">
        <Card class="bg-slate-800/50 border-slate-700">
          <CardHeader>
            <CardTitle class="text-white">
              Configure {overlayList.find(o => o.id === selectedOverlay)?.name}
            </CardTitle>
          </CardHeader>
          <CardContent class="space-y-6">
            {#if selectedOverlay}
              {@const config = $overlayConfigs[selectedOverlay]}
              {@const overlay = overlayList.find(o => o.id === selectedOverlay)}
              
              <!-- Enable/Disable -->
              <div class="flex items-center justify-between">
                <div>
                  <label class="text-white font-medium">Enable Overlay</label>
                  <p class="text-slate-400 text-sm">Show this overlay during gameplay</p>
                </div>
                <Switch 
                  checked={config.enabled}
                  onCheckedChange={(checked) => updateOverlayConfig(selectedOverlay, 'enabled', checked)}
                />
              </div>

              <!-- Position -->
              <div class="space-y-4">
                <h4 class="text-white font-medium">Position</h4>
                <div class="grid grid-cols-2 gap-4">
                  <div>
                    <label for="x-position" class="text-slate-300 text-sm">X Position</label>
                    <Slider 
                      id="x-position"
                      type="single"
                      value={config.x}
                      onValueChange={(value: number) => updateOverlayConfig(selectedOverlay, 'x', value)}
                      max={1920}
                      step={10}
                      class="mt-2"
                    />
                    <div class="text-slate-400 text-xs mt-1">{config.x}px</div>
                  </div>
                  <div>
                    <label for="y-position" class="text-slate-300 text-sm">Y Position</label>
                    <Slider 
                      id="y-position"
                      type="single"
                      value={config.y}
                      onValueChange={(value: number) => updateOverlayConfig(selectedOverlay, 'y', value)}
                      max={1080}
                      step={10}
                      class="mt-2"
                    />
                    <div class="text-slate-400 text-xs mt-1">{config.y}px</div>
                  </div>
                </div>
                <Button 
                  variant="outline" 
                  size="sm"
                  onclick={() => resetOverlayPosition(selectedOverlay)}
                >
                  Reset Position
                </Button>
              </div>

              <!-- Appearance -->
              <div class="space-y-4">
                <h4 class="text-white font-medium">Appearance</h4>
                <div class="grid grid-cols-2 gap-4">
                  <div>
                    <label for="opacity" class="text-slate-300 text-sm">Opacity</label>
                    <Slider 
                      id="opacity"
                      type="single"
                      value={config.opacity}
                      onValueChange={(value: number) => updateOverlayConfig(selectedOverlay, 'opacity', value)}
                      max={1}
                      step={0.1}
                      class="mt-2"
                    />
                    <div class="text-slate-400 text-xs mt-1">{Math.round(config.opacity * 100)}%</div>
                  </div>
                  <div>
                    <label for="scale" class="text-slate-300 text-sm">Scale</label>
                    <Slider 
                      id="scale"
                      type="single"
                      value={config.scale}
                      onValueChange={(value: number) => updateOverlayConfig(selectedOverlay, 'scale', value)}
                      max={2}
                      min={0.5}
                      step={0.1}
                      class="mt-2"
                    />
                    <div class="text-slate-400 text-xs mt-1">{config.scale}x</div>
                  </div>
                </div>
              </div>

              <!-- Preview Toggle -->
              <div class="flex items-center justify-between">
                <div>
                  <label class="text-white font-medium">Preview Mode</label>
                  <p class="text-slate-400 text-sm">Show overlay preview in this panel</p>
                </div>
                <Switch 
                  checked={isPreviewMode}
                  onCheckedChange={(checked) => isPreviewMode = checked}
                />
              </div>
            {/if}
          </CardContent>
        </Card>

        <!-- Preview Area -->
        {#if isPreviewMode && selectedOverlay}
          <div class="mt-6">
            <Card class="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle class="text-white">Preview</CardTitle>
              </CardHeader>
              <CardContent>
                <div class="relative h-96 bg-slate-900 rounded-lg overflow-hidden">
                  <div class="absolute inset-0 bg-gradient-to-br from-slate-800 to-slate-900"></div>
                  <div class="absolute inset-0 flex items-center justify-center text-slate-500">
                    Game Preview Area
                  </div>
                  
                  <!-- Render selected overlay component -->
                  {#if selectedOverlay === 'csTracker'}
                    <CSTracker />
                  {:else if selectedOverlay === 'jungleTimers'}
                    <JungleTimers />
                  {:else if selectedOverlay === 'objectiveTimers'}
                    <ObjectiveTimers />
                  {:else if selectedOverlay === 'counterItems'}
                    <CounterItems />
                  {/if}
                </div>
              </CardContent>
            </Card>
          </div>
        {/if}
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="mt-8">
      <Card class="bg-slate-800/50 border-slate-700">
        <CardContent class="p-6">
          <div class="flex items-center justify-between">
            <div>
              <h3 class="text-white font-medium">Quick Actions</h3>
              <p class="text-slate-400 text-sm">Common overlay operations</p>
            </div>
            <div class="flex gap-2">
              <Button variant="outline" onclick={() => {
                overlayList.forEach(overlay => {
                  overlayActions.updateConfig(overlay.id, { enabled: true });
                });
              }}>
                Enable All
              </Button>
              <Button variant="outline" onclick={() => {
                overlayList.forEach(overlay => {
                  overlayActions.updateConfig(overlay.id, { enabled: false });
                });
              }}>
                Disable All
              </Button>
              <Button variant="outline" onclick={() => {
                overlayList.forEach(overlay => {
                  overlayActions.resetPosition(overlay.id);
                });
              }}>
                Reset All Positions
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  </div>
</div>
