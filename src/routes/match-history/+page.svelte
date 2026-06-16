<script lang="ts">
  import { RefreshCw, Trophy, Play, Settings, Sparkles } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';
  import ImageWithFallback from '$lib/components/ui/ImageWithFallback.svelte';
  import MatchHistoryAnalysis from '$lib/components/live-game/MatchHistoryAnalysis.svelte';
  import {
    currentPlayerStats,
    matchHistoryActions,
    matchHistoryLoading,
    matchHistorySummary,
    recentMatches,
  } from '$lib/stores/matchHistoryStore';
  import { getChampionImageUrlByIdSync, getChampionFallback } from '$lib/services/imageService';
  import { openLauncherModal } from '$lib/stores/appModalsStore';
  import {
    getGradeBadgeClass,
    getRecentStreakDots,
    getTrendLabel,
  } from '$lib/utils/matchHistory';

  let refreshError = $state<string | null>(null);

  let summary = $derived($matchHistorySummary);
  let hasData = $derived(!!$currentPlayerStats);
  let streakDots = $derived(getRecentStreakDots($recentMatches));
  let trendInfo = $derived(getTrendLabel(summary.recentTrend.trend));
  let isRefreshing = $derived($matchHistoryLoading);

  async function refreshAnalysis() {
    try {
      refreshError = null;
      await matchHistoryActions.fetchAnalysis({ forceRefresh: true });
    } catch (err) {
      refreshError = err instanceof Error ? err.message : String(err);
    }
  }

  function champIcon(id: number | string) {
    const numId = typeof id === 'string' ? parseInt(id, 10) : id;
    return getChampionImageUrlByIdSync(numId) || null;
  }
</script>

<div class="min-h-full bg-gradient-to-b from-violet-50/60 via-background to-background dark:from-violet-950/20">
  <div class="p-4 lg:p-6 space-y-4 max-w-4xl mx-auto">
    <div
      class="flex items-center justify-between gap-3 rounded-2xl bg-gradient-to-r from-violet-600 to-purple-600 px-4 py-3 text-white shadow-md"
    >
      <div class="flex items-center gap-3 min-w-0">
        <div class="p-2 rounded-xl bg-white/15 shrink-0">
          <Trophy class="w-5 h-5" />
        </div>
        <div class="min-w-0">
          <h1 class="text-lg font-bold tracking-tight">Match History</h1>
          <p class="text-xs text-white/80 truncate">
            {#if hasData}
              Last {summary.matchCountLoaded} games
            {:else}
              Recent League performance
            {/if}
          </p>
        </div>
      </div>
      <Button
        onclick={refreshAnalysis}
        disabled={isRefreshing}
        size="sm"
        class="gap-1 shrink-0 bg-white text-violet-700 hover:bg-white/90 border-0 h-8 text-xs"
      >
        <RefreshCw class="w-3.5 h-3.5 {isRefreshing ? 'animate-spin' : ''}" />
        Refresh
      </Button>
    </div>

    {#if refreshError}
      <p class="text-xs text-destructive">{refreshError}</p>
    {/if}

    {#if hasData}
      <div class="flex flex-col lg:flex-row lg:items-start gap-4">
        <div class="flex-1 min-w-0">
          <MatchHistoryAnalysis variant="full" showFullLink={false} />
        </div>

        <div class="w-full lg:w-52 shrink-0 space-y-3">
          <div
            class="rounded-xl bg-white/90 dark:bg-card/95 border border-violet-100 dark:border-violet-900/30 p-3 shadow-sm"
          >
            <div class="flex items-center gap-1.5 mb-3">
              <Sparkles class="w-3.5 h-3.5 text-violet-600" />
              <h2 class="text-xs font-semibold">Summary</h2>
            </div>

            <div class="flex items-center gap-3 mb-3">
              <div
                class="w-12 h-12 rounded-xl flex items-center justify-center text-2xl font-black border {getGradeBadgeClass(summary.performanceGrade.grade)}"
              >
                {summary.performanceGrade.grade}
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-[10px] text-muted-foreground">Win rate</p>
                <p class="text-lg font-bold leading-tight {matchHistoryActions.getWinRateColor(summary.winRate)}">
                  {summary.winRate}%
                </p>
              </div>
            </div>

            <div class="h-1.5 rounded-full bg-violet-100 dark:bg-violet-950/50 overflow-hidden mb-3">
              <div
                class="h-full rounded-full bg-gradient-to-r from-violet-500 to-purple-600"
                style="width: {summary.winRate}%"
              ></div>
            </div>

            <div class="flex justify-between text-[10px] mb-1.5">
              <span class="text-muted-foreground">Form</span>
              <span class="font-medium {trendInfo.color}">{trendInfo.label}</span>
            </div>
            {#if streakDots.length > 0}
              <div class="flex gap-1 justify-center">
                {#each streakDots as won}
                  <div
                    class="w-2.5 h-2.5 rounded-full {won ? 'bg-green-500' : 'bg-red-500'}"
                    title={won ? 'Win' : 'Loss'}
                  ></div>
                {/each}
              </div>
            {/if}
          </div>

          {#if summary.topChampions.length > 0}
            <div
              class="rounded-xl bg-white/90 dark:bg-card/95 border border-violet-100 dark:border-violet-900/30 p-3 shadow-sm"
            >
              <h2 class="text-xs font-semibold mb-2">Top picks</h2>
              <div class="space-y-2">
                {#each summary.topChampions.slice(0, 3) as champion, i}
                  {@const icon = champIcon(champion.championId)}
                  <div class="flex items-center gap-2">
                    <span class="text-[10px] font-bold text-violet-400 w-3">{i + 1}</span>
                    <div class="w-7 h-7 rounded-md overflow-hidden ring-1 ring-violet-200 shrink-0">
                      <ImageWithFallback
                        src={icon}
                        alt={champion.championName}
                        fallback={getChampionFallback(champion.championName)}
                        class="w-full h-full object-cover"
                        width={28}
                        height={28}
                      />
                    </div>
                    <div class="flex-1 min-w-0">
                      <p class="text-xs font-medium truncate">{champion.championName}</p>
                      <p class="text-[10px] text-muted-foreground">
                        {champion.games}g ·
                        <span class={matchHistoryActions.getWinRateColor(champion.winRate)}>
                          {champion.winRate.toFixed(0)}%
                        </span>
                      </p>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      </div>
    {:else}
      <div
        class="rounded-2xl border border-dashed border-violet-200 dark:border-violet-800 bg-white/60 dark:bg-card/40 p-8 text-center max-w-md mx-auto"
      >
        <div class="p-3 rounded-xl bg-violet-100/80 dark:bg-violet-950/40 w-fit mx-auto mb-3">
          <Trophy class="w-8 h-8 text-violet-500" />
        </div>
        <h2 class="text-base font-bold mb-1">No match history yet</h2>
        <p class="text-xs text-muted-foreground mb-5 leading-relaxed">
          Launch League or configure your Riot API key. Data syncs when the client connects.
        </p>
        <div class="flex items-center justify-center gap-2 flex-wrap">
          <Button
            onclick={refreshAnalysis}
            disabled={isRefreshing}
            size="sm"
            class="gap-1 bg-gradient-to-r from-violet-600 to-purple-600 hover:from-violet-700 hover:to-purple-700 h-8 text-xs"
          >
            <RefreshCw class="w-3.5 h-3.5 {isRefreshing ? 'animate-spin' : ''}" />
            Retry
          </Button>
          <Button onclick={openLauncherModal} variant="outline" size="sm" class="gap-1 h-8 text-xs">
            <Play class="w-3.5 h-3.5" />
            Launch
          </Button>
          <Button href="/settings" variant="outline" size="sm" class="gap-1 h-8 text-xs">
            <Settings class="w-3.5 h-3.5" />
            Settings
          </Button>
        </div>
      </div>
    {/if}
  </div>
</div>
