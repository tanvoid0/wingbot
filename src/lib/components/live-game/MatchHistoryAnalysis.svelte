<script lang="ts">
  import { Trophy, TrendingUp, Target, Users, Award, AlertCircle, CheckCircle, Clock } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Progress } from '$lib/components/ui/progress';
  import { 
    currentPlayerStats,
    championPerformance,
    recentMatches,
    matchHistoryActions,
    matchHistorySummary,
    playerInsights
  } from '$lib/stores/matchHistoryStore';
  import { onMount } from 'svelte';

  // Local state
  let selectedTab = $state<'overview' | 'champions' | 'recent'>('overview');
  let showInsights = $state(true);

  // Derived values from stores
  let summary = $derived($matchHistorySummary);
  let insights = $derived($playerInsights);

  function getInsightIcon(type: string) {
    switch (type) {
      case 'positive': return CheckCircle;
      case 'negative': return AlertCircle;
      case 'warning': return AlertCircle;
      default: return Clock;
    }
  }

  function getInsightColor(type: string) {
    switch (type) {
      case 'positive': return 'text-green-600';
      case 'negative': return 'text-red-600';
      case 'warning': return 'text-yellow-600';
      default: return 'text-gray-600';
    }
  }
</script>

{#if $currentPlayerStats}
  <Card class="bg-gradient-to-br from-card/50 to-card/30 backdrop-blur-sm border-border/50 shadow-lg">
    <CardHeader class="pb-4">
      <CardTitle class="flex items-center gap-3 text-lg">
        <div class="p-2 rounded-lg bg-primary/10">
          <Trophy class="w-5 h-5 text-primary" />
        </div>
        <div class="flex-1">
          <div class="font-bold">Match History Analysis</div>
          <div class="text-sm text-muted-foreground font-normal">
            Performance insights and statistics
          </div>
        </div>
        <Badge variant="outline" class="text-xs">
          {summary.totalMatches} Matches
        </Badge>
      </CardTitle>
    </CardHeader>
    <CardContent>
      <div class="space-y-6">
        <!-- Tab Navigation -->
        <div class="flex space-x-1 bg-muted/30 p-1 rounded-lg">
          <button
            onclick={() => selectedTab = 'overview'}
            class="flex-1 px-3 py-2 text-sm font-medium rounded-md transition-colors
              {selectedTab === 'overview' 
                ? 'bg-background text-foreground shadow-sm' 
                : 'text-muted-foreground hover:text-foreground'}"
          >
            Overview
          </button>
          <button
            onclick={() => selectedTab = 'champions'}
            class="flex-1 px-3 py-2 text-sm font-medium rounded-md transition-colors
              {selectedTab === 'champions' 
                ? 'bg-background text-foreground shadow-sm' 
                : 'text-muted-foreground hover:text-foreground'}"
          >
            Champions
          </button>
          <button
            onclick={() => selectedTab = 'recent'}
            class="flex-1 px-3 py-2 text-sm font-medium rounded-md transition-colors
              {selectedTab === 'recent' 
                ? 'bg-background text-foreground shadow-sm' 
                : 'text-muted-foreground hover:text-foreground'}"
          >
            Recent
          </button>
        </div>

        <!-- Overview Tab -->
        {#if selectedTab === 'overview'}
          <div class="space-y-6">
            <!-- Key Statistics -->
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div class="p-3 bg-muted/30 rounded-lg">
                <div class="flex items-center gap-2 mb-2">
                  <Trophy class="w-4 h-4 text-yellow-500" />
                  <span class="text-sm font-medium">Win Rate</span>
                </div>
                <div class="text-2xl font-bold {matchHistoryActions.getWinRateColor(summary.winRate)}">
                  {summary.winRate}%
                </div>
                <Progress value={summary.winRate} class="h-2 mt-2" />
              </div>
              
              <div class="p-3 bg-muted/30 rounded-lg">
                <div class="flex items-center gap-2 mb-2">
                  <Target class="w-4 h-4 text-red-500" />
                  <span class="text-sm font-medium">Avg KDA</span>
                </div>
                <div class="text-2xl font-bold {matchHistoryActions.getKDAColor(summary.avgKDA)}">
                  {summary.avgKDA}
                </div>
                <div class="text-xs text-muted-foreground mt-1">
                  {matchHistoryActions.formatKDA(
                    $currentPlayerStats.avg_kills || 0,
                    $currentPlayerStats.avg_deaths || 0,
                    $currentPlayerStats.avg_assists || 0
                  )}
                </div>
              </div>
              
              <div class="p-3 bg-muted/30 rounded-lg">
                <div class="flex items-center gap-2 mb-2">
                  <Award class="w-4 h-4 text-purple-500" />
                  <span class="text-sm font-medium">Grade</span>
                </div>
                <div class="text-2xl font-bold {summary.performanceGrade.color}">
                  {summary.performanceGrade.grade}
                </div>
                <div class="text-xs text-muted-foreground mt-1">
                  Performance
                </div>
              </div>
            </div>

            <!-- Performance Insights -->
            {#if showInsights && insights.length > 0}
              <div class="space-y-2">
                <div class="flex items-center justify-between">
                  <h4 class="text-sm font-medium">Performance Insights:</h4>
                  <button
                    onclick={() => showInsights = !showInsights}
                    class="text-xs text-muted-foreground hover:text-foreground"
                  >
                    Hide
                  </button>
                </div>
                <div class="space-y-2">
                  {#each insights as insight}
                    {@const IconComponent = getInsightIcon(insight.type)}
                    <div class="p-3 bg-muted/30 rounded-lg">
                      <div class="flex items-start gap-2">
                        <IconComponent class="w-4 h-4 mt-0.5 {getInsightColor(insight.type)}" />
                        <div class="flex-1">
                          <div class="text-sm font-medium {getInsightColor(insight.type)}">
                            {insight.title}
                          </div>
                          <div class="text-xs text-muted-foreground mt-1">
                            {insight.message}
                          </div>
                        </div>
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            {:else if !showInsights}
              <button
                onclick={() => showInsights = true}
                class="text-xs text-muted-foreground hover:text-foreground"
              >
                Show Performance Insights
              </button>
            {/if}
          </div>
        {/if}

        <!-- Champions Tab -->
        {#if selectedTab === 'champions'}
          <div class="space-y-4">
            <h4 class="text-sm font-medium">Top Performing Champions:</h4>
            {#if summary.topChampions.length > 0}
              <div class="space-y-3">
                {#each summary.topChampions as champion}
                  <div class="p-3 bg-muted/30 rounded-lg">
                    <div class="flex items-center justify-between mb-2">
                      <div class="flex items-center gap-2">
                        <span class="font-medium">Champion {champion.championId}</span>
                        <Badge variant="outline" class="text-xs">
                          {champion.games} games
                        </Badge>
                      </div>
                      <div class="text-sm font-medium {matchHistoryActions.getWinRateColor(champion.winRate)}">
                        {champion.winRate.toFixed(1)}% WR
                      </div>
                    </div>
                    <div class="grid grid-cols-3 gap-2 text-xs text-muted-foreground">
                      <div>KDA: {champion.avgKDA.toFixed(2)}</div>
                      <div>Wins: {champion.wins}</div>
                      <div>Losses: {champion.games - champion.wins}</div>
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="text-center py-8 text-muted-foreground">
                <Users class="w-8 h-8 mx-auto mb-2 opacity-50" />
                <p class="text-sm">No champion data available</p>
              </div>
            {/if}
          </div>
        {/if}

        <!-- Recent Tab -->
        {#if selectedTab === 'recent'}
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <h4 class="text-sm font-medium">Recent Performance:</h4>
              <Badge variant="outline" class="text-xs {summary.recentTrend.color}">
                {summary.recentTrend.trend}
              </Badge>
            </div>
            {#if $recentMatches.length > 0}
              <div class="space-y-2">
                {#each $recentMatches.slice(0, 5) as match, index}
                  <div class="p-2 bg-muted/30 rounded-lg text-xs">
                    <div class="flex items-center justify-between">
                      <span>Match {index + 1}</span>
                      <span class="text-muted-foreground">
                        {new Date(match.gameCreation || Date.now()).toLocaleDateString()}
                      </span>
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="text-center py-8 text-muted-foreground">
                <Clock class="w-8 h-8 mx-auto mb-2 opacity-50" />
                <p class="text-sm">No recent matches available</p>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </CardContent>
  </Card>
{:else}
  <Card class="bg-gradient-to-br from-card/50 to-card/30 backdrop-blur-sm border-border/50 shadow-lg opacity-60">
    <CardHeader class="pb-4">
      <CardTitle class="flex items-center gap-3 text-lg">
        <div class="p-2 rounded-lg bg-muted/50">
          <Trophy class="w-5 h-5 text-muted-foreground" />
        </div>
        <div class="flex-1">
          <div class="font-bold text-muted-foreground">Match History Analysis</div>
          <div class="text-sm text-muted-foreground font-normal">
            No match data available
          </div>
        </div>
        <Badge variant="outline" class="text-muted-foreground">
          Inactive
        </Badge>
      </CardTitle>
    </CardHeader>
    <CardContent>
      <div class="text-center py-8 text-muted-foreground">
        <div class="p-4 rounded-full bg-muted/50 w-fit mx-auto mb-4">
          <Trophy class="w-8 h-8 text-muted-foreground/50" />
        </div>
        <p class="text-sm">Waiting for match history data...</p>
      </div>
    </CardContent>
  </Card>
{/if}
