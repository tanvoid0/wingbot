<script lang="ts">
  import { onMount } from 'svelte';
  import { 
    getCurrentPatchVersion, 
    getAllChampionNames, 
    getChampionImageUrl,
    refreshImageServiceCache,
    initializeImageService,
    getCacheStats,
    cleanupExpiredCache
  } from '$lib/services/imageService';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import AsyncImageWithFallback from '$lib/components/ui/AsyncImageWithFallback.svelte';
  
  let patchVersion = $state<string>('Loading...');
  let championNames = $state<string[]>([]);
  let isLoading = $state(true);
  let error = $state<string | null>(null);
  let cacheStats = $state<any>(null);
  
  onMount(async () => {
    await loadData();
  });
  
  async function loadData() {
    try {
      isLoading = true;
      error = null;
      
      patchVersion = await getCurrentPatchVersion();
      championNames = await getAllChampionNames();
      cacheStats = getCacheStats();
      
      console.log('Loaded champions:', championNames);
      console.log('Cache stats:', cacheStats);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Unknown error';
      console.error('Failed to load data:', err);
    } finally {
      isLoading = false;
    }
  }
  
  async function handleRefresh() {
    await refreshImageServiceCache();
    await loadData();
  }
  
  function handleCleanupCache() {
    cleanupExpiredCache();
    cacheStats = getCacheStats();
  }
  
  // Test with some champion names (including potential new ones)
  const testChampions = ['Ahri', 'Mel', 'Yunara', 'Aatrox', 'Yasuo', 'Lee Sin'];
</script>

<div class="container mx-auto p-6 space-y-6">
  <div class="flex items-center justify-between">
    <h1 class="text-3xl font-bold">Dynamic Image Service Test</h1>
    <div class="flex gap-2">
      <Button onclick={handleCleanupCache} variant="outline">
        Cleanup Cache
      </Button>
      <Button onclick={handleRefresh} disabled={isLoading}>
        {isLoading ? 'Loading...' : 'Refresh Cache'}
      </Button>
    </div>
  </div>
  
  {#if error}
    <Card class="border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-950">
      <CardContent class="p-4">
        <p class="text-red-600 dark:text-red-400">Error: {error}</p>
      </CardContent>
    </Card>
  {/if}
  
  <Card>
    <CardHeader>
      <CardTitle>Service Status</CardTitle>
    </CardHeader>
    <CardContent class="space-y-4">
      <div class="flex items-center gap-2">
        <span class="font-medium">Current Patch Version:</span>
        <Badge variant="outline">{patchVersion}</Badge>
      </div>
      <div class="flex items-center gap-2">
        <span class="font-medium">Total Champions Loaded:</span>
        <Badge variant="outline">{championNames.length}</Badge>
      </div>
      <div class="flex items-center gap-2">
        <span class="font-medium">Status:</span>
        <Badge variant={isLoading ? 'secondary' : 'default'}>
          {isLoading ? 'Loading...' : 'Ready'}
        </Badge>
      </div>
    </CardContent>
  </Card>
  
  {#if cacheStats}
    <Card>
      <CardHeader>
        <CardTitle>Cache Statistics</CardTitle>
      </CardHeader>
      <CardContent class="space-y-4">
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div class="text-center">
            <div class="text-2xl font-bold text-blue-600">{cacheStats.imageUrlCacheSize}</div>
            <div class="text-sm text-muted-foreground">Image URLs Cached</div>
          </div>
          <div class="text-center">
            <div class="text-2xl font-bold text-green-600">{cacheStats.championIdCacheSize}</div>
            <div class="text-sm text-muted-foreground">Champion IDs Cached</div>
          </div>
          <div class="text-center">
            <div class="text-2xl font-bold text-purple-600">{cacheStats.championDataSize}</div>
            <div class="text-sm text-muted-foreground">Champions Loaded</div>
          </div>
          <div class="text-center">
            <div class="text-2xl font-bold text-orange-600">{cacheStats.patchVersion || 'N/A'}</div>
            <div class="text-sm text-muted-foreground">Patch Version</div>
          </div>
        </div>
        <div class="text-xs text-muted-foreground">
          💡 Cache automatically expires: Image URLs (1h), Champion IDs (12h), Patch Data (24h)
        </div>
      </CardContent>
    </Card>
  {/if}
  
  <Card>
    <CardHeader>
      <CardTitle>Champion Image Test</CardTitle>
    </CardHeader>
    <CardContent>
      <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
        {#each testChampions as championName}
          <div class="text-center space-y-2">
            <div class="w-16 h-16 mx-auto rounded-lg overflow-hidden border-2 border-gray-200 dark:border-gray-700">
              <AsyncImageWithFallback
                getImageUrl={() => getChampionImageUrl(championName)}
                alt={championName}
                fallback={championName.charAt(0)}
                class="w-full h-full"
                width={64}
                height={64}
              />
            </div>
            <p class="text-sm font-medium">{championName}</p>
            <p class="text-xs text-muted-foreground">
              {championNames.includes(championName) ? 'Found' : 'Not Found'}
            </p>
          </div>
        {/each}
      </div>
    </CardContent>
  </Card>
  
  <Card>
    <CardHeader>
      <CardTitle>All Available Champions ({championNames.length})</CardTitle>
    </CardHeader>
    <CardContent>
      <div class="grid grid-cols-4 md:grid-cols-6 lg:grid-cols-8 gap-2 max-h-96 overflow-y-auto">
        {#each championNames as championName}
          <Badge variant="outline" class="text-xs justify-center">
            {championName}
          </Badge>
        {/each}
      </div>
    </CardContent>
  </Card>
</div>
