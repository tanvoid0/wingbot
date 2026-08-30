<script lang="ts">
  import { Crosshair, Loader2, AlertCircle, KeyRound, Flame, Lightbulb, Sparkles, RefreshCw } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import ImageWithFallback from '$lib/components/ui/ImageWithFallback.svelte';
  import {
    wsChampionSelectData,
    wsIsChampionSelectActive,
    wsGameData,
    wsLiveGameData,
    wsGamePhase,
    wsGameTime,
  } from '$lib/stores/websocketStore';
  import {
    scoutProfiles,
    scoutLoading,
    scoutError,
    scoutApiRequired,
    scoutAiInsights,
    scoutAiInsightsLoading,
    scoutAiInsightsError,
    scoutActions,
  } from '$lib/stores/scoutStore';
  import { ollamaFeatureFlags } from '$lib/stores/ollamaStatusStore';
  import {
    extractScoutableEnemies,
    extractScoutableFromLiveGame,
    scoutEnemyKey,
    type ChampSelectSlot,
  } from '$lib/utils/championSelectParser';
  import {
    formatMasteryPoints,
    getRoleLabel,
    getRankTierColor,
    getThreatLabel,
    type ScoutPlayerProfile,
  } from '$lib/utils/scout';
  import { getChampionImageUrlByIdSync, getChampionImageUrlSync, getRankEmblemUrl } from '$lib/services/imageService';

  interface Props {
    /** When set, only show in this context; otherwise auto-detect from game phase. */
    mode?: 'auto' | 'champ-select' | 'loading';
    /** Hide the card when there is nothing to scout yet. */
    hideWhenEmpty?: boolean;
  }

  let { mode = 'auto', hideWhenEmpty = false }: Props = $props();

  let targets = $state<ChampSelectSlot[]>([]);
  let lastFetchKey = $state('');
  let panelTitle = $state('Enemy Scout');
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  const loadingPhase = $derived(
    $wsGamePhase === 'GameStart' ||
      $wsGamePhase === 'InProgress' && ($wsGameTime ?? 0) < 180,
  );

  const activeMode = $derived(
    mode === 'auto'
      ? $wsIsChampionSelectActive
        ? 'champ-select'
        : loadingPhase
          ? 'loading'
          : 'none'
      : mode,
  );

  function buildFetchKey(list: ChampSelectSlot[]): string {
    return list
      .map((e) => `${scoutEnemyKey(e)}:${e.championId}:${e.championName ?? ''}`)
      .sort()
      .join('|');
  }

  function scheduleScout(
    list: ChampSelectSlot[],
    fetcher: () => Promise<unknown>,
  ) {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      void fetcher();
    }, 600);
  }

  $effect(() => {
    if (activeMode === 'none') {
      if (mode !== 'auto') return;
      targets = [];
      lastFetchKey = '';
      return;
    }

    if (activeMode === 'champ-select') {
      panelTitle = 'Enemy Scout';
      if (!$wsChampionSelectData) {
        targets = [];
        return;
      }
      const next = extractScoutableEnemies($wsChampionSelectData);
      targets = next;
      const key = buildFetchKey(next);
      if (key && key !== lastFetchKey) {
        lastFetchKey = key;
        scheduleScout(next, () =>
          scoutActions.scoutEnemiesFromSession($wsChampionSelectData),
        );
      }
      return;
    }

    if (activeMode === 'loading') {
      panelTitle = 'Loading Screen Intel';
      const gameData = ($wsGameData ?? $wsLiveGameData) as Record<string, unknown> | null;
      if (!gameData) {
        targets = [];
        return;
      }
      const next = extractScoutableFromLiveGame(gameData);
      targets = next;
      const key = buildFetchKey(next);
      if (key && key !== lastFetchKey) {
        lastFetchKey = key;
        scheduleScout(next, () => scoutActions.scoutEnemiesFromLiveGame(gameData));
      }
    }
  });

  $effect(() => {
    if (mode === 'auto' && activeMode === 'none' && !$wsIsChampionSelectActive) {
      scoutActions.clear();
    }
  });

  function profileFor(player: ChampSelectSlot): ScoutPlayerProfile | undefined {
    return $scoutProfiles[scoutEnemyKey(player)];
  }

  function championImage(player: ChampSelectSlot): string {
    if (player.championId > 0) {
      return getChampionImageUrlByIdSync(player.championId) || '';
    }
    if (player.championName) {
      return getChampionImageUrlSync(player.championName) || '';
    }
    return '';
  }

  const visible = $derived(activeMode !== 'none' && (!hideWhenEmpty || targets.length > 0 || $scoutApiRequired));

  function retryScoutAi() {
    void scoutActions.fetchAiInsights(targets, undefined, { force: true });
  }
</script>

{#if visible}
  <Card class="border-orange-500/20 bg-gradient-to-b from-orange-50/40 to-background dark:from-orange-950/10">
    <CardHeader class="pb-2">
      <CardTitle class="flex items-center gap-2 text-base">
        <Crosshair class="w-4 h-4 text-orange-500" />
        {panelTitle}
        {#if $scoutLoading}
          <Loader2 class="w-3.5 h-3.5 animate-spin text-muted-foreground" />
        {/if}
      </CardTitle>
    </CardHeader>
    <CardContent class="space-y-3">
      {#if $scoutApiRequired}
        <div class="flex items-start gap-2 rounded-lg border border-amber-500/30 bg-amber-500/10 p-2.5 text-sm">
          <KeyRound class="w-4 h-4 shrink-0 text-amber-600 mt-0.5" />
          <p class="text-muted-foreground">
            Add a Riot API key in <strong>Settings</strong> to scout enemy rank, mastery, and recent form.
          </p>
        </div>
      {:else if $scoutError && !$scoutApiRequired}
        <div class="flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/10 p-2.5 text-sm text-destructive">
          <AlertCircle class="w-4 h-4 shrink-0 mt-0.5" />
          <p>{$scoutError}</p>
        </div>
      {/if}

      {#if targets.length === 0}
        <p class="text-sm text-muted-foreground py-2">
          {#if activeMode === 'loading'}
            Enemy intel appears once loading screen shows Riot IDs for all players.
          {:else}
            Enemy intel appears when opponents lock champions and their Riot IDs are visible.
          {/if}
        </p>
      {:else}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-2">
          {#each targets as enemy (scoutEnemyKey(enemy))}
            {@const profile = profileFor(enemy)}
            {@const rank = profile?.primary_rank ?? profile?.solo_rank}
            {@const threat = profile ? getThreatLabel(profile) : null}
            <div class="rounded-xl border border-border/60 bg-card/80 p-2.5 space-y-2 shadow-sm">
              <div class="flex items-center gap-2">
                <ImageWithFallback
                  src={championImage(enemy)}
                  alt="Champion"
                  class="w-9 h-9 rounded-lg object-cover shrink-0"
                  fallback="?"
                />
                <div class="min-w-0 flex-1">
                  <p class="text-xs font-medium truncate">
                    {enemy.summonerName ?? 'Unknown'}
                  </p>
                  {#if enemy.position}
                    <p class="text-[10px] text-muted-foreground">
                      {getRoleLabel(enemy.position)}
                    </p>
                  {/if}
                </div>
              </div>

              {#if profile?.error}
                <p class="text-[10px] text-destructive line-clamp-2">{profile.error}</p>
              {:else if !profile && $scoutLoading}
                <div class="h-8 flex items-center justify-center">
                  <Loader2 class="w-4 h-4 animate-spin text-muted-foreground" />
                </div>
              {:else if profile}
                {#if rank}
                  <div class="flex items-center gap-1.5">
                    {#if getRankEmblemUrl(rank.tier)}
                      <ImageWithFallback
                        src={getRankEmblemUrl(rank.tier)}
                        alt={rank.tier}
                        class="w-4 h-4 object-contain shrink-0"
                        fallback={rank.tier.charAt(0)}
                        width={16}
                        height={16}
                      />
                    {/if}
                    <span class="text-[11px] font-medium truncate {getRankTierColor(rank.tier)}">
                      {rank.summary}
                    </span>
                  </div>
                  <p class="text-[10px] text-muted-foreground">
                    {rank.wins}W {rank.losses}L · {rank.win_rate}% WR
                  </p>
                {:else}
                  <p class="text-[10px] text-muted-foreground">Unranked</p>
                {/if}

                {#if profile.champion_mastery}
                  <p class="text-[10px] text-muted-foreground">
                    M{profile.champion_mastery.level}
                    · {formatMasteryPoints(profile.champion_mastery.points)} pts
                  </p>
                {/if}

                {#if profile.champion_form}
                  <p class="text-[10px] text-muted-foreground">
                    Recent: {profile.champion_form.summary}
                  </p>
                {/if}

                {#if threat}
                  <Badge
                    variant="outline"
                    class="text-[9px] h-5 px-1.5 gap-0.5 {profile.is_one_trick
                      ? 'border-orange-500/40 text-orange-600 bg-orange-500/10'
                      : ''}"
                  >
                    {#if profile.is_one_trick}
                      <Flame class="w-2.5 h-2.5" />
                    {/if}
                    {threat}
                  </Badge>
                {/if}
              {/if}
            </div>
          {/each}
        </div>

        {#if $ollamaFeatureFlags.scout && ($scoutAiInsights.length > 0 || $scoutAiInsightsLoading || $scoutAiInsightsError)}
          <div class="rounded-xl border border-violet-100 dark:border-violet-900/30 bg-violet-50/40 dark:bg-violet-950/15 p-2.5 space-y-1.5">
            <div class="flex items-center gap-1.5 text-xs font-semibold text-muted-foreground">
              <Lightbulb class="w-3.5 h-3.5 text-amber-500" />
              AI Draft Notes
              {#if $scoutAiInsightsLoading}
                <Loader2 class="w-3 h-3 animate-spin text-violet-500" />
              {/if}
            </div>
            {#each $scoutAiInsights as insight}
              <div class="rounded-lg border border-violet-200/60 dark:border-violet-900/40 bg-white/70 dark:bg-card/60 p-2">
                <div class="flex items-center gap-1.5">
                  <p class="text-xs font-semibold">{insight.title}</p>
                  <Sparkles class="w-3 h-3 text-violet-500 shrink-0" />
                </div>
                <p class="text-[10px] text-muted-foreground mt-0.5 leading-snug">{insight.message}</p>
              </div>
            {/each}
            {#if $scoutAiInsightsError && !$scoutAiInsightsLoading}
              <div class="flex items-start gap-2 text-[10px] text-amber-700 dark:text-amber-300">
                <AlertCircle class="w-3.5 h-3.5 shrink-0 mt-0.5" />
                <div>
                  <p>{$scoutAiInsightsError}</p>
                  <button
                    type="button"
                    onclick={retryScoutAi}
                    class="mt-1 inline-flex items-center gap-1 font-medium text-violet-600 hover:text-violet-700"
                  >
                    <RefreshCw class="w-3 h-3" />
                    Retry
                  </button>
                </div>
              </div>
            {/if}
          </div>
        {/if}
      {/if}
    </CardContent>
  </Card>
{/if}
