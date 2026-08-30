/**
 * Image Service for League of Legends assets
 * Uses Riot's Data Dragon CDN for champion and item images
 * Dynamically fetches latest patch version and champion data
 */

// Data Dragon CDN base URL
const DATA_DRAGON_BASE = 'https://ddragon.leagueoflegends.com/cdn';

// Community Dragon base URL (more up-to-date assets)
const COMMUNITY_DRAGON_BASE = 'https://raw.communitydragon.org/latest/game/assets/spells/icons2d';

// Enhanced caching system
interface CacheEntry<T> {
  data: T;
  timestamp: number;
  expiresAt: number;
}

interface ImageUrlCache {
  [key: string]: CacheEntry<string | null>;
}

// Cache for dynamic data
let cachedPatchVersion: string | null = null;
let cachedChampionData: Record<string, any> | null = null;
let cacheTimestamp: number = 0;

// Image URL cache for frequently requested images
let imageUrlCache: ImageUrlCache = {};

// Cache durations (in milliseconds)
const CACHE_DURATION = 24 * 60 * 60 * 1000; // 24 hours for patch/champion data
const IMAGE_URL_CACHE_DURATION = 60 * 60 * 1000; // 1 hour for image URLs
const CHAMPION_ID_CACHE_DURATION = 12 * 60 * 60 * 1000; // 12 hours for champion ID lookups

// Champion ID cache for faster lookups
let championIdCache: Record<string, CacheEntry<string | null>> = {};

/**
 * Cache management utilities
 */
function isCacheValid<T>(entry: CacheEntry<T> | undefined): boolean {
  if (!entry) return false;
  return Date.now() < entry.expiresAt;
}

function createCacheEntry<T>(data: T, duration: number): CacheEntry<T> {
  const now = Date.now();
  return {
    data,
    timestamp: now,
    expiresAt: now + duration
  };
}

function getCachedData<T>(cache: Record<string, CacheEntry<T>>, key: string): T | null {
  const entry = cache[key];
  if (isCacheValid(entry)) {
    return entry.data;
  }
  return null;
}

function setCachedData<T>(cache: Record<string, CacheEntry<T>>, key: string, data: T, duration: number): void {
  cache[key] = createCacheEntry(data, duration);
}

function clearExpiredCache<T>(cache: Record<string, CacheEntry<T>>): void {
  const now = Date.now();
  Object.keys(cache).forEach(key => {
    if (cache[key].expiresAt < now) {
      delete cache[key];
    }
  });
}

/**
 * Get the latest patch version from Data Dragon
 * @returns Promise<string> - Latest patch version
 */
async function getLatestPatchVersion(): Promise<string> {
  try {
    const response = await fetch('https://ddragon.leagueoflegends.com/api/versions.json');
    const versions = await response.json();
    return versions[0]; // First version is the latest
  } catch (error) {
    console.warn('Failed to fetch latest patch version, using fallback:', error);
    return '15.13.1'; // Fallback version
  }
}

/**
 * Get champion data from Data Dragon
 * @param patchVersion - The patch version to use
 * @returns Promise<Record<string, any>> - Champion data
 */
async function getChampionData(patchVersion: string): Promise<Record<string, any>> {
  try {
    const response = await fetch(`${DATA_DRAGON_BASE}/${patchVersion}/data/en_US/champion.json`);
    const data = await response.json();
    return data.data;
  } catch (error) {
    console.warn('Failed to fetch champion data:', error);
    return {};
  }
}

/**
 * Get current patch version (cached)
 * @returns Promise<string> - Current patch version
 */
export async function getCurrentPatchVersion(): Promise<string> {
  const now = Date.now();
  
  // Return cached version if still valid
  if (cachedPatchVersion && (now - cacheTimestamp) < CACHE_DURATION) {
    return cachedPatchVersion;
  }
  
  // Fetch new version
  cachedPatchVersion = await getLatestPatchVersion();
  cacheTimestamp = now;
  
  return cachedPatchVersion;
}

/**
 * Get champion data (cached)
 * @returns Promise<Record<string, any>> - Champion data
 */
export async function getChampionDataCached(): Promise<Record<string, any>> {
  const now = Date.now();
  
  // Return cached data if still valid
  if (cachedChampionData && (now - cacheTimestamp) < CACHE_DURATION) {
    return cachedChampionData;
  }
  
  // Fetch new data
  const patchVersion = await getCurrentPatchVersion();
  cachedChampionData = await getChampionData(patchVersion);
  
  return cachedChampionData;
}

// Summoner spell name to key mapping
const SUMMONER_SPELL_MAPPING: Record<string, string> = {
  'Flash': 'SummonerFlash',
  'Ignite': 'SummonerDot',
  'Exhaust': 'SummonerExhaust',
  'Heal': 'SummonerHeal',
  'Barrier': 'SummonerBarrier',
  'Cleanse': 'SummonerBoost',
  'Ghost': 'SummonerHaste',
  'Teleport': 'SummonerTeleport',
  'Smite': 'SummonerSmite',
  // Updated smite variants (2023+) - using Community Dragon
  'Primal Smite': 'smite_red',
  'Unleashed Smite': 'smite_red', // Alternative name
  'Chilling Smite': 'smite_blue',
  'Avatar Smite': 'smite_blue', // Alternative name
  // Trinkets
  'Farsight Alteration': 'SummonerOdinRecall',
  'Oracle Lens': 'SummonerOdinRecall',
  'Stealth Ward': 'SummonerOdinRecall',
  // Riot internal / spectator API display names
  'Summoner Boost': 'SummonerBoost',
  'Summoner Exhaust': 'SummonerExhaust',
  'Summoner Flash': 'SummonerFlash',
  'Summoner Haste': 'SummonerHaste',
  'Summoner Heal': 'SummonerHeal',
  'Summoner Smite': 'SummonerSmite',
  'Summoner Teleport': 'SummonerTeleport',
  'Summoner Mana': 'SummonerMana',
  'Summoner Ignite': 'SummonerDot',
  'Summoner Barrier': 'SummonerBarrier',
  'Summoner Snowball': 'SummonerSnowball',
  Mark: 'SummonerSnowball',
};

/** LCU / champ-select spell IDs → Data Dragon asset keys. */
const SUMMONER_SPELL_ID_TO_KEY: Record<number, string> = {
  1: 'SummonerBoost',
  3: 'SummonerExhaust',
  4: 'SummonerFlash',
  6: 'SummonerHaste',
  7: 'SummonerHeal',
  11: 'SummonerSmite',
  12: 'SummonerTeleport',
  13: 'SummonerMana',
  14: 'SummonerDot',
  21: 'SummonerBarrier',
  32: 'SummonerSnowball',
};

// Role to icon mapping (using reliable emoji icons)
const ROLE_ICONS: Record<string, string> = {
  'TOP': '🔱',
  'JUNGLE': '🗡️',
  'MIDDLE': '⚔️',
  'BOTTOM': '🏹',
  'UTILITY': '🛡️'
};

/**
 * Get champion ID from champion name using dynamic data with caching
 * @param championName - The champion display name
 * @returns Promise<string | null> - Champion ID or null if not found
 */
async function getChampionId(championName: string): Promise<string | null> {
  const cacheKey = championName.toLowerCase();
  
  // Check cache first
  const cachedId = getCachedData(championIdCache, cacheKey);
  if (cachedId !== null) {
    return cachedId;
  }
  
  try {
    const championData = await getChampionDataCached();
    
    // Search for champion by name (case-insensitive)
    const champion = Object.values(championData).find((champ: any) => 
      champ.name.toLowerCase() === championName.toLowerCase()
    );
    
    const championId = champion ? champion.id : null;
    
    // Cache the result
    setCachedData(championIdCache, cacheKey, championId, CHAMPION_ID_CACHE_DURATION);
    
    return championId;
  } catch (error) {
    console.warn('Failed to get champion ID:', error);
    return null;
  }
}

/**
 * Get champion image URL from Data Dragon CDN (async version with caching)
 * @param championName - The champion name (e.g., "Ahri", "Lee Sin")
 * @returns Promise<string | null> - URL to champion image or null if not found
 */
export async function getChampionImageUrl(championName: string | null | undefined): Promise<string | null> {
  if (!championName) return null;
  
  const cacheKey = `champion:${championName.toLowerCase()}`;
  
  // Check image URL cache first
  const cachedUrl = getCachedData(imageUrlCache, cacheKey);
  if (cachedUrl !== null) {
    return cachedUrl;
  }
  
  try {
    const championId = await getChampionId(championName);
    if (!championId) {
      // Cache null result to avoid repeated failed lookups
      setCachedData(imageUrlCache, cacheKey, null, IMAGE_URL_CACHE_DURATION);
      return null;
    }
    
    const patchVersion = await getCurrentPatchVersion();
    const imageUrl = `${DATA_DRAGON_BASE}/${patchVersion}/img/champion/${championId}.png`;
    
    // Cache the successful result
    setCachedData(imageUrlCache, cacheKey, imageUrl, IMAGE_URL_CACHE_DURATION);
    
    return imageUrl;
  } catch (error) {
    console.warn('Failed to get champion image URL:', error);
    return null;
  }
}

/**
 * Get champion image URL from Data Dragon CDN (sync version with fallback)
 * @param championName - The champion name (e.g., "Ahri", "Lee Sin")
 * @returns string | null - URL to champion image or null if not found
 */
export function getChampionImageUrlSync(championName: string | null | undefined): string | null {
  if (!championName) return null;
  
  // For sync version, we'll use a fallback approach
  // This is less ideal but maintains backward compatibility
  const fallbackChampionId = championName.replace(/\s+/g, '').replace(/[^a-zA-Z0-9]/g, '');
  return `${DATA_DRAGON_BASE}/15.13.1/img/champion/${fallbackChampionId}.png`;
}

const COMMUNITY_DRAGON_CHAMPION_ICON =
  'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons';

/** Champion square icon by numeric Riot champion ID (LCU format). */
export function getChampionImageUrlByIdSync(
  championId: number | null | undefined,
): string | null {
  if (!championId || championId <= 0) return null;
  return `${COMMUNITY_DRAGON_CHAMPION_ICON}/${championId}.png`;
}

/** Summoner spell icon by numeric spell ID (LCU format). */
export function getSummonerSpellImageUrlByIdSync(
  spellId: number | null | undefined,
): string | null {
  if (!spellId || spellId <= 0) return null;
  const spellKey = SUMMONER_SPELL_ID_TO_KEY[spellId];
  if (!spellKey) return null;
  return `${DATA_DRAGON_BASE}/15.13.1/img/spell/${spellKey}.png`;
}

/**
 * Get item image URL from Data Dragon CDN (async version with caching)
 * @param itemId - The item ID (e.g., 1001, 3006)
 * @returns Promise<string | null> - URL to item image or null if invalid
 */
export async function getItemImageUrl(itemId: number | string | null | undefined): Promise<string | null> {
  if (!itemId || itemId === 0) return null;
  
  const cacheKey = `item:${itemId}`;
  
  // Check image URL cache first
  const cachedUrl = getCachedData(imageUrlCache, cacheKey);
  if (cachedUrl !== null) {
    return cachedUrl;
  }
  
  try {
    const patchVersion = await getCurrentPatchVersion();
    const imageUrl = `${DATA_DRAGON_BASE}/${patchVersion}/img/item/${itemId}.png`;
    
    // Cache the result
    setCachedData(imageUrlCache, cacheKey, imageUrl, IMAGE_URL_CACHE_DURATION);
    
    return imageUrl;
  } catch (error) {
    console.warn('Failed to get item image URL:', error);
    return null;
  }
}

/**
 * Get item image URL from Data Dragon CDN (sync version with fallback)
 * @param itemId - The item ID (e.g., 1001, 3006)
 * @returns string | null - URL to item image or null if invalid
 */
export function getItemImageUrlSync(itemId: number | string | null | undefined): string | null {
  if (!itemId || itemId === 0) return null;
  
  return `${DATA_DRAGON_BASE}/15.13.1/img/item/${itemId}.png`;
}

/**
 * Get champion square image URL (smaller, square version) - async version
 * @param championName - The champion name
 * @returns Promise<string | null> - URL to champion square image or null if not found
 */
export async function getChampionSquareImageUrl(championName: string | null | undefined): Promise<string | null> {
  // Champion square images use the same URL as regular champion images
  return await getChampionImageUrl(championName);
}

/**
 * Get champion square image URL (smaller, square version) - sync version
 * @param championName - The champion name
 * @returns string | null - URL to champion square image or null if not found
 */
export function getChampionSquareImageUrlSync(championName: string | null | undefined): string | null {
  // Champion square images use the same URL as regular champion images
  return getChampionImageUrlSync(championName);
}


/**
 * Get rune image URL (async version with caching)
 * @param runeId - The rune ID
 * @returns Promise<string | null> - URL to rune image or null if invalid
 */
export async function getRuneImageUrl(runeId: number | string | null | undefined): Promise<string | null> {
  if (!runeId) return null;
  
  const cacheKey = `rune:${runeId}`;
  
  // Check image URL cache first
  const cachedUrl = getCachedData(imageUrlCache, cacheKey);
  if (cachedUrl !== null) {
    return cachedUrl;
  }
  
  try {
    const patchVersion = await getCurrentPatchVersion();
    const imageUrl = `${DATA_DRAGON_BASE}/${patchVersion}/img/${runeId}.png`;
    
    // Cache the result
    setCachedData(imageUrlCache, cacheKey, imageUrl, IMAGE_URL_CACHE_DURATION);
    
    return imageUrl;
  } catch (error) {
    console.warn('Failed to get rune image URL:', error);
    return null;
  }
}

/**
 * Get rune image URL (sync version with fallback)
 * @param runeId - The rune ID
 * @returns string | null - URL to rune image or null if invalid
 */
export function getRuneImageUrlSync(runeId: number | string | null | undefined): string | null {
  if (!runeId) return null;
  
  return `${DATA_DRAGON_BASE}/15.13.1/img/${runeId}.png`;
}

/**
 * Get keystone rune image URL (async version with caching)
 * @param keystoneId - The keystone rune ID
 * @returns Promise<string> - URL to keystone rune image
 */
export async function getKeystoneImageUrl(keystoneId: number | undefined): Promise<string> {
  if (!keystoneId) return '';
  
  const cacheKey = `keystone:${keystoneId}`;
  
  // Check image URL cache first
  const cachedUrl = getCachedData(imageUrlCache, cacheKey);
  if (cachedUrl !== null) {
    return cachedUrl;
  }
  
  try {
    const patchVersion = await getCurrentPatchVersion();
    const imageUrl = `${DATA_DRAGON_BASE}/${patchVersion}/img/${keystoneId}.png`;
    
    // Cache the result
    setCachedData(imageUrlCache, cacheKey, imageUrl, IMAGE_URL_CACHE_DURATION);
    
    return imageUrl;
  } catch (error) {
    console.warn('Failed to get keystone image URL:', error);
    return '';
  }
}

/**
 * Get keystone rune image URL (sync version with fallback)
 * @param keystoneId - The keystone rune ID
 * @returns string - URL to keystone rune image
 */
export function getKeystoneImageUrlSync(keystoneId: number | undefined): string {
  if (!keystoneId) return '';
  
  return `${DATA_DRAGON_BASE}/15.13.1/img/${keystoneId}.png`;
}

/**
 * Get keystone rune fallback text
 * @param keystoneName - The keystone rune display name
 * @returns Fallback text for the keystone
 */
export function getKeystoneFallback(keystoneName: string | undefined): string {
  if (!keystoneName) return '?';
  
  // Return first letter of each word
  return keystoneName
    .split(' ')
    .map(word => word.charAt(0))
    .join('')
    .toUpperCase()
    .slice(0, 3);
}

/**
 * Get summoner spell image URL (async version with caching)
 * @param spellName - The summoner spell display name
 * @returns Promise<string> - URL to summoner spell image
 */
export async function getSummonerSpellImageUrl(spellName: string | undefined): Promise<string> {
  if (!spellName) return '';
  
  const cacheKey = `spell:${spellName.toLowerCase()}`;
  
  // Check image URL cache first
  const cachedUrl = getCachedData(imageUrlCache, cacheKey);
  if (cachedUrl !== null) {
    return cachedUrl;
  }
  
  try {
    let spellKey = SUMMONER_SPELL_MAPPING[spellName];
    let baseUrl = DATA_DRAGON_BASE;
    let urlPath = '';
    
    // If no mapping found, try to construct the key from the name
    if (!spellKey) {
      // Handle special cases
      if (spellName.includes('Smite')) {
        if (spellName.includes('Primal') || spellName.includes('Unleashed')) {
          spellKey = 'smite_red';
          baseUrl = COMMUNITY_DRAGON_BASE;
          urlPath = `/${spellKey}.png`;
        } else if (spellName.includes('Chilling') || spellName.includes('Avatar')) {
          spellKey = 'smite_blue';
          baseUrl = COMMUNITY_DRAGON_BASE;
          urlPath = `/${spellKey}.png`;
        } else {
          spellKey = 'SummonerSmite';
          const patchVersion = await getCurrentPatchVersion();
          urlPath = `/${patchVersion}/img/spell/${spellKey}.png`;
        }
      } else {
        // Default fallback - try to construct from name
        spellKey = `Summoner${spellName.replace(/\s+/g, '')}`;
        const patchVersion = await getCurrentPatchVersion();
        urlPath = `/${patchVersion}/img/spell/${spellKey}.png`;
      }
    } else {
      // Check if this is a smite variant that should use Community Dragon
      if (spellKey === 'smite_red' || spellKey === 'smite_blue') {
        baseUrl = COMMUNITY_DRAGON_BASE;
        urlPath = `/${spellKey}.png`;
      } else {
        const patchVersion = await getCurrentPatchVersion();
        urlPath = `/${patchVersion}/img/spell/${spellKey}.png`;
      }
    }
    
    const url = `${baseUrl}${urlPath}`;
    
    // Cache the result
    setCachedData(imageUrlCache, cacheKey, url, IMAGE_URL_CACHE_DURATION);
    
    // Debug logging for Primal Smite
    if (spellName.includes('Smite') || spellName.includes('Primal')) {
      console.log(`Summoner Spell Debug: "${spellName}" -> "${spellKey}" -> "${url}"`);
    }
    
    return url;
  } catch (error) {
    console.warn('Failed to get summoner spell image URL:', error);
    return '';
  }
}

/**
 * Get summoner spell image URL (sync version with fallback)
 * @param spellName - The summoner spell display name
 * @returns string - URL to summoner spell image
 */
export function getSummonerSpellImageUrlSync(spellName: string | undefined): string {
  if (!spellName) return '';
  
  let spellKey = SUMMONER_SPELL_MAPPING[spellName];
  let baseUrl = DATA_DRAGON_BASE;
  let urlPath = '';
  
  // If no mapping found, try to construct the key from the name
  if (!spellKey) {
    // Handle special cases
    if (spellName.includes('Smite')) {
      if (spellName.includes('Primal') || spellName.includes('Unleashed')) {
        spellKey = 'smite_red';
        baseUrl = COMMUNITY_DRAGON_BASE;
        urlPath = `/${spellKey}.png`;
      } else if (spellName.includes('Chilling') || spellName.includes('Avatar')) {
        spellKey = 'smite_blue';
        baseUrl = COMMUNITY_DRAGON_BASE;
        urlPath = `/${spellKey}.png`;
      } else {
        spellKey = 'SummonerSmite';
        urlPath = `/15.13.1/img/spell/${spellKey}.png`;
      }
    } else {
      // Default fallback - try to construct from name
      spellKey = `Summoner${spellName.replace(/\s+/g, '')}`;
      urlPath = `/15.13.1/img/spell/${spellKey}.png`;
    }
  } else {
    // Check if this is a smite variant that should use Community Dragon
    if (spellKey === 'smite_red' || spellKey === 'smite_blue') {
      baseUrl = COMMUNITY_DRAGON_BASE;
      urlPath = `/${spellKey}.png`;
    } else {
      urlPath = `/15.13.1/img/spell/${spellKey}.png`;
    }
  }
  
  const url = `${baseUrl}${urlPath}`;
  
  // Debug logging for Primal Smite
  if (spellName.includes('Smite') || spellName.includes('Primal')) {
    console.log(`Summoner Spell Debug: "${spellName}" -> "${spellKey}" -> "${url}"`);
  }
  
  return url;
}

/**
 * Get summoner spell fallback text
 * @param spellName - The summoner spell display name
 * @returns Fallback text for the spell
 */
export function getSummonerSpellFallback(spellName: string | undefined): string {
  if (!spellName) return '?';
  
  // Return first letter of each word
  return spellName
    .split(' ')
    .map(word => word.charAt(0))
    .join('')
    .toUpperCase()
    .slice(0, 2);
}

/**
 * Get role icon URL
 * @param role - The role name (TOP, JUNGLE, etc.)
 * @returns URL to role icon image
 */
export function getRoleIconUrl(role: string | undefined): string {
  if (!role) return '';
  return ROLE_ICONS[role] || '';
}

/**
 * Get role icon fallback text
 * @param role - The role name (TOP, JUNGLE, etc.)
 * @returns Fallback text for the role
 */
export function getRoleIconFallback(role: string | undefined): string {
  if (!role) return '❓';
  
  // Return first letter of role
  return role.charAt(0);
}

/**
 * Get role icon (legacy function - now returns the role key for image component)
 * @param role - The role name (TOP, JUNGLE, etc.)
 * @returns Role key for use with image component
 */
export function getRoleIcon(role: string | undefined): string {
  if (!role) return '❓';
  return ROLE_ICONS[role] || '❓';
}

/**
 * Check if an image URL is valid by attempting to load it
 * @param url - The image URL to check
 * @returns Promise that resolves to true if image loads successfully
 */
export async function isImageUrlValid(url: string): Promise<boolean> {
  try {
    const response = await fetch(url, { method: 'HEAD' });
    return response.ok;
  } catch {
    return false;
  }
}

/**
 * Rank tier emblem URL from Community Dragon (falls back gracefully in UI).
 */
export function getRankEmblemUrl(tier: string | null | undefined): string | null {
  if (!tier) return null;
  const normalized = tier.toLowerCase().replace(/\s+/g, '');
  const valid = [
    'iron', 'bronze', 'silver', 'gold', 'platinum', 'emerald',
    'diamond', 'master', 'grandmaster', 'challenger',
  ];
  if (!valid.includes(normalized)) return null;
  return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/ranked-emblem/emblem-${normalized}.png`;
}

/**
 * Get champion name fallback (first letter)
 * @param championName - The champion name
 * @returns First letter of champion name or '?' if not available
 */
export function getChampionFallback(championName: string | null | undefined): string {
  return championName?.charAt(0) || '?';
}

/**
 * Get item fallback (item ID or '?')
 * @param itemId - The item ID
 * @returns Item ID as string or '?' if not available
 */
export function getItemFallback(itemId: number | string | null | undefined): string {
  return itemId ? String(itemId) : '?';
}

/**
 * Initialize the image service by pre-loading the latest data
 * This should be called once when the app starts
 * @returns Promise<void>
 */
export async function initializeImageService(): Promise<void> {
  try {
    console.log('Initializing Image Service...');
    
    // Clean up any expired cache entries
    cleanupExpiredCache();
    
    // Pre-load the latest patch version and champion data
    const patchVersion = await getCurrentPatchVersion();
    const championData = await getChampionDataCached();
    
    console.log(`Image Service initialized with patch version: ${patchVersion}`);
    console.log(`Loaded ${Object.keys(championData).length} champions`);
    
    // Log any new champions that might not be in the old hardcoded list
    const championNames = Object.values(championData).map((champ: any) => champ.name);
    console.log('Available champions:', championNames.slice(0, 10), '...');
    
    // Set up periodic cache cleanup (every 30 minutes)
    setInterval(cleanupExpiredCache, 30 * 60 * 1000);
    
  } catch (error) {
    console.error('Failed to initialize Image Service:', error);
  }
}

/**
 * Force refresh the cached data
 * Useful for testing or when you want to ensure you have the latest data
 * @returns Promise<void>
 */
export async function refreshImageServiceCache(): Promise<void> {
  try {
    console.log('Refreshing Image Service cache...');
    
    // Clear all caches
    cachedPatchVersion = null;
    cachedChampionData = null;
    cacheTimestamp = 0;
    imageUrlCache = {};
    championIdCache = {};
    
    // Re-initialize
    await initializeImageService();
    
  } catch (error) {
    console.error('Failed to refresh Image Service cache:', error);
  }
}

/**
 * Clean up expired cache entries
 * Should be called periodically to prevent memory leaks
 * @returns void
 */
export function cleanupExpiredCache(): void {
  clearExpiredCache(imageUrlCache);
  clearExpiredCache(championIdCache);
}

/**
 * Get cache statistics for debugging
 * @returns Object with cache statistics
 */
export function getCacheStats(): {
  imageUrlCacheSize: number;
  championIdCacheSize: number;
  patchVersion: string | null;
  championDataSize: number;
} {
  return {
    imageUrlCacheSize: Object.keys(imageUrlCache).length,
    championIdCacheSize: Object.keys(championIdCache).length,
    patchVersion: cachedPatchVersion,
    championDataSize: cachedChampionData ? Object.keys(cachedChampionData).length : 0
  };
}

/**
 * Get all available champion names (for debugging/testing)
 * @returns Promise<string[]> - Array of champion names
 */
export async function getAllChampionNames(): Promise<string[]> {
  try {
    const championData = await getChampionDataCached();
    return Object.values(championData).map((champ: any) => champ.name);
  } catch (error) {
    console.warn('Failed to get champion names:', error);
    return [];
  }
}
