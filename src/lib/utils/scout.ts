/** Format helpers for Riot API scout profiles. */

import { playerToScoutSlot, scoutEnemyKey } from '$lib/utils/championSelectParser';

export interface ScoutRank {
  tier: string;
  division: string;
  lp: number;
  wins: number;
  losses: number;
  win_rate: number;
  summary: string;
}

export interface ScoutMastery {
  champion_id: number;
  level: number;
  points: number;
}

export interface ScoutChampionForm {
  champion_id: number;
  games: number;
  wins: number;
  losses: number;
  win_rate: number;
  avg_kda: number;
  summary: string;
}

export interface ScoutPlayerProfile {
  puuid?: string | null;
  riot_id?: string | null;
  position?: string | null;
  champion_id?: number | null;
  solo_rank?: ScoutRank | null;
  flex_rank?: ScoutRank | null;
  primary_rank?: ScoutRank | null;
  mastery_score?: number;
  top_masteries?: ScoutMastery[];
  champion_mastery?: ScoutMastery | null;
  champion_form?: ScoutChampionForm | null;
  is_one_trick?: boolean;
  champion_in_top_masteries?: boolean;
  error?: string | null;
}

export function formatMasteryPoints(points: number): string {
  if (points >= 1_000_000) return `${(points / 1_000_000).toFixed(1)}M`;
  if (points >= 1_000) return `${Math.round(points / 1_000)}k`;
  return String(points);
}

export function getRoleLabel(position: string | null | undefined): string {
  if (!position) return '';
  const map: Record<string, string> = {
    TOP: 'Top',
    JUNGLE: 'Jungle',
    MIDDLE: 'Mid',
    BOTTOM: 'ADC',
    UTILITY: 'Support',
  };
  return map[position.toUpperCase()] ?? position;
}

export function getRankTierColor(tier: string): string {
  const t = tier.toUpperCase();
  if (t === 'CHALLENGER' || t === 'GRANDMASTER') return 'text-amber-500';
  if (t === 'MASTER') return 'text-purple-500';
  if (t === 'DIAMOND') return 'text-cyan-500';
  if (t === 'EMERALD') return 'text-emerald-500';
  if (t === 'PLATINUM') return 'text-teal-500';
  if (t === 'GOLD') return 'text-yellow-500';
  if (t === 'SILVER') return 'text-slate-400';
  if (t === 'BRONZE') return 'text-orange-700';
  if (t === 'IRON') return 'text-stone-500';
  return 'text-muted-foreground';
}

export function getThreatLabel(profile: ScoutPlayerProfile): string | null {
  if (profile.is_one_trick) return 'One-trick';
  if (profile.champion_in_top_masteries) return 'Main champ';
  const rank = profile.primary_rank ?? profile.solo_rank;
  if (!rank) return null;
  const t = rank.tier.toUpperCase();
  if (['CHALLENGER', 'GRANDMASTER', 'MASTER'].includes(t)) return 'High elo';
  if (t === 'DIAMOND' || t === 'EMERALD') return 'Strong';
  return null;
}

export function profileForPlayer(
  player: Record<string, unknown>,
  profiles: Record<string, ScoutPlayerProfile>,
): ScoutPlayerProfile | undefined {
  return profiles[scoutEnemyKey(playerToScoutSlot(player))];
}
