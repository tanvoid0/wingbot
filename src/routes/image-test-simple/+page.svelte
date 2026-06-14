<script lang="ts">
  import { onMount } from 'svelte';
  import { getChampionImageUrlSync, getCurrentPatchVersion } from '$lib/services/imageService';
  import ImageWithFallback from '$lib/components/ui/ImageWithFallback.svelte';
  
  let patchVersion = $state<string>('Loading...');
  
  onMount(async () => {
    patchVersion = await getCurrentPatchVersion();
  });
  
  const testChampions = ['Ahri', 'Mel', 'Yunara', 'Aatrox'];
</script>

<div class="container mx-auto p-6">
  <h1 class="text-2xl font-bold mb-4">Simple Image Test</h1>
  <p class="mb-4">Patch Version: {patchVersion}</p>
  
  <div class="grid grid-cols-4 gap-4">
    {#each testChampions as championName}
      <div class="text-center">
        <div class="w-16 h-16 mx-auto mb-2 rounded-lg overflow-hidden border-2 border-gray-200">
          <ImageWithFallback
            src={getChampionImageUrlSync(championName)}
            alt={championName}
            fallback={championName.charAt(0)}
            class="w-full h-full"
            width={64}
            height={64}
          />
        </div>
        <p class="text-sm">{championName}</p>
      </div>
    {/each}
  </div>
</div>
