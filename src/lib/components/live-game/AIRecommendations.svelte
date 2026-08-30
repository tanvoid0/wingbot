<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Zap, Target, Sword, Shield, RefreshCw, AlertCircle } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import {
    wsGameMode,
    wsGameData,
    wsChampionSelect,
    wsIsChampionSelectActive,
    wsGamePhase,
  } from '$lib/stores/websocketStore';
  import { ollamaEnabled, ollamaFeatureFlags } from '$lib/stores/ollamaStatusStore';

  interface RecommendationCard {
    title: string;
    content: string;
    variant: 'blue' | 'green' | 'yellow' | 'purple';
  }

  const CARD_STYLES: Record<
    RecommendationCard['variant'],
    { bg: string; icon: string; title: string; text: string }
  > = {
    blue: {
      bg: 'bg-blue-50 dark:bg-blue-950',
      icon: 'text-blue-600',
      title: 'text-blue-900 dark:text-blue-100',
      text: 'text-blue-800 dark:text-blue-200',
    },
    green: {
      bg: 'bg-green-50 dark:bg-green-950',
      icon: 'text-green-600',
      title: 'text-green-900 dark:text-green-100',
      text: 'text-green-800 dark:text-green-200',
    },
    yellow: {
      bg: 'bg-yellow-50 dark:bg-yellow-950',
      icon: 'text-yellow-600',
      title: 'text-yellow-900 dark:text-yellow-100',
      text: 'text-yellow-800 dark:text-yellow-200',
    },
    purple: {
      bg: 'bg-purple-50 dark:bg-purple-950',
      icon: 'text-purple-600',
      title: 'text-purple-900 dark:text-purple-100',
      text: 'text-purple-800 dark:text-purple-200',
    },
  };

  const CARD_ICONS = [Target, Sword, Shield];
  const CARD_VARIANTS: RecommendationCard['variant'][] = ['blue', 'green', 'yellow'];
  const CARD_TITLES = ['Strategy', 'Combat Advice', 'Game Plan'];

  let gameMode = $derived($wsGameMode);
  let ollamaOn = $derived($ollamaEnabled && $ollamaFeatureFlags.liveCoach);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let recommendations = $state<RecommendationCard[]>([]);
  let lastFetchKey = $state('');
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  function isArenaMode() {
    return gameMode === 'Arena';
  }

  function parseRecommendations(text: string): RecommendationCard[] {
    const trimmed = text.trim();
    if (!trimmed) return [];

    const paragraphs = trimmed
      .split(/\n\s*\n/)
      .map((p) => p.trim())
      .filter(Boolean);

    if (paragraphs.length > 1) {
      return paragraphs.slice(0, 3).map((content, i) => ({
        title: CARD_TITLES[i] ?? 'Insight',
        content,
        variant: CARD_VARIANTS[i] ?? 'blue',
      }));
    }

    const sentences = trimmed.split(/(?<=[.!?])\s+/).filter(Boolean);
    if (sentences.length <= 2) {
      return [{ title: 'AI Coach', content: trimmed, variant: 'blue' }];
    }

    const chunkSize = Math.ceil(sentences.length / 3);
    const cards: RecommendationCard[] = [];
    for (let i = 0; i < 3; i++) {
      const chunk = sentences.slice(i * chunkSize, (i + 1) * chunkSize).join(' ').trim();
      if (chunk) {
        cards.push({
          title: CARD_TITLES[i] ?? 'Insight',
          content: chunk,
          variant: CARD_VARIANTS[i] ?? 'blue',
        });
      }
    }
    return cards.length > 0 ? cards : [{ title: 'AI Coach', content: trimmed, variant: 'blue' }];
  }

  function buildFetchKey(): string {
    const phase = $wsGamePhase ?? '';
    const cs = $wsChampionSelect;
    const live = $wsGameData;
    const csKey = cs
      ? JSON.stringify(
          (cs.myTeam as unknown[] | undefined)?.map((p: unknown) => {
            const player = p as Record<string, unknown>;
            return player.championId ?? player.championPickIntent;
          }) ?? [],
        )
      : '';
    const liveKey = live
      ? `${live.gameData?.gameTime ?? 0}-${live.activePlayer?.level ?? 0}`
      : '';
    return `${phase}:${csKey}:${liveKey}`;
  }

  async function fetchRecommendations(force = false) {
    if (!ollamaOn) return;

    const fetchKey = buildFetchKey();
    if (!force && fetchKey === lastFetchKey && recommendations.length > 0) return;
    if (!force && !fetchKey.replace(/^:*|:*$/g, '')) return;

    loading = true;
    error = null;

    try {
      const text = await invoke<string>('get_ai_recommendations');
      recommendations = parseRecommendations(text);
      lastFetchKey = fetchKey;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      recommendations = [];
    } finally {
      loading = false;
    }
  }

  function scheduleFetch() {
    if (!ollamaOn) return;
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      void fetchRecommendations();
    }, 1500);
  }

  onMount(() => {
    if (ollamaOn) void fetchRecommendations();
    return () => {
      if (debounceTimer) clearTimeout(debounceTimer);
    };
  });
  $effect(() => {
    if (!ollamaOn) return;
    void $wsGamePhase;
    void $wsGameData;
    void $wsChampionSelect;
    void $wsIsChampionSelectActive;
    scheduleFetch();
  });
</script>

<Card>
  <CardHeader>
    <div class="flex items-center justify-between gap-2">
      <CardTitle class="flex items-center gap-2">
        <Zap class="w-5 h-5" />
        AI Recommendations
      </CardTitle>
      {#if ollamaOn}
        <Button
          variant="ghost"
          size="sm"
          onclick={() => fetchRecommendations(true)}
          disabled={loading}
          title="Refresh recommendations"
        >
          <RefreshCw class="w-4 h-4 {loading ? 'animate-spin' : ''}" />
        </Button>
      {/if}
    </div>
  </CardHeader>
  <CardContent>
    <div class="space-y-4">
      {#if !ollamaOn}
        <div class="p-4 border rounded-lg bg-muted/30">
          <div class="flex items-start gap-2">
            <AlertCircle class="w-4 h-4 text-muted-foreground mt-0.5 shrink-0" />
            <p class="text-sm text-muted-foreground">
              Enable Ollama in Settings to get live coaching from the <code class="text-xs">wingbot-lol</code> model.
              Start Ollama first, then toggle AI recommendations on.
            </p>
          </div>
        </div>
      {:else if loading && recommendations.length === 0}
        <div class="flex items-center gap-2 p-4 text-sm text-muted-foreground">
          <RefreshCw class="w-4 h-4 animate-spin" />
          Generating recommendations…
        </div>
      {:else if error}
        <div class="p-4 border rounded-lg border-destructive/30 bg-destructive/10">
          <div class="flex items-start gap-2">
            <AlertCircle class="w-4 h-4 text-destructive mt-0.5 shrink-0" />
            <div class="space-y-2">
              <p class="text-sm text-destructive">{error}</p>
              <Button variant="outline" size="sm" onclick={() => fetchRecommendations(true)}>
                Retry
              </Button>
            </div>
          </div>
        </div>
      {:else if recommendations.length > 0}
        {#each recommendations as rec, i}
          {@const styles = CARD_STYLES[rec.variant]}
          {@const Icon = isArenaMode() && i === 0 ? Target : CARD_ICONS[i % CARD_ICONS.length]}
          <div class="p-4 border rounded-lg {styles.bg}">
            <div class="flex items-center gap-2 mb-2">
              <Icon class="w-4 h-4 {styles.icon}" />
              <span class="font-medium {styles.title}">{rec.title}</span>
            </div>
            <p class="text-sm {styles.text}">{rec.content}</p>
          </div>
        {/each}
        {#if loading}
          <p class="text-xs text-muted-foreground flex items-center gap-1">
            <RefreshCw class="w-3 h-3 animate-spin" />
            Updating…
          </p>
        {/if}
      {:else}
        <div class="p-4 border rounded-lg bg-muted/30">
          <p class="text-sm text-muted-foreground">
            Join champion select or enter a game to receive AI coaching tips.
          </p>
        </div>
      {/if}
    </div>
  </CardContent>
</Card>
