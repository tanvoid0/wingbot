<script lang="ts">
  import { ExternalLink, Sparkles } from '@lucide/svelte';
  import { Badge } from '$lib/components/ui/badge';
  import ImageWithFallback from '$lib/components/ui/ImageWithFallback.svelte';
  import { getRankEmblemUrl } from '$lib/services/imageService';
  import { getRankedInfo, getImpactLabel } from '$lib/utils/matchHistory';

  interface Props {
    playerStats: {
      wins?: number;
      losses?: number;
      ranked?: unknown;
    };
    summary: {
      totalMatches: number;
      avgImpact: number;
      winRate: number;
      performanceGrade: { grade: string; color: string };
    };
    showFullLink?: boolean;
    variant?: 'compact' | 'full';
  }

  let { playerStats, summary, showFullLink = true, variant = 'compact' }: Props = $props();

  const ranked = $derived(getRankedInfo(playerStats.ranked));
  const rankEmblem = $derived(ranked ? getRankEmblemUrl(ranked.tier) : null);
  const impactLabel = $derived(getImpactLabel(summary.avgImpact || 0));
  const dense = $derived(variant === 'compact');
</script>

<div
  class="relative overflow-hidden rounded-xl bg-gradient-to-br from-violet-600 via-purple-600 to-fuchsia-600 text-white shadow-md {dense ? 'p-3' : 'p-4'}"
>
  <div class="relative flex items-center gap-3">
    <div
      class="shrink-0 rounded-xl bg-white/20 backdrop-blur-sm border border-white/30 flex items-center justify-center {dense ? 'w-11 h-11' : 'w-12 h-12'}"
    >
      <span class="{dense ? 'text-2xl' : 'text-2xl'} font-black">{summary.performanceGrade.grade}</span>
    </div>

    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-1.5 flex-wrap">
        <h2 class="{dense ? 'text-sm' : 'text-base'} font-bold leading-tight">Performance</h2>
        <Badge class="bg-white/20 text-white border-white/30 text-[9px] h-5 px-1.5 hover:bg-white/20">
          <Sparkles class="w-2.5 h-2.5 mr-0.5" />
          {impactLabel.label}
        </Badge>
      </div>

      <div class="flex flex-wrap items-center gap-1.5 mt-1 text-[11px]">
        <span class="inline-flex rounded-full bg-white/20 px-2 py-0.5 font-medium">
          <span class="text-emerald-200">{playerStats.wins ?? 0}W</span>
          <span class="mx-0.5 opacity-60">/</span>
          <span class="text-rose-200">{playerStats.losses ?? 0}L</span>
        </span>
        <span class="text-white/75">{summary.totalMatches}g · {summary.winRate}%</span>
        {#if ranked}
          <span class="inline-flex items-center gap-1 rounded-full bg-white/15 px-1.5 py-0.5 truncate max-w-[140px]">
            {#if rankEmblem}
              <ImageWithFallback
                src={rankEmblem}
                alt={ranked.tier}
                fallback={ranked.tier.charAt(0)}
                class="w-3.5 h-3.5 object-contain shrink-0"
                width={14}
                height={14}
              />
            {/if}
            <span class="truncate">{ranked.summary}</span>
          </span>
        {/if}
      </div>
    </div>

    <div class="text-right shrink-0">
      <p class="text-[9px] uppercase tracking-wide text-white/70">Impact</p>
      <p class="{dense ? 'text-lg' : 'text-xl'} font-bold leading-none">{summary.avgImpact}</p>
      {#if showFullLink}
        <a
          href="/match-history"
          class="mt-0.5 inline-flex items-center gap-0.5 text-[9px] text-white/75 hover:text-white"
        >
          More
          <ExternalLink class="w-2.5 h-2.5" />
        </a>
      {/if}
    </div>
  </div>
</div>
