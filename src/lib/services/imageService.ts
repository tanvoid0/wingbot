/**
 * Image Service for League of Legends assets
 * Uses Riot's Data Dragon CDN for champion and item images
 */

// Data Dragon CDN base URL
const DATA_DRAGON_BASE = 'https://ddragon.leagueoflegends.com/cdn';

// Community Dragon base URL (more up-to-date assets)
const COMMUNITY_DRAGON_BASE = 'https://raw.communitydragon.org/latest/game/assets/spells/icons2d';

// Current patch version (you might want to make this dynamic)
const PATCH_VERSION = '14.24.1';

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
  'Stealth Ward': 'SummonerOdinRecall'
};

// Role to icon mapping (using reliable emoji icons)
const ROLE_ICONS: Record<string, string> = {
  'TOP': '🔱',
  'JUNGLE': '🗡️',
  'MIDDLE': '⚔️',
  'BOTTOM': '🏹',
  'UTILITY': '🛡️'
};

// Champion name to ID mapping (common champions)
const CHAMPION_NAME_TO_ID: Record<string, string> = {
  'Aatrox': 'Aatrox',
  'Ahri': 'Ahri',
  'Akali': 'Akali',
  'Akshan': 'Akshan',
  'Alistar': 'Alistar',
  'Amumu': 'Amumu',
  'Anivia': 'Anivia',
  'Annie': 'Annie',
  'Aphelios': 'Aphelios',
  'Ashe': 'Ashe',
  'Azir': 'Azir',
  'Bard': 'Bard',
  'Blitzcrank': 'Blitzcrank',
  'Brand': 'Brand',
  'Braum': 'Braum',
  'Caitlyn': 'Caitlyn',
  'Camille': 'Camille',
  'Cassiopeia': 'Cassiopeia',
  'Cho\'Gath': 'Chogath',
  'Corki': 'Corki',
  'Darius': 'Darius',
  'Diana': 'Diana',
  'Draven': 'Draven',
  'Dr. Mundo': 'DrMundo',
  'Ekko': 'Ekko',
  'Elise': 'Elise',
  'Evelynn': 'Evelynn',
  'Ezreal': 'Ezreal',
  'Fiddlesticks': 'Fiddlesticks',
  'Fiora': 'Fiora',
  'Fizz': 'Fizz',
  'Galio': 'Galio',
  'Gangplank': 'Gangplank',
  'Garen': 'Garen',
  'Gnar': 'Gnar',
  'Gragas': 'Gragas',
  'Graves': 'Graves',
  'Gwen': 'Gwen',
  'Hecarim': 'Hecarim',
  'Heimerdinger': 'Heimerdinger',
  'Illaoi': 'Illaoi',
  'Irelia': 'Irelia',
  'Ivern': 'Ivern',
  'Janna': 'Janna',
  'Jarvan IV': 'JarvanIV',
  'Jax': 'Jax',
  'Jayce': 'Jayce',
  'Jhin': 'Jhin',
  'Jinx': 'Jinx',
  'Kai\'Sa': 'Kaisa',
  'Kalista': 'Kalista',
  'Karma': 'Karma',
  'Karthus': 'Karthus',
  'Kassadin': 'Kassadin',
  'Katarina': 'Katarina',
  'Kayle': 'Kayle',
  'Kayn': 'Kayn',
  'Kennen': 'Kennen',
  'Kha\'Zix': 'Khazix',
  'Kindred': 'Kindred',
  'Kled': 'Kled',
  'Kog\'Maw': 'KogMaw',
  'LeBlanc': 'Leblanc',
  'Lee Sin': 'LeeSin',
  'Leona': 'Leona',
  'Lillia': 'Lillia',
  'Lissandra': 'Lissandra',
  'Lucian': 'Lucian',
  'Lulu': 'Lulu',
  'Lux': 'Lux',
  'Malphite': 'Malphite',
  'Malzahar': 'Malzahar',
  'Maokai': 'Maokai',
  'Master Yi': 'MasterYi',
  'Miss Fortune': 'MissFortune',
  'Mordekaiser': 'Mordekaiser',
  'Morgana': 'Morgana',
  'Nami': 'Nami',
  'Nasus': 'Nasus',
  'Nautilus': 'Nautilus',
  'Neeko': 'Neeko',
  'Nidalee': 'Nidalee',
  'Nocturne': 'Nocturne',
  'Nunu & Willump': 'Nunu',
  'Olaf': 'Olaf',
  'Orianna': 'Orianna',
  'Ornn': 'Ornn',
  'Pantheon': 'Pantheon',
  'Poppy': 'Poppy',
  'Pyke': 'Pyke',
  'Qiyana': 'Qiyana',
  'Quinn': 'Quinn',
  'Rakan': 'Rakan',
  'Rammus': 'Rammus',
  'Rek\'Sai': 'RekSai',
  'Rell': 'Rell',
  'Renekton': 'Renekton',
  'Rengar': 'Rengar',
  'Riven': 'Riven',
  'Rumble': 'Rumble',
  'Ryze': 'Ryze',
  'Samira': 'Samira',
  'Sejuani': 'Sejuani',
  'Senna': 'Senna',
  'Seraphine': 'Seraphine',
  'Sett': 'Sett',
  'Shaco': 'Shaco',
  'Shen': 'Shen',
  'Shyvana': 'Shyvana',
  'Singed': 'Singed',
  'Sion': 'Sion',
  'Sivir': 'Sivir',
  'Skarner': 'Skarner',
  'Sona': 'Sona',
  'Soraka': 'Soraka',
  'Swain': 'Swain',
  'Sylas': 'Sylas',
  'Syndra': 'Syndra',
  'Tahm Kench': 'TahmKench',
  'Taliyah': 'Taliyah',
  'Talon': 'Talon',
  'Taric': 'Taric',
  'Teemo': 'Teemo',
  'Thresh': 'Thresh',
  'Tristana': 'Tristana',
  'Trundle': 'Trundle',
  'Tryndamere': 'Tryndamere',
  'Twisted Fate': 'TwistedFate',
  'Twitch': 'Twitch',
  'Udyr': 'Udyr',
  'Urgot': 'Urgot',
  'Varus': 'Varus',
  'Vayne': 'Vayne',
  'Veigar': 'Veigar',
  'Vel\'Koz': 'Velkoz',
  'Vex': 'Vex',
  'Vi': 'Vi',
  'Viego': 'Viego',
  'Viktor': 'Viktor',
  'Vladimir': 'Vladimir',
  'Volibear': 'Volibear',
  'Warwick': 'Warwick',
  'Wukong': 'MonkeyKing',
  'Xayah': 'Xayah',
  'Xerath': 'Xerath',
  'Xin Zhao': 'XinZhao',
  'Yasuo': 'Yasuo',
  'Yone': 'Yone',
  'Yorick': 'Yorick',
  'Yuumi': 'Yuumi',
  'Zac': 'Zac',
  'Zed': 'Zed',
  'Zeri': 'Zeri',
  'Ziggs': 'Ziggs',
  'Zilean': 'Zilean',
  'Zoe': 'Zoe',
  'Zyra': 'Zyra'
};

/**
 * Get champion image URL from Data Dragon CDN
 * @param championName - The champion name (e.g., "Ahri", "Lee Sin")
 * @returns URL to champion image or null if not found
 */
export function getChampionImageUrl(championName: string | null | undefined): string | null {
  if (!championName) return null;
  
  const championId = CHAMPION_NAME_TO_ID[championName];
  if (!championId) return null;
  
  return `${DATA_DRAGON_BASE}/${PATCH_VERSION}/img/champion/${championId}.png`;
}

/**
 * Get item image URL from Data Dragon CDN
 * @param itemId - The item ID (e.g., 1001, 3006)
 * @returns URL to item image or null if invalid
 */
export function getItemImageUrl(itemId: number | string | null | undefined): string | null {
  if (!itemId || itemId === 0) return null;
  
  return `${DATA_DRAGON_BASE}/${PATCH_VERSION}/img/item/${itemId}.png`;
}

/**
 * Get champion square image URL (smaller, square version)
 * @param championName - The champion name
 * @returns URL to champion square image or null if not found
 */
export function getChampionSquareImageUrl(championName: string | null | undefined): string | null {
  if (!championName) return null;
  
  const championId = CHAMPION_NAME_TO_ID[championName];
  if (!championId) return null;
  
  return `${DATA_DRAGON_BASE}/${PATCH_VERSION}/img/champion/${championId}.png`;
}


/**
 * Get rune image URL
 * @param runeId - The rune ID
 * @returns URL to rune image or null if invalid
 */
export function getRuneImageUrl(runeId: number | string | null | undefined): string | null {
  if (!runeId) return null;
  
  return `${DATA_DRAGON_BASE}/${PATCH_VERSION}/img/${runeId}.png`;
}

/**
 * Get keystone rune image URL
 * @param keystoneId - The keystone rune ID
 * @returns URL to keystone rune image
 */
export function getKeystoneImageUrl(keystoneId: number | undefined): string {
  if (!keystoneId) return '';
  
  return `${DATA_DRAGON_BASE}/${PATCH_VERSION}/img/${keystoneId}.png`;
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
 * Get summoner spell image URL
 * @param spellName - The summoner spell display name
 * @returns URL to summoner spell image
 */
export function getSummonerSpellImageUrl(spellName: string | undefined): string {
  if (!spellName) return '';
  
  let spellKey = SUMMONER_SPELL_MAPPING[spellName];
  let baseUrl = DATA_DRAGON_BASE;
  let urlPath = `/${PATCH_VERSION}/img/spell/${spellKey}.png`;
  
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
        urlPath = `/${PATCH_VERSION}/img/spell/${spellKey}.png`;
      }
    } else {
      // Default fallback - try to construct from name
      spellKey = `Summoner${spellName.replace(/\s+/g, '')}`;
      urlPath = `/${PATCH_VERSION}/img/spell/${spellKey}.png`;
    }
  } else {
    // Check if this is a smite variant that should use Community Dragon
    if (spellKey === 'smite_red' || spellKey === 'smite_blue') {
      baseUrl = COMMUNITY_DRAGON_BASE;
      urlPath = `/${spellKey}.png`;
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
