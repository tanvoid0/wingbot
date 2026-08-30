// Game data types that match the backend Rust structures

export interface ChampionStats {
  abilityHaste: number;
  abilityPower: number;
  armor: number;
  armorPenetrationFlat: number;
  armorPenetrationPercent: number;
  attackDamage: number;
  attackRange: number;
  attackSpeed: number;
  bonusArmorPenetrationPercent: number;
  bonusMagicPenetrationPercent: number;
  critChance: number;
  critDamage: number;
  currentHealth: number;
  healShieldPower: number;
  healthRegenRate: number;
  lifeSteal: number;
  magicLethality: number;
  magicPenetrationFlat: number;
  magicPenetrationPercent: number;
  magicResist: number;
  maxHealth: number;
  moveSpeed: number;
  omnivamp: number;
  physicalLethality: number;
  physicalVamp: number;
  resourceMax: number;
  resourceRegenRate: number;
  resourceType: string;
  resourceValue: number;
  spellVamp: number;
  tenacity: number;
}

export interface Ability {
  abilityLevel: number;
  displayName: string;
  id: string;
  rawDescription: string;
  rawDisplayName: string;
}

export interface Rune {
  displayName: string;
  id: number;
  rawDescription: string;
  rawDisplayName: string;
}

export interface RuneTree {
  displayName: string;
  id: number;
  rawDescription: string;
  rawDisplayName: string;
}

export interface StatRune {
  id: number;
  rawDescription: string;
}

export interface FullRunes {
  generalRunes: Rune[];
  keystone: Rune;
  primaryRuneTree: RuneTree;
  secondaryRuneTree: RuneTree;
  statRunes: StatRune[];
}

// Live Client API Active Player structure (matches actual API response)
export interface LiveClientActivePlayer {
  abilities: Record<string, Ability>;
  championStats: ChampionStats;
  championName?: string;
  currentGold: number;
  fullRunes: FullRunes;
  level: number;
  riotId: string;
  riotIdGameName: string;
  riotIdTagLine: string;
  summonerName: string;
  teamRelativeColors: boolean;
  // Live Client API specific fields
  scores?: PlayerScores;
  items?: Item[];
}

// Keep the original ActivePlayer for backward compatibility
export interface ActivePlayer {
  abilities: Record<string, Ability>;
  championStats: ChampionStats;
  championName?: string;
  currentGold: number;
  fullRunes: FullRunes;
  level: number;
  riotId: string;
  riotIdGameName: string;
  riotIdTagLine: string;
  summonerName: string;
  teamRelativeColors: boolean;
}

export interface Item {
  canUse: boolean;
  consumable: boolean;
  count: number;
  displayName: string;
  itemID: number;
  price: number;
  rawDescription: string;
  rawDisplayName: string;
  slot: number;
}

export interface PlayerRunes {
  keystone: Rune;
  primaryRuneTree: RuneTree;
  secondaryRuneTree: RuneTree;
}

export interface PlayerScores {
  assists: number;
  creepScore: number;
  deaths: number;
  kills: number;
  wardScore: number;
}

export interface SummonerSpell {
  displayName: string;
  rawDescription: string;
  rawDisplayName: string;
}

export interface PlayerSummonerSpells {
  summonerSpellOne: SummonerSpell;
  summonerSpellTwo: SummonerSpell;
}

export interface Player {
  championName: string;
  isBot: boolean;
  isDead: boolean;
  items: Item[];
  level: number;
  position: string;
  rawChampionName: string;
  rawSkinName: string;
  respawnTimer: number;
  riotId: string;
  riotIdGameName: string;
  riotIdTagLine: string;
  runes: PlayerRunes;
  scores: PlayerScores;
  skinID: number;
  skinName: string;
  summonerName: string;
  summonerSpells: PlayerSummonerSpells;
  team: 'ORDER' | 'CHAOS';
}

export interface GameEvent {
  EventID: number;
  EventName: string;
  EventTime: number;
}

export interface GameEvents {
  Events: GameEvent[];
}

export interface GameData {
  gameMode: string;
  gameTime: number;
  mapName: string;
  mapNumber: number;
  mapTerrain: string;
}

// Live Client API Game Data structure (matches actual API response)
export interface LiveClientGameData {
  activePlayer: LiveClientActivePlayer;
  allPlayers: Player[];
  events: GameEvents;
  gameData: GameData;
  recent_events?: any[];
}

// Keep the original LiveGameData for backward compatibility
export interface LiveGameData {
  activePlayer: ActivePlayer;
  allPlayers: Player[];
  events: GameEvents;
  gameData: GameData;
  recent_events?: any[];
}

export interface ComprehensiveGameState {
  lcu_connected: boolean;
  game_phase: string;
  live_client_available: boolean;
  is_spectator_mode: boolean;
  current_summoner: Record<string, unknown> | null;
  champion_select: Record<string, unknown> | null;
  live_game: LiveClientGameData | Record<string, unknown> | null;
  live_stats: GameStats | null;
  player_scores: PlayerScoresData | null;
  spectator_game?: LiveClientGameData | Record<string, unknown> | null;
  timestamp: number;
}

export interface GameStats {
  // Add specific game stats structure when needed
  [key: string]: unknown;
}

export interface PlayerScoresData {
  // Add specific player scores structure when needed
  [key: string]: unknown;
}
