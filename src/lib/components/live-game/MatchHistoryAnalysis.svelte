<script lang="ts">
  import { Trophy, LayoutDashboard, Users, History, ChevronDown, Loader2 } from '@lucide/svelte';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import {
    currentPlayerStats,
    matchHistoryActions,
    matchHistoryLoading,
    matchHistorySummary,
    MAX_MATCH_COUNT,
    MATCH_COUNT_STEP,
    playerInsights,
    recentMatches,
  } from '$lib/stores/matchHistoryStore';
  import MatchHistoryHeroBanner from './match-history/MatchHistoryHeroBanner.svelte';
  import MatchHistoryStatGrid from './match-history/MatchHistoryStatGrid.svelte';
  import MatchHistoryRoleGrid from './match-history/MatchHistoryRoleGrid.svelte';
  import MatchHistoryInsights from './match-history/MatchHistoryInsights.svelte';
  import MatchHistoryChampionList from './match-history/MatchHistoryChampionList.svelte';
  import MatchHistoryRecentList from './match-history/MatchHistoryRecentList.svelte';

  interface Props {
    variant?: 'compact' | 'full';
    showFullLink?: boolean;
  }

  let { variant = 'compact', showFullLink = true }: Props = $props();

  let summary = $derived($matchHistorySummary);
  let insights = $derived($playerInsights);
  let loadMoreError = $state<string | null>(null);

  const championsForTab = $derived(
    variant === 'full' ? summary.allChampions : summary.topChampions,
  );
  const canLoadMore = $derived(
    summary.hasMoreMatches && summary.matchCountLoaded < MAX_MATCH_COUNT,
  );

  async function loadMoreMatches() {
    try {
      loadMoreError = null;
      await matchHistoryActions.loadMoreMatches();
    } catch (err) {
      loadMoreError = err instanceof Error ? err.message : String(err);
    }
  }
</script>

{#if $currentPlayerStats}
  <div
    class="rounded-2xl border border-violet-100/80 dark:border-violet-900/30 bg-gradient-to-b from-violet-50/50 to-background dark:from-violet-950/15 dark:to-background shadow-sm overflow-hidden {variant === 'full' ? 'max-w-3xl' : ''}"
  >
    <div class="p-3 {variant === 'full' ? 'sm:p-4' : ''} space-y-3">
      <MatchHistoryHeroBanner
        playerStats={$currentPlayerStats}
        {summary}
        {variant}
        showFullLink={showFullLink && variant === 'compact'}
      />

      <Tabs value="overview" class="space-y-3">
        <TabsList
          class="inline-flex h-auto w-auto max-w-full p-1 rounded-xl bg-white/70 dark:bg-card/60 border border-violet-100 dark:border-violet-900/30"
        >
          <TabsTrigger
            value="overview"
            class="gap-1 px-2.5 py-1.5 text-xs rounded-lg data-[state=active]:bg-gradient-to-r data-[state=active]:from-violet-600 data-[state=active]:to-purple-600 data-[state=active]:text-white border-0"
          >
            <LayoutDashboard class="w-3.5 h-3.5" />
            <span>Overview</span>
          </TabsTrigger>
          <TabsTrigger
            value="champions"
            class="gap-1 px-2.5 py-1.5 text-xs rounded-lg data-[state=active]:bg-gradient-to-r data-[state=active]:from-violet-600 data-[state=active]:to-purple-600 data-[state=active]:text-white border-0"
          >
            <Users class="w-3.5 h-3.5" />
            <span>Champs</span>
          </TabsTrigger>
          <TabsTrigger
            value="recent"
            class="gap-1 px-2.5 py-1.5 text-xs rounded-lg data-[state=active]:bg-gradient-to-r data-[state=active]:from-violet-600 data-[state=active]:to-purple-600 data-[state=active]:text-white border-0"
          >
            <History class="w-3.5 h-3.5" />
            <span>Recent</span>
          </TabsTrigger>
        </TabsList>

        <TabsContent value="overview" class="space-y-3 mt-0">
          <MatchHistoryStatGrid
            {summary}
            playerStats={$currentPlayerStats}
            matches={$recentMatches}
            {variant}
          />
          {#if variant === 'full'}
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3 max-w-3xl">
              <MatchHistoryRoleGrid roleStats={summary.roleStats} />
              <MatchHistoryInsights insights={insights} />
            </div>
          {:else}
            <MatchHistoryRoleGrid roleStats={summary.roleStats} />
            <MatchHistoryInsights insights={insights} />
          {/if}
        </TabsContent>

        <TabsContent value="champions" class="mt-0">
          <MatchHistoryChampionList
            champions={championsForTab}
            totalCount={summary.allChampions.length}
            {variant}
          />
        </TabsContent>

        <TabsContent value="recent" class="mt-0">
          <MatchHistoryRecentList
            matches={$recentMatches}
            recentTrend={summary.recentTrend}
            {variant}
          />
        </TabsContent>
      </Tabs>

      {#if variant === 'full'}
        <div class="space-y-1.5 pt-1 border-t border-violet-100 dark:border-violet-900/30">
          <p class="text-[10px] text-muted-foreground text-center">
            Stats from last {summary.matchCountLoaded} games
            {#if summary.allChampions.length > 0}
              · {summary.allChampions.length} champions
            {/if}
          </p>
          {#if canLoadMore}
            <Button
              variant="outline"
              size="sm"
              class="w-full h-8 text-xs gap-1.5 border-violet-200 dark:border-violet-800"
              onclick={loadMoreMatches}
              disabled={$matchHistoryLoading}
            >
              {#if $matchHistoryLoading}
                <Loader2 class="w-3.5 h-3.5 animate-spin" />
                Loading…
              {:else}
                <ChevronDown class="w-3.5 h-3.5" />
                Load {MATCH_COUNT_STEP} more games
              {/if}
            </Button>
          {:else if summary.matchCountLoaded >= MAX_MATCH_COUNT}
            <p class="text-[10px] text-muted-foreground text-center">Maximum sample loaded</p>
          {/if}
          {#if loadMoreError}
            <p class="text-[10px] text-destructive text-center">{loadMoreError}</p>
          {/if}
        </div>
      {/if}
    </div>
  </div>
{:else}
  <div
    class="rounded-2xl border border-dashed border-violet-200 dark:border-violet-900/40 bg-violet-50/30 dark:bg-violet-950/10 p-5 text-center max-w-sm"
  >
    <div class="p-3 rounded-xl bg-white/80 dark:bg-card/80 w-fit mx-auto mb-3 shadow-sm">
      <Trophy class="w-7 h-7 text-violet-400" />
    </div>
    <p class="text-sm font-semibold text-foreground">Waiting for match history</p>
    <p class="text-xs text-muted-foreground mt-0.5">Data loads when League is connected</p>
    <Badge variant="outline" class="mt-2 text-[10px] text-muted-foreground">Inactive</Badge>
    {#if showFullLink}
      <a href="/match-history" class="block text-xs text-violet-600 hover:underline mt-2">
        Open match history page
      </a>
    {/if}
  </div>
{/if}
