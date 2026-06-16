<script lang="ts">
  import { ChevronDown, Clock } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';
  import ImageWithFallback from '$lib/components/ui/ImageWithFallback.svelte';
  import { getChampionImageUrlByIdSync, getChampionFallback, getRoleIcon } from '$lib/services/imageService';
  import {
    formatKdaString,
    formatDamage,
    formatCsPerMin,
    getTrendLabel,
    getRecentStreakDots,
    getRoleLabel,
  } from '$lib/utils/matchHistory';

  interface Props {
    matches: unknown[];
    recentTrend: { trend: string; color: string };
    variant?: 'compact' | 'full';
  }

  let { matches, recentTrend, variant = 'compact' }: Props = $props();

  const initialVisible = $derived(variant === 'full' ? 10 : 5);
  let visibleCount = $state(10);

  $effect(() => {
    visibleCount = initialVisible;
  });

  const visibleMatches = $derived(matches.slice(0, visibleCount));
  const canShowMore = $derived(matches.length > visibleCount);
  const trendInfo = $derived(getTrendLabel(recentTrend.trend));
  const streakDots = $derived(getRecentStreakDots(matches));

  function champIcon(id: number | string) {
    const numId = typeof id === 'string' ? parseInt(id, 10) : id;
    return getChampionImageUrlByIdSync(numId) || null;
  }
</script>

<div class="space-y-2 max-w-2xl">
  <div class="flex items-center justify-between gap-2 rounded-lg bg-white/70 dark:bg-card/60 border border-violet-100 dark:border-violet-900/30 px-2.5 py-2">
    <div>
      <h4 class="text-xs font-semibold">Recent</h4>
      <p class="text-[10px] text-muted-foreground">{trendInfo.label}</p>
    </div>
    {#if streakDots.length > 0}
      <div class="flex gap-0.5">
        {#each streakDots as won}
          <div
            class="w-2 h-2 rounded-full {won ? 'bg-green-500' : 'bg-red-500'}"
            title={won ? 'Win' : 'Loss'}
          ></div>
        {/each}
      </div>
    {/if}
  </div>

  {#if matches.length > 0}
    <div class="space-y-1.5">
      {#each visibleMatches as match}
        {@const m = match as {
          stats?: { win?: boolean; kills?: number; deaths?: number; assists?: number; cs?: number; damage?: number };
          championId?: number | string;
          championName?: string;
          queueName?: string;
          role?: string;
          gameDuration?: number;
          gameCreation?: number;
          impactScore?: number;
          impactLabel?: string;
        }}
        {@const stats = m.stats || {}}
        {@const icon = champIcon(m.championId ?? 0)}
        {@const impact = m.impactScore ?? 0}
        <div
          class="flex rounded-lg bg-white/90 dark:bg-card/95 border border-violet-100/80 dark:border-violet-900/25 overflow-hidden"
        >
          <div class="w-1 shrink-0 {stats.win ? 'bg-green-500' : 'bg-red-500'}"></div>
          <div class="flex items-center gap-2.5 p-2 flex-1 min-w-0">
            <div class="w-9 h-9 rounded-lg overflow-hidden ring-1 ring-violet-100 dark:ring-violet-900 shrink-0">
              <ImageWithFallback
                src={icon}
                alt={m.championName || 'Champion'}
                fallback={getChampionFallback(m.championName)}
                class="w-full h-full object-cover"
                width={36}
                height={36}
              />
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-1.5 text-xs">
                <span class="font-semibold truncate">{m.championName || 'Unknown'}</span>
                <span class="text-[10px] font-bold {stats.win ? 'text-green-600' : 'text-red-600'}">
                  {stats.win ? 'W' : 'L'}
                </span>
                {#if m.role}
                  <span title={getRoleLabel(m.role)}>{getRoleIcon(m.role)}</span>
                {/if}
              </div>
              <div class="text-[10px] text-muted-foreground mt-0.5 flex flex-wrap gap-x-1.5">
                <span>{formatKdaString(stats.kills || 0, stats.deaths || 0, stats.assists || 0)}</span>
                {#if m.gameDuration}
                  <span>{formatCsPerMin(stats.cs || 0, m.gameDuration)}/m</span>
                {/if}
                <span>{formatDamage(stats.damage || 0)}</span>
              </div>
            </div>
            <div class="text-right shrink-0">
              <span class="text-sm font-bold text-violet-600 dark:text-violet-400">{Math.round(impact)}</span>
              <p class="text-[9px] text-muted-foreground">
                {new Date(m.gameCreation || Date.now()).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })}
              </p>
            </div>
          </div>
        </div>
      {/each}
    </div>
    {#if canShowMore}
      <Button
        variant="ghost"
        size="sm"
        class="w-full h-7 text-[10px] gap-1 text-violet-600 hover:text-violet-700"
        onclick={() => (visibleCount += variant === 'full' ? 10 : 5)}
      >
        <ChevronDown class="w-3 h-3" />
        Show {Math.min(variant === 'full' ? 10 : 5, matches.length - visibleCount)} more
      </Button>
    {/if}
  {:else}
    <div class="text-center py-6 rounded-lg border border-dashed border-violet-200 dark:border-violet-800">
      <Clock class="w-6 h-6 mx-auto mb-1 text-violet-300" />
      <p class="text-xs text-muted-foreground">No recent matches</p>
    </div>
  {/if}
</div>
