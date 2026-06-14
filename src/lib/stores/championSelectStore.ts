import { writable, derived } from 'svelte/store';

// Champion Select State
export const championSelectData = writable<any>(null);
export const championSelectStore = writable<any>(null);
export const championSelectPhase = writable<string>('None');
export const championSelectTimer = writable<number>(0);
export const myTeam = writable<any[]>([]);
export const enemyTeam = writable<any[]>([]);
export const bans = writable<any[]>([]);
export const picks = writable<any[]>([]);
export const isChampionSelectActive = writable<boolean>(false);

// Champion Select Actions
export const championSelectActions = {
  updateChampionSelectData: (data: any) => {
    if (data && data.phase) {
      championSelectData.set(data);
      championSelectStore.set(data);
      championSelectPhase.set(data.phase);
      isChampionSelectActive.set(true);
      
      // Update timer if available
      if (data.timer) {
        championSelectTimer.set(data.timer);
      }
      
      // Update teams if available
      if (data.myTeam) {
        myTeam.set(data.myTeam);
      }
      if (data.theirTeam) {
        enemyTeam.set(data.theirTeam);
      }
      
      // Update bans and picks
      if (data.bans) {
        bans.set(data.bans);
      }
      if (data.picks) {
        picks.set(data.picks);
      }
    } else {
      // Clear champion select data
      championSelectData.set(null);
      championSelectStore.set(null);
      championSelectPhase.set('None');
      isChampionSelectActive.set(false);
      championSelectTimer.set(0);
      myTeam.set([]);
      enemyTeam.set([]);
      bans.set([]);
      picks.set([]);
    }
  },

  getPhaseDisplayName: (phase: string) => {
    switch (phase) {
      case 'BAN_PICK':
        return 'Ban & Pick Phase';
      case 'FINALIZATION':
        return 'Finalization';
      case 'GAME_STARTING':
        return 'Game Starting';
      case 'PLANNING':
        return 'Planning';
      case 'READY_CHECK':
        return 'Ready Check';
      default:
        return phase;
    }
  },

  getPhaseColor: (phase: string) => {
    switch (phase) {
      case 'BAN_PICK':
        return 'text-blue-600';
      case 'FINALIZATION':
        return 'text-green-600';
      case 'GAME_STARTING':
        return 'text-purple-600';
      case 'PLANNING':
        return 'text-yellow-600';
      case 'READY_CHECK':
        return 'text-orange-600';
      default:
        return 'text-gray-600';
    }
  },

  formatTimer: (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
};

// Derived stores
export const championSelectSummary = derived(
  [championSelectPhase, championSelectTimer, isChampionSelectActive],
  ([phase, timer, isActive]) => ({
    phase,
    timer,
    isActive,
    displayName: championSelectActions.getPhaseDisplayName(phase),
    color: championSelectActions.getPhaseColor(phase),
    formattedTimer: championSelectActions.formatTimer(timer)
  })
);
