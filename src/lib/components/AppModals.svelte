<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog';
	import { ScrollArea } from '$lib/components/ui/scroll-area';
	import LeagueLauncher from '$lib/components/LeagueLauncher.svelte';
	import OverlayConfigPanel from '$lib/components/OverlayConfigPanel.svelte';
	import {
		isLauncherModalOpen,
		isOverlaysModalOpen,
		closeLauncherModal,
		closeOverlaysModal
	} from '$lib/stores/appModalsStore';
</script>

<Dialog.Root
	open={$isLauncherModalOpen}
	onOpenChange={(open) => (open ? null : closeLauncherModal())}
>
	<Dialog.Content class="max-w-md gap-3">
		<Dialog.Header class="space-y-1">
			<Dialog.Title class="text-base">Launch League</Dialog.Title>
		</Dialog.Header>
		<LeagueLauncher />
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root
	open={$isOverlaysModalOpen}
	onOpenChange={(open) => (open ? null : closeOverlaysModal())}
>
	<Dialog.Content class="max-w-3xl max-h-[85vh] p-0 gap-0">
		<Dialog.Header class="px-4 pt-4 pb-2 space-y-0">
			<Dialog.Title class="text-base">In-Game Overlays</Dialog.Title>
		</Dialog.Header>
		<ScrollArea class="max-h-[calc(85vh-3.5rem)] px-4 pb-4">
			<OverlayConfigPanel />
		</ScrollArea>
	</Dialog.Content>
</Dialog.Root>
