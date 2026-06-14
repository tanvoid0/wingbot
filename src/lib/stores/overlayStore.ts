import { writable } from 'svelte/store';

export interface OverlayConfig {
  enabled: boolean;
  x: number;
  y: number;
  opacity: number;
  scale: number;
}

export interface JungleCamp {
  id: string;
  campName: string;
  type: 'buff' | 'camp';
  respawnTime: number;
  isAlive: boolean;
  team: 'blue' | 'red' | 'neutral' | 'ORDER' | 'CHAOS';
}

export interface Objective {
  id: string;
  objective: string;
  type: 'dragon' | 'baron' | 'herald' | 'grubs';
  spawnTime: number;
  isAlive: boolean;
  timeUntilSpawn: number;
  nextSpawn: number;
}

export interface CSTrackerData {
  current: number;
  target: number;
  perMinute: number;
  goldFromCS: number;
  benchmark: 'poor' | 'average' | 'good' | 'excellent';
}

export interface CounterItem {
  itemId: number;
  name: string;
  cost: number;
  reason: string;
  priority: 'high' | 'medium' | 'low';
  enemyThreat: string;
}

// Overlay configurations
export const overlayConfigs = writable<Record<string, OverlayConfig>>({
  csTracker: { enabled: true, x: 100, y: 100, opacity: 0.9, scale: 1.0 },
  jungleTimers: { enabled: true, x: 200, y: 100, opacity: 0.9, scale: 1.0 },
  objectiveTimers: { enabled: true, x: 300, y: 100, opacity: 0.9, scale: 1.0 },
  counterItems: { enabled: true, x: 400, y: 100, opacity: 0.9, scale: 1.0 },
  enemySpellTracker: { enabled: false, x: 500, y: 100, opacity: 0.9, scale: 1.0 },
  mapReminder: { enabled: false, x: 600, y: 100, opacity: 0.9, scale: 1.0 },
  trinketReminder: { enabled: false, x: 700, y: 100, opacity: 0.9, scale: 1.0 }
});

// Overlay data stores
export const jungleTimers = writable<JungleCamp[]>([]);
export const objectiveTimers = writable<Objective[]>([]);
export const csTrackerData = writable<CSTrackerData | null>(null);
export const counterItems = writable<CounterItem[]>([]);
export const enemySpells = writable<Record<string, { cooldown: number; lastUsed: number }>>({});

// Overlay actions
export const overlayActions = {
  updateConfig: (overlayName: string, config: Partial<OverlayConfig>) => {
    overlayConfigs.update(configs => ({
      ...configs,
      [overlayName]: { ...configs[overlayName], ...config }
    }));
  },

  toggleOverlay: (overlayName: string) => {
    overlayConfigs.update(configs => ({
      ...configs,
      [overlayName]: { ...configs[overlayName], enabled: !configs[overlayName].enabled }
    }));
  },

  resetPosition: (overlayName: string) => {
    overlayConfigs.update(configs => ({
      ...configs,
      [overlayName]: { ...configs[overlayName], x: 100, y: 100 }
    }));
  }
};
