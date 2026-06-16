<script lang="ts">
  import { Trophy, Target, Zap, Swords, TrendingUp, Eye, Award } from '@lucide/svelte';
  import MatchHistoryStatCard from './MatchHistoryStatCard.svelte';
  import MatchHistoryTrendChart from './MatchHistoryTrendChart.svelte';
  import { matchHistoryActions } from '$lib/stores/matchHistoryStore';
  import { formatDamage, getImpactLabel } from '$lib/utils/matchHistory';

  interface Props {
    summary: {
      winRate: number;
      avgKDA: number;
      avgImpact: number;
      avgDamage: number;
      avgCS: number;
      avgVision: number;
      performanceGrade: { grade: string; color: string };
    };
    playerStats: {
      avg_kills?: number;
      avg_deaths?: number;
      avg_assists?: number;
    };
    matches?: unknown[];
    variant?: 'compact' | 'full';
  }

  let { summary, playerStats, matches = [], variant = 'compact' }: Props = $props();

  const impactLabel = $derived(getImpactLabel(summary.avgImpact || 0));
  const dense = $derived(variant === 'compact');
</script>

<div class="space-y-3 max-w-3xl">
  <div class="flex flex-col sm:flex-row gap-3 items-start">
    <div class="{dense ? 'w-full sm:max-w-[220px]' : 'w-full sm:max-w-xs'} shrink-0">
      <MatchHistoryTrendChart {matches} compact={dense} />
    </div>
    <div class="grid grid-cols-2 gap-2 flex-1 min-w-0 w-full sm:max-w-[280px]">
      <MatchHistoryStatCard
        label="Win Rate"
        value="{summary.winRate}%"
        icon={Trophy}
        accent="amber"
        valueClass={matchHistoryActions.getWinRateColor(summary.winRate)}
        progress={summary.winRate}
      />
      <MatchHistoryStatCard
        label="Grade"
        value={summary.performanceGrade.grade}
        icon={Award}
        accent="violet"
        valueClass={summary.performanceGrade.color}
      />
      <MatchHistoryStatCard
        label="KDA"
        value={summary.avgKDA}
        icon={Target}
        accent="rose"
        valueClass={matchHistoryActions.getKDAColor(summary.avgKDA)}
        subtitle={matchHistoryActions.formatKDA(
          playerStats.avg_kills || 0,
          playerStats.avg_deaths || 0,
          playerStats.avg_assists || 0,
        )}
      />
      <MatchHistoryStatCard
        label="Impact"
        value={summary.avgImpact}
        icon={Zap}
        accent="violet"
        valueClass={impactLabel.color}
        subtitle={impactLabel.label}
      />
    </div>
  </div>

  <div class="grid grid-cols-3 gap-2 max-w-md">
    <MatchHistoryStatCard label="Dmg" value={formatDamage(summary.avgDamage)} icon={Swords} accent="orange" />
    <MatchHistoryStatCard label="CS" value={summary.avgCS} icon={TrendingUp} accent="emerald" />
    <MatchHistoryStatCard label="Vis" value={summary.avgVision} icon={Eye} accent="sky" />
  </div>
</div>
