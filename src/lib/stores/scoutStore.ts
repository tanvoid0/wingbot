import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import {
  extractScoutableEnemies,
  scoutEnemyKey,
  type ChampSelectSlot,
} from '$lib/utils/championSelectParser';
import type { ScoutPlayerProfile } from '$lib/utils/scout';
import { isOllamaFeatureEnabled } from './ollamaStatusStore';

export const scoutProfiles = writable<Record<string, ScoutPlayerProfile>>({});
export const scoutLoading = writable(false);
export const scoutError = writable<string | null>(null);
export const scoutApiRequired = writable(false);
export const scoutAiInsights = writable<Array<{ type: string; title: string; message: string; source?: string }>>([]);
export const scoutAiInsightsLoading = writable(false);
export const scoutAiInsightsError = writable<string | null>(null);

let scoutAiCacheKey = '';

function toScoutInput(player: ChampSelectSlot, includeForm = false) {
  return {
    puuid: player.puuid ?? null,
    gameName: player.gameName ?? null,
    tagLine: player.tagLine ?? null,
    championId: player.isLocked && player.championId > 0 ? player.championId : null,
    championName: player.championName ?? null,
    position: player.position ?? null,
    includeChampionForm: includeForm && (player.isLocked || !!player.championName),
  };
}

async function scoutPlayerBatch(
  players: ChampSelectSlot[],
  includeForm: boolean,
): Promise<ScoutPlayerProfile[]> {
  if (players.length === 0) return [];

  const current = get(scoutProfiles);
  const missing = players.filter((e) => !current[scoutEnemyKey(e)]);
  if (missing.length === 0) {
    return players.map((e) => current[scoutEnemyKey(e)]).filter(Boolean);
  }

  scoutLoading.set(true);
  scoutError.set(null);
  scoutApiRequired.set(false);

  try {
    const result = await invoke<{ players: ScoutPlayerProfile[] }>('scout_players', {
      players: missing.map((p) => toScoutInput(p, includeForm)),
    });

    const updated = { ...get(scoutProfiles) };
    result.players.forEach((profile, i) => {
      updated[scoutEnemyKey(missing[i])] = profile;
    });
    scoutProfiles.set(updated);

    void scoutActions.fetchAiInsights(players, updated);

    return players
      .map((e) => updated[scoutEnemyKey(e)] ?? current[scoutEnemyKey(e)])
      .filter(Boolean);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    if (message.toLowerCase().includes('api key')) {
      scoutApiRequired.set(true);
    }
    scoutError.set(message);
    return [];
  } finally {
    scoutLoading.set(false);
  }
}

export const scoutActions = {
  async scoutPlayers(
    players: ChampSelectSlot[],
    options?: { includeChampionForm?: boolean },
  ): Promise<ScoutPlayerProfile[]> {
    return scoutPlayerBatch(players, options?.includeChampionForm ?? false);
  },

  async scoutEnemiesFromSession(
    session: Record<string, unknown> | null | undefined,
  ): Promise<ScoutPlayerProfile[]> {
    const enemies = extractScoutableEnemies(session);
    return scoutPlayerBatch(enemies, true);
  },

  async scoutEnemiesFromLiveGame(
    gameData: Record<string, unknown> | null | undefined,
  ): Promise<ScoutPlayerProfile[]> {
    const { extractScoutableFromLiveGame } = await import('$lib/utils/championSelectParser');
    const enemies = extractScoutableFromLiveGame(gameData);
    return scoutPlayerBatch(enemies, true);
  },

  async scoutLobbyMembers(
    lobby: Record<string, unknown> | null | undefined,
    currentPuuid?: string | null,
  ): Promise<ScoutPlayerProfile[]> {
    const { extractScoutableLobbyMembers } = await import('$lib/utils/championSelectParser');
    const members = extractScoutableLobbyMembers(lobby, currentPuuid);
    return scoutPlayerBatch(members, false);
  },

  buildAiCacheKey(enemies: ChampSelectSlot[], profiles: Record<string, ScoutPlayerProfile>) {
    return enemies
      .map((e) => {
        const profile = profiles[scoutEnemyKey(e)];
        const rank = profile?.primary_rank ?? profile?.solo_rank;
        return `${scoutEnemyKey(e)}:${e.championId}:${rank?.tier ?? 'none'}:${profile?.is_one_trick ?? false}`;
      })
      .sort()
      .join('|');
  },

  async fetchAiInsights(
    enemies: ChampSelectSlot[],
    profiles?: Record<string, ScoutPlayerProfile>,
    options?: { force?: boolean },
  ) {
    const profileMap = profiles ?? get(scoutProfiles);
    const scouted = enemies.filter((e) => e.championId > 0);
    if (scouted.length === 0 || !isOllamaFeatureEnabled('scout')) {
      scoutAiInsights.set([]);
      scoutAiInsightsError.set(null);
      return [];
    }

    const cacheKey = scoutActions.buildAiCacheKey(scouted, profileMap);
    const force = options?.force ?? false;
    if (!force && cacheKey === scoutAiCacheKey && get(scoutAiInsights).length > 0) {
      return get(scoutAiInsights);
    }

    const payload = {
      enemies: scouted.map((enemy) => {
        const profile = profileMap[scoutEnemyKey(enemy)];
        return {
          ...profile,
          position: enemy.position ?? profile?.position ?? null,
          champion_id: enemy.championId,
          champion_name: enemy.championName ?? enemy.summonerName ?? profile?.riot_id ?? null,
          champion_form: profile?.champion_form ?? null,
        };
      }),
    };

    scoutAiInsightsLoading.set(true);
    scoutAiInsightsError.set(null);

    try {
      const insights = await invoke<Array<{ type: string; title: string; message: string; source?: string }>>(
        'get_scout_ai_insights',
        { payload },
      );
      scoutAiCacheKey = cacheKey;
      scoutAiInsights.set(Array.isArray(insights) ? insights : []);
      return get(scoutAiInsights);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      scoutAiInsightsError.set(message);
      if (force) {
        scoutAiInsights.set([]);
      }
      return get(scoutAiInsights);
    } finally {
      scoutAiInsightsLoading.set(false);
    }
  },

  clear() {
    scoutProfiles.set({});
    scoutError.set(null);
    scoutApiRequired.set(false);
    scoutAiInsights.set([]);
    scoutAiInsightsError.set(null);
    scoutAiCacheKey = '';
  },
};
