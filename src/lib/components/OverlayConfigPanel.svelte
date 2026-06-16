<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Switch } from '$lib/components/ui/switch';
	import { Slider } from '$lib/components/ui/slider';
	import {
		overlayConfigs,
		overlayActions
	} from '$lib/stores/overlayStore';
	import CSTracker from '$lib/components/overlays/CSTracker.svelte';
	import JungleTimers from '$lib/components/overlays/JungleTimers.svelte';
	import ObjectiveTimers from '$lib/components/overlays/ObjectiveTimers.svelte';
	import CounterItems from '$lib/components/overlays/CounterItems.svelte';

	let selectedOverlay = $state('csTracker');
	let isPreviewMode = $state(false);

	function toggleOverlay(overlayName: string) {
		overlayActions.toggleOverlay(overlayName);
	}

	function resetOverlayPosition(overlayName: string) {
		overlayActions.resetPosition(overlayName);
	}

	function updateOverlayConfig(overlayName: string, key: string, value: unknown) {
		overlayActions.updateConfig(overlayName, { [key]: value });
	}

	const overlayList = [
		{
			id: 'csTracker',
			name: 'CS Tracker',
			description: 'Track your CS count, CS/min, and gold from CS',
			icon: '📊'
		},
		{
			id: 'jungleTimers',
			name: 'Jungle Timers',
			description: 'Track jungle camp respawn timers',
			icon: '🌿'
		},
		{
			id: 'objectiveTimers',
			name: 'Objective Timers',
			description: 'Track Dragon, Baron, and Herald spawn timers',
			icon: '🐉'
		},
		{
			id: 'counterItems',
			name: 'Counter Items',
			description: 'Get item suggestions based on enemy team',
			icon: '🛡️'
		}
	];
</script>

<div class="space-y-4">
	<div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
		<Card class="lg:col-span-1">
			<CardHeader>
				<CardTitle>Available Overlays</CardTitle>
			</CardHeader>
			<CardContent class="space-y-3">
				{#each overlayList as overlay}
					{@const config = $overlayConfigs[overlay.id]}
					<button
						type="button"
						class="w-full p-3 rounded-lg border cursor-pointer transition-all text-left
              {selectedOverlay === overlay.id
							? 'bg-accent border-primary'
							: 'bg-muted/50 border-border hover:bg-accent/50'}"
						onclick={() => (selectedOverlay = overlay.id)}
					>
						<div class="flex items-center gap-3 mb-2">
							<span class="text-xl">{overlay.icon}</span>
							<div>
								<h3 class="font-medium text-sm">{overlay.name}</h3>
								<p class="text-muted-foreground text-xs">{overlay.description}</p>
							</div>
						</div>
						<div class="flex items-center justify-between">
							<Badge variant={config.enabled ? 'default' : 'secondary'}>
								{config.enabled ? 'Enabled' : 'Disabled'}
							</Badge>
							<Button
								size="sm"
								variant="outline"
								onclick={(e) => {
									e.stopPropagation();
									toggleOverlay(overlay.id);
								}}
							>
								{config.enabled ? 'Disable' : 'Enable'}
							</Button>
						</div>
					</button>
				{/each}
			</CardContent>
		</Card>

		<div class="lg:col-span-2 space-y-4">
			<Card>
				<CardHeader>
					<CardTitle>
						Configure {overlayList.find((o) => o.id === selectedOverlay)?.name}
					</CardTitle>
				</CardHeader>
				<CardContent class="space-y-6">
					{#if selectedOverlay}
						{@const config = $overlayConfigs[selectedOverlay]}

						<div class="flex items-center justify-between">
							<div>
								<p class="font-medium">Enable Overlay</p>
								<p class="text-muted-foreground text-sm">Show this overlay during gameplay</p>
							</div>
							<Switch
								checked={config.enabled}
								onCheckedChange={(checked) =>
									updateOverlayConfig(selectedOverlay, 'enabled', checked)}
							/>
						</div>

						<div class="space-y-4">
							<h4 class="font-medium">Position</h4>
							<div class="grid grid-cols-2 gap-4">
								<div>
									<label for="x-position" class="text-sm text-muted-foreground">X Position</label>
									<Slider
										id="x-position"
										type="single"
										value={config.x}
										onValueChange={(value: number) =>
											updateOverlayConfig(selectedOverlay, 'x', value)}
										max={1920}
										step={10}
										class="mt-2"
									/>
									<div class="text-muted-foreground text-xs mt-1">{config.x}px</div>
								</div>
								<div>
									<label for="y-position" class="text-sm text-muted-foreground">Y Position</label>
									<Slider
										id="y-position"
										type="single"
										value={config.y}
										onValueChange={(value: number) =>
											updateOverlayConfig(selectedOverlay, 'y', value)}
										max={1080}
										step={10}
										class="mt-2"
									/>
									<div class="text-muted-foreground text-xs mt-1">{config.y}px</div>
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

						<div class="space-y-4">
							<h4 class="font-medium">Appearance</h4>
							<div class="grid grid-cols-2 gap-4">
								<div>
									<label for="opacity" class="text-sm text-muted-foreground">Opacity</label>
									<Slider
										id="opacity"
										type="single"
										value={config.opacity}
										onValueChange={(value: number) =>
											updateOverlayConfig(selectedOverlay, 'opacity', value)}
										max={1}
										step={0.1}
										class="mt-2"
									/>
									<div class="text-muted-foreground text-xs mt-1">
										{Math.round(config.opacity * 100)}%
									</div>
								</div>
								<div>
									<label for="scale" class="text-sm text-muted-foreground">Scale</label>
									<Slider
										id="scale"
										type="single"
										value={config.scale}
										onValueChange={(value: number) =>
											updateOverlayConfig(selectedOverlay, 'scale', value)}
										max={2}
										min={0.5}
										step={0.1}
										class="mt-2"
									/>
									<div class="text-muted-foreground text-xs mt-1">{config.scale}x</div>
								</div>
							</div>
						</div>

						<div class="flex items-center justify-between">
							<div>
								<p class="font-medium">Preview Mode</p>
								<p class="text-muted-foreground text-sm">Show overlay preview below</p>
							</div>
							<Switch
								checked={isPreviewMode}
								onCheckedChange={(checked) => (isPreviewMode = checked)}
							/>
						</div>
					{/if}
				</CardContent>
			</Card>

			{#if isPreviewMode && selectedOverlay}
				<Card>
					<CardHeader>
						<CardTitle>Preview</CardTitle>
					</CardHeader>
					<CardContent>
						<div class="relative h-64 bg-muted rounded-lg overflow-hidden">
							<div class="absolute inset-0 flex items-center justify-center text-muted-foreground text-sm">
								Game Preview Area
							</div>
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
			{/if}
		</div>
	</div>

	<Card>
		<CardContent class="p-4">
			<div class="flex flex-wrap items-center justify-between gap-4">
				<div>
					<p class="font-medium">Quick Actions</p>
					<p class="text-muted-foreground text-sm">Common overlay operations</p>
				</div>
				<div class="flex flex-wrap gap-2">
					<Button
						variant="outline"
						size="sm"
						onclick={() => {
							overlayList.forEach((overlay) => {
								overlayActions.updateConfig(overlay.id, { enabled: true });
							});
						}}
					>
						Enable All
					</Button>
					<Button
						variant="outline"
						size="sm"
						onclick={() => {
							overlayList.forEach((overlay) => {
								overlayActions.updateConfig(overlay.id, { enabled: false });
							});
						}}
					>
						Disable All
					</Button>
					<Button
						variant="outline"
						size="sm"
						onclick={() => {
							overlayList.forEach((overlay) => {
								overlayActions.resetPosition(overlay.id);
							});
						}}
					>
						Reset All Positions
					</Button>
				</div>
			</div>
		</CardContent>
	</Card>
</div>
