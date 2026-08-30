/** Parsed champion select view model from LCU `/lol-champ-select/v1/session`. */

export interface ChampSelectSlot {
  championId: number;
  championName?: string;
  cellId?: number;
  spell1Id?: number;
  spell2Id?: number;
  summonerName?: string;
  gameName?: string;
  tagLine?: string;
  puuid?: string;
  position?: string;
  /** True when showing hover intent before lock-in. */
  isHovered?: boolean;
  /** True when the player has locked a champion (not just hovering). */
  isLocked?: boolean;
  /** True when this player currently has the active pick/ban timer. */
  isActive?: boolean;
}

/** Standard draft role order (top → support). */
export const ROLE_SORT_ORDER = [
  'TOP',
  'JUNGLE',
  'MIDDLE',
  'BOTTOM',
  'UTILITY',
] as const;

const ROLE_SORT_INDEX: Record<string, number> = {
  TOP: 0,
  JUNGLE: 1,
  MIDDLE: 2,
  BOTTOM: 3,
  UTILITY: 4,
};

export interface ParsedChampionSelectTeam {
  bans: ChampSelectSlot[];
  picks: ChampSelectSlot[];
}

export interface ParsedChampionSelect {
  blueTeam: ParsedChampionSelectTeam;
  redTeam: ParsedChampionSelectTeam;
  phase: string;
  timeRemainingSec: number;
  /** LCU cell id of the player whose pick/ban timer is active. */
  activeCellId: number | null;
}

function readChampionName(player: Record<string, unknown>): string | undefined {
  const raw =
    player.championName ??
    player.championname ??
    player.internalName;
  if (typeof raw === 'string' && raw.trim()) return raw.trim();
  return undefined;
}

function mapPlayer(player: Record<string, unknown>): ChampSelectSlot {
  const lockedId = player.championId as number | undefined;
  const hoverId = player.championPickIntent as number | undefined;
  const championId = lockedId || hoverId || 0;
  const gameName = player.gameName as string | undefined;
  const tagLine = player.tagLine as string | undefined;

  return {
    championId,
    championName: readChampionName(player),
    cellId: player.cellId as number | undefined,
    spell1Id: player.spell1Id as number | undefined,
    spell2Id: player.spell2Id as number | undefined,
    gameName,
    tagLine,
    puuid: player.puuid as string | undefined,
    summonerName: gameName
      ? tagLine
        ? `${gameName}#${tagLine}`
        : gameName
      : undefined,
    position: player.assignedPosition as string | undefined,
    isHovered: !lockedId && !!hoverId,
    isLocked: !!lockedId && lockedId > 0,
  };
}

/** Resolve display name: LCU name first, then Data Dragon id lookup. */
export function resolveChampionDisplayName(
  slot: Pick<ChampSelectSlot, 'championId' | 'championName'>,
  namesById?: Record<number, string>,
): string {
  if (slot.championName) return slot.championName;
  if (slot.championId && namesById?.[slot.championId]) {
    return namesById[slot.championId];
  }
  return '';
}


/** Order picks top → jungle → mid → bot → support; fill gaps with unassigned players. */
export function sortPicksByRole(picks: ChampSelectSlot[]): ChampSelectSlot[] {
  const byRole = new Map<string, ChampSelectSlot>();
  const unassigned: ChampSelectSlot[] = [];

  for (const pick of picks) {
    const pos = (pick.position ?? '').toUpperCase();
    if (pos && pos in ROLE_SORT_INDEX && !byRole.has(pos)) {
      byRole.set(pos, pick);
    } else {
      unassigned.push(pick);
    }
  }

  unassigned.sort(
    (a, b) => (a.cellId ?? 999) - (b.cellId ?? 999),
  );

  const ordered: ChampSelectSlot[] = [];
  for (const role of ROLE_SORT_ORDER) {
    if (byRole.has(role)) {
      ordered.push(byRole.get(role)!);
    } else if (unassigned.length > 0) {
      ordered.push(unassigned.shift()!);
    } else {
      ordered.push({ championId: 0, position: role });
    }
  }

  return ordered.slice(0, 5);
}

/**
 * The action currently awaiting input. Its `type` is the only thing that
 * distinguishes banning from picking — the LCU reports `timer.phase` as
 * `BAN_PICK` for both, and never changes it when bans finish.
 */
function extractActiveAction(
  actions: unknown,
): { cellId: number | null; type: string | null } {
  if (!Array.isArray(actions)) return { cellId: null, type: null };

  for (const round of actions) {
    if (!Array.isArray(round)) continue;
    for (const action of round) {
      if (action?.isInProgress === true) {
        return {
          cellId:
            typeof action.actorCellId === 'number' ? action.actorCellId : null,
          type: typeof action.type === 'string' ? action.type : null,
        };
      }
    }
  }
  return { cellId: null, type: null };
}

/** `type` of the action awaiting input (`ban` / `pick`), if any. */
export function activeChampSelectActionType(actions: unknown): string | null {
  return extractActiveAction(actions).type;
}

/** Refines the LCU's coarse `BAN_PICK` into the phase actually in progress. */
export function refineChampSelectPhase(
  timerPhase: string,
  activeActionType: string | null,
): string {
  if (timerPhase !== 'BAN_PICK') return timerPhase;
  if (activeActionType === 'ban') return 'BAN';
  if (activeActionType === 'pick') return 'PICK';
  return timerPhase;
}

function markActivePicks(
  picks: ChampSelectSlot[],
  activeCellId: number | null,
): ChampSelectSlot[] {
  if (activeCellId == null) return picks;
  return picks.map((pick) => ({
    ...pick,
    isActive: pick.cellId === activeCellId,
  }));
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
      const slot: ChampSelectSlot = {
        championId: action.championId as number,
        championName:
          typeof action.championName === 'string'
            ? action.championName
            : undefined,
      };
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

  const activeAction = extractActiveAction(raw.actions);
  const activeCellId = activeAction.cellId;

  const myTeam = markActivePicks(
    sortPicksByRole(
      Array.isArray(raw.myTeam)
        ? raw.myTeam.map((p) => mapPlayer(p as Record<string, unknown>))
        : [],
    ),
    activeCellId,
  );

  const theirTeam = markActivePicks(
    sortPicksByRole(
      Array.isArray(raw.theirTeam)
        ? raw.theirTeam.map((p) => mapPlayer(p as Record<string, unknown>))
        : raw.enemyTeam && Array.isArray(raw.enemyTeam)
          ? raw.enemyTeam.map((p) => mapPlayer(p as Record<string, unknown>))
          : [],
    ),
    activeCellId,
  );

  const timer = raw.timer as Record<string, unknown> | undefined;

  return {
    blueTeam: { bans: allyBans, picks: myTeam },
    redTeam: { bans: enemyBans, picks: theirTeam },
    phase: refineChampSelectPhase(
      (timer?.phase as string) ?? 'UNKNOWN',
      activeAction.type,
    ),
    timeRemainingSec: timerSeconds(timer),
    activeCellId,
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

/** Enemy players from LCU session — only those with identity and a locked champion. */
export function extractScoutableEnemies(
  raw: Record<string, unknown> | null | undefined,
): ChampSelectSlot[] {
  if (!raw) return [];

  const theirTeam = (raw.theirTeam ?? raw.enemyTeam) as unknown;
  if (!Array.isArray(theirTeam)) return [];

  return theirTeam
    .map((p) => mapPlayer(p as Record<string, unknown>))
    .filter(
      (p) =>
        p.isLocked &&
        p.championId > 0 &&
        (!!p.puuid || (!!p.gameName && !!p.tagLine)),
    );
}

/** Enemy players from live client — loading screen / early game. */
export function extractScoutableFromLiveGame(
  gameData: Record<string, unknown> | null | undefined,
): ChampSelectSlot[] {
  if (!gameData) return [];

  const allPlayers = gameData.allPlayers as Array<Record<string, unknown>> | undefined;
  const activePlayer = gameData.activePlayer as Record<string, unknown> | undefined;
  if (!Array.isArray(allPlayers) || allPlayers.length === 0) return [];

  const myTeam =
    (activePlayer?.team as string | undefined) ??
    (allPlayers.find(
      (p) =>
        p.summonerName === activePlayer?.summonerName ||
        p.riotIdGameName === activePlayer?.riotIdGameName,
    )?.team as string | undefined);

  return allPlayers
    .filter((p) => !p.isBot && p.team !== myTeam)
    .filter((p) => p.riotIdGameName && p.riotIdTagLine)
    .map((p) => {
      const gameName = p.riotIdGameName as string;
      const tagLine = p.riotIdTagLine as string;
      const championName = (p.championName as string) || undefined;
      return {
        championId: 0,
        championName,
        gameName,
        tagLine,
        summonerName: `${gameName}#${tagLine}`,
        position: p.position as string | undefined,
        isLocked: true,
        isHovered: false,
      };
    });
}

/** Premade lobby members eligible for rank scouting (excludes self when puuid known). */
export function extractScoutableLobbyMembers(
  lobby: Record<string, unknown> | null | undefined,
  currentPuuid?: string | null,
): ChampSelectSlot[] {
  if (!lobby) return [];

  const members = (lobby.members ?? lobby.parsed_members) as
    | Array<Record<string, unknown>>
    | undefined;
  if (!Array.isArray(members)) return [];

  return members
    .filter((m) => {
      const puuid = m.puuid as string | undefined;
      if (currentPuuid && puuid === currentPuuid) return false;
      return (
        !!puuid ||
        (!!m.game_name && !!m.tag_line) ||
        (!!m.gameName && !!m.tagLine)
      );
    })
    .map((m) => {
      const gameName = (m.game_name ?? m.gameName) as string | undefined;
      const tagLine = (m.tag_line ?? m.tagLine) as string | undefined;
      return {
        championId: 0,
        puuid: m.puuid as string | undefined,
        gameName,
        tagLine,
        summonerName:
          gameName && tagLine ? `${gameName}#${tagLine}` : (m.riot_id as string | undefined),
        position: (m.position ?? m.firstPositionPreference) as string | undefined,
        isLocked: false,
        isHovered: false,
      };
    });
}

/** Map any player-like object to a scout input slot. */
export function playerToScoutSlot(player: Record<string, unknown>): ChampSelectSlot {
  const gameName = (player.riotIdGameName ?? player.gameName) as string | undefined;
  const tagLine = (player.riotIdTagLine ?? player.tagLine) as string | undefined;
  const riotId = player.riotId as string | undefined;
  let parsedGame = gameName;
  let parsedTag = tagLine;
  if (!parsedGame && riotId?.includes('#')) {
    const [g, t] = riotId.split('#');
    parsedGame = g;
    parsedTag = t;
  }
  return {
    championId: (player.championId as number) || 0,
    championName: (player.championName as string) || undefined,
    puuid: player.puuid as string | undefined,
    gameName: parsedGame,
    tagLine: parsedTag,
    summonerName:
      parsedGame && parsedTag
        ? `${parsedGame}#${parsedTag}`
        : (riotId ?? (player.summonerName as string | undefined)),
    position: (player.position as string) || undefined,
    isLocked: true,
    isHovered: false,
  };
}

export function extractScoutableFromAllPlayers(
  players: Array<Record<string, unknown>>,
): ChampSelectSlot[] {
  return players
    .filter((p) => !p.isBot)
    .filter(
      (p) =>
        !!p.puuid ||
        (!!p.riotIdGameName && !!p.riotIdTagLine) ||
        (typeof p.riotId === 'string' && p.riotId.includes('#')),
    )
    .map((p) => playerToScoutSlot(p));
}

/** Stable cache key for a scoutable enemy. */
export function scoutEnemyKey(player: ChampSelectSlot): string {
  if (player.puuid) return `puuid:${player.puuid}`;
  if (player.gameName && player.tagLine) {
    return `riot:${player.gameName}#${player.tagLine}`;
  }
  return `champ:${player.championId}`;
}
