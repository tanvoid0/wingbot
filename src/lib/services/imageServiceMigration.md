# Image Service Migration Guide

## Overview
The image service has been updated to be **fully dynamic** instead of using hardcoded champion mappings and patch versions. This means:

✅ **Automatic updates** - New champions and patches are picked up automatically  
✅ **No more hardcoding** - No need to manually update champion lists  
✅ **Better reliability** - Always uses the latest Data Dragon data  
✅ **Future-proof** - Works with any new champions Riot releases  

## What Changed

### Before (Hardcoded)
```typescript
// ❌ OLD: Hardcoded patch version
const PATCH_VERSION = '14.24.1';

// ❌ OLD: Hardcoded champion mapping
const CHAMPION_NAME_TO_ID = {
  'Ahri': 'Ahri',
  'Lee Sin': 'LeeSin',
  // ... 200+ hardcoded entries
};

// ❌ OLD: Synchronous functions
export function getChampionImageUrl(championName: string): string | null {
  const championId = CHAMPION_NAME_TO_ID[championName];
  return `${DATA_DRAGON_BASE}/${PATCH_VERSION}/img/champion/${championId}.png`;
}
```

### After (Dynamic)
```typescript
// ✅ NEW: Dynamic patch version fetching
export async function getCurrentPatchVersion(): Promise<string> {
  // Fetches latest version from Data Dragon API
}

// ✅ NEW: Dynamic champion data fetching
export async function getChampionDataCached(): Promise<Record<string, any>> {
  // Fetches latest champion data from Data Dragon API
}

// ✅ NEW: Async functions with caching
export async function getChampionImageUrl(championName: string): Promise<string | null> {
  const championId = await getChampionId(championName);
  const patchVersion = await getCurrentPatchVersion();
  return `${DATA_DRAGON_BASE}/${patchVersion}/img/champion/${championId}.png`;
}
```

## Migration Options

### Option 1: Use Async Functions (Recommended)
```svelte
<script>
  import { getChampionImageUrl } from '$lib/services/imageService';
  import AsyncImageWithFallback from '$lib/components/ui/AsyncImageWithFallback.svelte';
</script>

<!-- ✅ NEW: Async approach -->
<AsyncImageWithFallback
  getImageUrl={() => getChampionImageUrl(championName)}
  alt={championName}
  fallback={championName.charAt(0)}
  class="w-16 h-16"
  width={64}
  height={64}
/>
```

### Option 2: Use Sync Fallback Functions
```svelte
<script>
  import { getChampionImageUrlSync } from '$lib/services/imageService';
  import ImageWithFallback from '$lib/components/ui/ImageWithFallback.svelte';
</script>

<!-- ⚠️ FALLBACK: Sync approach (less reliable) -->
<ImageWithFallback
  src={getChampionImageUrlSync(championName)}
  alt={championName}
  fallback={championName.charAt(0)}
  class="w-16 h-16"
  width={64}
  height={64}
/>
```

## Available Functions

### Async Functions (Recommended)
- `getChampionImageUrl(championName)` - Get champion image URL
- `getItemImageUrl(itemId)` - Get item image URL  
- `getRuneImageUrl(runeId)` - Get rune image URL
- `getKeystoneImageUrl(keystoneId)` - Get keystone rune image URL
- `getSummonerSpellImageUrl(spellName)` - Get summoner spell image URL

### Sync Fallback Functions
- `getChampionImageUrlSync(championName)` - Sync version with fallback
- `getItemImageUrlSync(itemId)` - Sync version with fallback
- `getRuneImageUrlSync(runeId)` - Sync version with fallback
- `getKeystoneImageUrlSync(keystoneId)` - Sync version with fallback
- `getSummonerSpellImageUrlSync(spellName)` - Sync version with fallback

### Utility Functions
- `getCurrentPatchVersion()` - Get latest patch version
- `getAllChampionNames()` - Get all available champion names
- `initializeImageService()` - Initialize the service (call once on app start)
- `refreshImageServiceCache()` - Force refresh cached data

## Benefits

### 🚀 **Automatic Updates**
- New champions like "Mel" and "Yunara" are automatically available
- No more manual updates when Riot releases new content
- Always uses the latest patch version

### 🎯 **Better Accuracy**
- Uses official Data Dragon data directly
- No more missing champions or incorrect mappings
- Handles special cases (like "Wukong" → "MonkeyKing") automatically

### ⚡ **Performance**
- 24-hour caching to avoid repeated API calls
- Smart fallbacks for offline scenarios
- Pre-loading on app initialization

### 🛠️ **Developer Experience**
- No more hardcoded lists to maintain
- Automatic error handling and fallbacks
- Easy debugging with console logs

## Testing

Visit `/image-service-test` to see the new system in action and test with different champions including new ones like "Mel" and "Yunara".

## Backward Compatibility

The old sync functions still work but are less reliable. For best results, migrate to the async approach using `AsyncImageWithFallback` component.
