/** Parsed champion select view model from LCU `/lol-champ-select/v1/session`. */

export interface ChampSelectSlot {
  championId: number;
  spell1Id?: number;
  spell2Id?: number;
  summonerName?: string;
  position?: string;
  /** True when showing hover intent before lock-in. */
  isHovered?: boolean;
}

export interface ParsedChampionSelectTeam {
  bans: ChampSelectSlot[];
  picks: ChampSelectSlot[];
}

export interface ParsedChampionSelect {
  blueTeam: ParsedChampionSelectTeam;
  redTeam: ParsedChampionSelectTeam;
  phase: string;
  timeRemainingSec: number;
}

function mapPlayer(player: Record<string, unknown>): ChampSelectSlot {
  const championId =
    (player.championId as number) ||
    (player.championPickIntent as number) ||
    0;
  const gameName = player.gameName as string | undefined;
  const tagLine = player.tagLine as string | undefined;

  return {
    championId,
    spell1Id: player.spell1Id as number | undefined,
    spell2Id: player.spell2Id as number | undefined,
    summonerName: gameName
      ? tagLine
        ? `${gameName}#${tagLine}`
        : gameName
      : undefined,
    position: player.assignedPosition as string | undefined,
    isHovered: !(player.championId as number) && !!(player.championPickIntent as number),
  };
}

function extractBansFromActions(
  actions: unknown,
): { ally: ChampSelectSlot[]; enemy: ChampSelectSlot[] } {
  const ally: ChampSelectSlot[] = [];
  const enemy: ChampSelectSlot[] = [];

  if (!Array.isArray(actions)) {
    return { ally, enemy };
  }

  for (const round of actions) {
    if (!Array.isArray(round)) continue;
    for (const action of round) {
      if (
        action?.type !== 'ban' ||
        !action.completed ||
        !action.championId ||
        action.championId <= 0
      ) {
        continue;
      }
      const slot: ChampSelectSlot = { championId: action.championId as number };
      if (action.isAllyAction) {
        ally.push(slot);
      } else {
        enemy.push(slot);
      }
    }
  }

  return { ally, enemy };
}

function timerSeconds(timer: Record<string, unknown> | undefined): number {
  if (!timer) return 0;
  if (timer.isInfinite === true) return 0;

  // LCU exposes a snapshot: remaining ms at internalNowInEpochMs, not a live counter.
  if (
    typeof timer.adjustedTimeLeftInPhase === 'number' &&
    typeof timer.internalNowInEpochMs === 'number'
  ) {
    const elapsedMs = Date.now() - timer.internalNowInEpochMs;
    const remainingMs = timer.adjustedTimeLeftInPhase - elapsedMs;
    return Math.max(0, Math.floor(remainingMs / 1000));
  }

  if (typeof timer.adjustedTimeLeftInPhaseInSec === 'number') {
    return timer.adjustedTimeLeftInPhaseInSec;
  }
  if (typeof timer.timeLeftInPhaseInSec === 'number') {
    return timer.timeLeftInPhaseInSec;
  }
  if (typeof timer.adjustedTimeLeftInPhase === 'number') {
    return Math.max(0, Math.ceil(timer.adjustedTimeLeftInPhase / 1000));
  }
  return 0;
}

/** Convert raw LCU champion select session JSON into UI-friendly team data. */
export function parseChampionSelectSession(
  raw: Record<string, unknown> | null | undefined,
): ParsedChampionSelect | null {
  if (!raw) return null;

  const hasSession =
    Array.isArray(raw.myTeam) ||
    (raw.timer &&
      typeof raw.timer === 'object' &&
      (raw.timer as Record<string, unknown>).phase);
  if (!hasSession) return null;

  const { ally: allyBans, enemy: enemyBans } = extractBansFromActions(raw.actions);

  const myTeam = Array.isArray(raw.myTeam)
    ? [...raw.myTeam]
        .sort(
          (a, b) =>
            ((a as Record<string, number>).cellId ?? 0) -
            ((b as Record<string, number>).cellId ?? 0),
        )
        .map((p) => mapPlayer(p as Record<string, unknown>))
    : [];

  const theirTeam = Array.isArray(raw.theirTeam)
    ? [...raw.theirTeam]
        .sort(
          (a, b) =>
            ((a as Record<string, number>).cellId ?? 0) -
            ((b as Record<string, number>).cellId ?? 0),
        )
        .map((p) => mapPlayer(p as Record<string, unknown>))
    : [];

  const timer = raw.timer as Record<string, unknown> | undefined;

  return {
    blueTeam: { bans: allyBans, picks: myTeam },
    redTeam: { bans: enemyBans, picks: theirTeam },
    phase: (timer?.phase as string) ?? 'UNKNOWN',
    timeRemainingSec: timerSeconds(timer),
  };
}

export function champSelectTimerSec(timer: unknown): number {
  if (!timer || typeof timer !== 'object') return 0;
  return timerSeconds(timer as Record<string, unknown>);
}

/** True when only the timer field changed between two session snapshots. */
export function isTimerOnlyChampSelectChange(
  prev: Record<string, unknown>,
  next: Record<string, unknown>,
): boolean {
  const keys = new Set([...Object.keys(prev), ...Object.keys(next)]);
  keys.delete('timer');
  for (const key of keys) {
    if (JSON.stringify(prev[key]) !== JSON.stringify(next[key])) {
      return false;
    }
  }
  return JSON.stringify(prev.timer) !== JSON.stringify(next.timer);
}
