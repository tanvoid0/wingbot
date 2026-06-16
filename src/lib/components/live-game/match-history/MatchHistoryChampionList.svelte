<script lang="ts">
  import { Users, Target, Zap, ChevronDown } from '@lucide/svelte';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import ImageWithFallback from '$lib/components/ui/ImageWithFallback.svelte';
  import { getChampionImageUrlByIdSync, getChampionFallback } from '$lib/services/imageService';
  import { getImpactLabel } from '$lib/utils/matchHistory';
  import { matchHistoryActions } from '$lib/stores/matchHistoryStore';

  interface Champion {
    championId: string | number;
    championName: string;
    games: number;
    winRate: number;
    avgKDA: number;
    avgImpact: number;
  }

  interface Props {
    champions: Champion[];
    totalCount?: number;
    variant?: 'compact' | 'full';
  }

  let { champions, totalCount = champions.length, variant = 'compact' }: Props = $props();

  const initialVisible = $derived(variant === 'full' ? 10 : 5);
  let visibleCount = $state(10);

  $effect(() => {
    visibleCount = initialVisible;
  });

  const visibleChampions = $derived(champions.slice(0, visibleCount));
  const canShowMore = $derived(champions.length > visibleCount);

  function champIcon(id: number | string) {
    const numId = typeof id === 'string' ? parseInt(id, 10) : id;
    return getChampionImageUrlByIdSync(numId) || null;
  }
</script>

<div class="space-y-2 max-w-2xl">
  <div class="flex items-center justify-between gap-2">
    <h4 class="text-xs font-semibold text-muted-foreground">Champions</h4>
    {#if totalCount > 0}
      <span class="text-[10px] text-muted-foreground">{totalCount} played</span>
    {/if}
  </div>
  {#if champions.length > 0}
    <div class="space-y-1.5">
      {#each visibleChampions as champion}
        {@const icon = champIcon(champion.championId)}
        {@const mastery = matchHistoryActions.getChampionMastery(champion)}
        {@const impact = getImpactLabel(champion.avgImpact)}
        <div class="rounded-lg bg-white/90 dark:bg-card/95 border border-violet-100 dark:border-violet-900/25 p-2.5">
          <div class="flex items-center gap-2.5">
            <div class="relative shrink-0">
              <div class="w-10 h-10 rounded-lg overflow-hidden ring-1 ring-violet-200 dark:ring-violet-800">
                <ImageWithFallback
                  src={icon}
                  alt={champion.championName}
                  fallback={getChampionFallback(champion.championName)}
                  class="w-full h-full object-cover"
                  width={40}
                  height={40}
                />
              </div>
              <span
                class="absolute -bottom-0.5 -right-0.5 text-[8px] font-bold px-1 rounded-full bg-violet-600 text-white"
              >
                {champion.winRate.toFixed(0)}%
              </span>
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-1.5 flex-wrap">
                <span class="text-sm font-semibold truncate">{champion.championName}</span>
                <Badge variant="outline" class="text-[9px] h-4 px-1">{champion.games}g</Badge>
                <Badge variant="outline" class="text-[9px] h-4 px-1">{mastery.text}</Badge>
              </div>
              <div class="flex items-center gap-2 mt-0.5 text-[10px] text-muted-foreground">
                <span class="inline-flex items-center gap-0.5">
                  <Target class="w-2.5 h-2.5" />
                  {champion.avgKDA.toFixed(2)}
                </span>
                <span class="inline-flex items-center gap-0.5 {impact.color}">
                  <Zap class="w-2.5 h-2.5" />
                  {Math.round(champion.avgImpact)}
                </span>
              </div>
            </div>
          </div>
          <div class="mt-1.5 h-1 rounded-full bg-violet-100 dark:bg-violet-950/40 overflow-hidden">
            <div
              class="h-full rounded-full bg-gradient-to-r from-violet-500 to-purple-500"
              style="width: {champion.winRate}%"
            ></div>
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
        Show {Math.min((variant === 'full' ? 10 : 5), champions.length - visibleCount)} more
      </Button>
    {/if}
  {:else}
    <div class="text-center py-6 rounded-lg border border-dashed border-violet-200 dark:border-violet-800">
      <Users class="w-6 h-6 mx-auto mb-1 text-violet-300" />
      <p class="text-xs text-muted-foreground">No champion data</p>
    </div>
  {/if}
</div>
