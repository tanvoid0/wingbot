/** Queue ID → display label (common queues). */
export function getQueueLabel(queueId: number): string {
  const map: Record<number, string> = {
    420: 'Ranked Solo',
    440: 'Ranked Flex',
    450: 'ARAM',
    400: 'Normal Draft',
    430: 'Normal Blind',
    490: 'Quickplay',
    1700: 'Arena',
    900: 'URF',
    1020: 'One for All',
    1300: 'Nexus Blitz',
    1400: 'Ultimate Spellbook',
    1900: 'URF',
    0: 'Custom',
  };
  return map[queueId] ?? `Queue ${queueId}`;
}

export function formatKdaRatio(kills: number, deaths: number, assists: number): number {
  if (deaths <= 0) return kills + assists;
  return (kills + assists) / deaths;
}

export function formatKdaString(kills: number, deaths: number, assists: number): string {
  const ratio = formatKdaRatio(kills, deaths, assists);
  return `${kills}/${deaths}/${assists} (${ratio.toFixed(2)})`;
}

export function formatDamage(damage: number): string {
  if (damage >= 1000) return `${(damage / 1000).toFixed(1)}k`;
  return String(damage);
}

export function formatCsPerMin(cs: number, durationSecs: number): string {
  const mins = Math.max(durationSecs / 60, 1);
  return (cs / mins).toFixed(1);
}

export function getImpactLabel(score: number): { label: string; color: string } {
  if (score >= 85) return { label: 'Hard Carry', color: 'text-purple-500' };
  if (score >= 70) return { label: 'Strong', color: 'text-green-500' };
  if (score >= 55) return { label: 'Solid', color: 'text-blue-500' };
  if (score >= 40) return { label: 'Average', color: 'text-yellow-500' };
  return { label: 'Weak', color: 'text-red-500' };
}

export function getImpactBadgeClass(score: number): string {
  if (score >= 85) return 'bg-purple-500/15 text-purple-600 border-purple-500/30';
  if (score >= 70) return 'bg-green-500/15 text-green-600 border-green-500/30';
  if (score >= 55) return 'bg-blue-500/15 text-blue-600 border-blue-500/30';
  if (score >= 40) return 'bg-yellow-500/15 text-yellow-600 border-yellow-500/30';
  return 'bg-red-500/15 text-red-600 border-red-500/30';
}

export function getRoleLabel(role: string): string {
  const map: Record<string, string> = {
    TOP: 'Top',
    JUNGLE: 'Jungle',
    MIDDLE: 'Mid',
    BOTTOM: 'ADC',
    UTILITY: 'Support',
  };
  return map[role.toUpperCase()] ?? role;
}

export function formatGameDuration(durationSecs: number): string {
  const mins = Math.floor(durationSecs / 60);
  const secs = Math.floor(durationSecs % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

export function getTrendLabel(trend: string): { label: string; color: string } {
  switch (trend) {
    case 'excellent':
      return { label: 'On a hot streak', color: 'text-green-600' };
    case 'good':
      return { label: 'Playing well', color: 'text-blue-600' };
    case 'average':
      return { label: 'Mixed results', color: 'text-yellow-600' };
    case 'poor':
      return { label: 'Rough patch', color: 'text-red-600' };
    default:
      return { label: 'Not enough games', color: 'text-muted-foreground' };
  }
}

export function getRankedInfo(ranked: unknown): {
  tier: string;
  division: string;
  lp: number | null;
  summary: string;
} | null {
  if (!ranked || typeof ranked !== 'object') return null;
  const r = ranked as Record<string, unknown>;
  const queues = (r.queues ?? ranked) as unknown;
  if (!Array.isArray(queues)) return null;
  const solo = queues.find(
    (q: Record<string, unknown>) =>
      q.queueType === 'RANKED_SOLO_5x5' || q.queueType === 'RANKED_SOLO',
  ) as Record<string, unknown> | undefined;
  if (!solo) return null;
  const tier = String(solo.tier ?? solo.highestTier ?? '');
  if (!tier) return null;
  const division = String(solo.division ?? solo.highestDivision ?? '');
  const lp = typeof solo.leaguePoints === 'number' ? solo.leaguePoints : null;
  const summary = `${tier} ${division}${lp != null ? ` · ${lp} LP` : ''}`.trim();
  return { tier, division, lp, summary };
}

export function getRecentStreakDots(matches: unknown[]): boolean[] {
  return matches.slice(0, 5).map((m) => {
    const match = m as { stats?: { win?: boolean } };
    return match.stats?.win ?? false;
  });
}

export function getGradeBadgeClass(grade: string): string {
  switch (grade) {
    case 'S':
      return 'bg-purple-500/15 text-purple-600 border-purple-500/40';
    case 'A':
      return 'bg-green-500/15 text-green-600 border-green-500/40';
    case 'B':
      return 'bg-blue-500/15 text-blue-600 border-blue-500/40';
    case 'C':
      return 'bg-yellow-500/15 text-yellow-600 border-yellow-500/40';
    case 'D':
      return 'bg-red-500/15 text-red-600 border-red-500/40';
    default:
      return 'bg-muted text-muted-foreground';
  }
}

export interface ImpactTrendPoint {
  value: number;
  win: boolean;
  label: string;
}

export function getImpactTrendSeries(matches: unknown[]): ImpactTrendPoint[] {
  return [...matches]
    .slice(0, 10)
    .reverse()
    .map((m, i) => {
      const match = m as {
        impactScore?: number;
        stats?: { win?: boolean };
        championName?: string;
      };
      return {
        value: match.impactScore ?? 0,
        win: match.stats?.win ?? false,
        label: match.championName || `Game ${i + 1}`,
      };
    });
}

export function summarizeTrendSeries(series: ImpactTrendPoint[]) {
  if (series.length === 0) {
    return { min: 0, max: 0, avg: 0, latest: 0, peakIndex: -1 };
  }
  const values = series.map((s) => s.value);
  let peakIndex = 0;
  values.forEach((v, i) => {
    if (v > values[peakIndex]) peakIndex = i;
  });
  return {
    min: Math.round(Math.min(...values)),
    max: Math.round(Math.max(...values)),
    avg: Math.round(values.reduce((a, b) => a + b, 0) / values.length),
    latest: Math.round(values[values.length - 1] ?? 0),
    peakIndex,
  };
}
