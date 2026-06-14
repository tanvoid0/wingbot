<script lang="ts">
  import { Zap, Shield, Sword, Target, AlertCircle, CheckCircle, Info, Star } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Progress } from '$lib/components/ui/progress';
  import { 
    allPlayersRunes,
    runeAnalysisData,
    currentPlayerRunes,
    runeAnalysisActions,
    runeAnalysisSummary,
    runeInsights
  } from '$lib/stores/runeAnalysisStore';
  import { onMount } from 'svelte';

  // Local state
  let selectedTab = $state<'overview' | 'team' | 'recommendations'>('overview');
  let showInsights = $state(true);

  // Derived values from stores
  let summary = $derived($runeAnalysisSummary);
  let insights = $derived($runeInsights);

  function getInsightIcon(type: string) {
    switch (type) {
      case 'success': return CheckCircle;
      case 'error': return AlertCircle;
      case 'warning': return AlertCircle;
      case 'info': return Info;
      default: return Info;
    }
  }

  function getInsightColor(type: string) {
    switch (type) {
      case 'success': return 'text-green-600';
      case 'error': return 'text-red-600';
      case 'warning': return 'text-yellow-600';
      case 'info': return 'text-blue-600';
      default: return 'text-gray-600';
    }
  }

  function getQualityColor(quality: string) {
    switch (quality) {
      case 'excellent': return 'text-green-600';
      case 'good': return 'text-blue-600';
      case 'average': return 'text-yellow-600';
      case 'poor': return 'text-orange-600';
      case 'critical': return 'text-red-600';
      default: return 'text-gray-600';
    }
  }
</script>

{#if $currentPlayerRunes || Object.keys($runeAnalysisData).length > 0}
  <Card class="bg-gradient-to-br from-card/50 to-card/30 backdrop-blur-sm border-border/50 shadow-lg">
    <CardHeader class="pb-4">
      <CardTitle class="flex items-center gap-3 text-lg">
        <div class="p-2 rounded-lg bg-primary/10">
          <Zap class="w-5 h-5 text-primary" />
        </div>
        <div class="flex-1">
          <div class="font-bold">Rune Analysis</div>
          <div class="text-sm text-muted-foreground font-normal">
            Real-time rune setup optimization
          </div>
        </div>
        <Badge variant="outline" class="text-xs">
          {summary.totalPlayers} Players
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
            onclick={() => selectedTab = 'team'}
            class="flex-1 px-3 py-2 text-sm font-medium rounded-md transition-colors
              {selectedTab === 'team' 
                ? 'bg-background text-foreground shadow-sm' 
                : 'text-muted-foreground hover:text-foreground'}"
          >
            Team
          </button>
          <button
            onclick={() => selectedTab = 'recommendations'}
            class="flex-1 px-3 py-2 text-sm font-medium rounded-md transition-colors
              {selectedTab === 'recommendations' 
                ? 'bg-background text-foreground shadow-sm' 
                : 'text-muted-foreground hover:text-foreground'}"
          >
            Tips
          </button>
        </div>

        <!-- Overview Tab -->
        {#if selectedTab === 'overview'}
          <div class="space-y-6">
            <!-- Current Player Rune Setup -->
            {#if summary.currentPlayer}
              <div class="space-y-4">
                <h3 class="text-lg font-semibold flex items-center gap-2">
                  <Star class="w-5 h-5 text-muted-foreground" />
                  Your Rune Setup
                </h3>

                <!-- Quality Score -->
                <div class="p-4 bg-muted/30 rounded-lg">
                  <div class="flex items-center justify-between mb-2">
                    <span class="text-sm font-medium">Setup Quality</span>
                    <Badge variant="outline" class={getQualityColor(summary.currentPlayer.quality)}>
                      {summary.currentPlayer.quality}
                    </Badge>
                  </div>
                  <Progress value={summary.currentPlayer.score || 0} class="h-2" />
                  <div class="text-xs text-muted-foreground mt-1">
                    Score: {summary.currentPlayer.score || 0}/100
                  </div>
                </div>

                <!-- Rune Trees -->
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div class="p-3 bg-muted/30 rounded-lg">
                    <div class="flex items-center gap-2 mb-2">
                      <span class="text-lg">{runeAnalysisActions.getRuneTreeIcon(summary.currentPlayer.primaryTree)}</span>
                      <span class="text-sm font-medium">Primary Tree</span>
                    </div>
                    <div class="text-lg font-semibold {runeAnalysisActions.getRuneTreeColor(summary.currentPlayer.primaryTree)}">
                      {summary.currentPlayer.primaryTree}
                    </div>
                  </div>

                  <div class="p-3 bg-muted/30 rounded-lg">
                    <div class="flex items-center gap-2 mb-2">
                      <span class="text-lg">{runeAnalysisActions.getRuneTreeIcon(summary.currentPlayer.secondaryTree)}</span>
                      <span class="text-sm font-medium">Secondary Tree</span>
                    </div>
                    <div class="text-lg font-semibold {runeAnalysisActions.getRuneTreeColor(summary.currentPlayer.secondaryTree)}">
                      {summary.currentPlayer.secondaryTree}
                    </div>
                  </div>
                </div>

                <!-- Rune Count -->
                <div class="p-3 bg-muted/30 rounded-lg">
                  <div class="flex items-center gap-2 mb-2">
                    <Target class="w-4 h-4 text-purple-500" />
                    <span class="text-sm font-medium">Total Runes</span>
                  </div>
                  <div class="text-2xl font-bold text-purple-600">
                    {summary.currentPlayer.totalRunes}
                  </div>
                  <div class="text-xs text-muted-foreground mt-1">
                    {summary.currentPlayer.totalRunes >= 6 ? 'Complete setup' : 'Incomplete setup'}
                  </div>
                </div>

                <!-- Issues -->
                {#if summary.currentPlayer.issues && summary.currentPlayer.issues.length > 0}
                  <div class="space-y-2">
                    <h4 class="text-sm font-medium text-red-600">Issues Found:</h4>
                    <div class="space-y-1">
                      {#each summary.currentPlayer.issues as issue}
                        <div class="flex items-center gap-2 text-sm text-red-600">
                          <AlertCircle class="w-4 h-4" />
                          {issue}
                        </div>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>
            {/if}

            <!-- Performance Insights -->
            {#if showInsights && insights.length > 0}
              <div class="space-y-2">
                <div class="flex items-center justify-between">
                  <h4 class="text-sm font-medium">Rune Insights:</h4>
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
                Show Rune Insights
              </button>
            {/if}
          </div>
        {/if}

        <!-- Team Tab -->
        {#if selectedTab === 'team'}
          <div class="space-y-4">
            <h4 class="text-sm font-medium">Team Rune Composition:</h4>
            {#if summary.teamComposition.totalPlayers > 0}
              <div class="space-y-3">
                <div class="p-3 bg-muted/30 rounded-lg">
                  <div class="flex items-center justify-between mb-2">
                    <span class="text-sm font-medium">Most Common Tree</span>
                    <Badge variant="outline">
                      {summary.teamComposition.mostCommon}
                    </Badge>
                  </div>
                  <div class="text-xs text-muted-foreground">
                    Used by {summary.teamComposition.composition[summary.teamComposition.mostCommon] || 0} players
                  </div>
                </div>

                <div class="grid grid-cols-2 gap-2">
                  {#each Object.entries(summary.teamComposition.composition) as [tree, count]}
                    <div class="p-2 bg-muted/30 rounded-lg text-center">
                      <div class="text-lg">{runeAnalysisActions.getRuneTreeIcon(tree)}</div>
                      <div class="text-sm font-medium {runeAnalysisActions.getRuneTreeColor(tree)}">
                        {tree}
                      </div>
                      <div class="text-xs text-muted-foreground">
                        {count} players
                      </div>
                    </div>
                  {/each}
                </div>

                <div class="p-3 bg-muted/30 rounded-lg">
                  <div class="flex items-center justify-between">
                    <span class="text-sm font-medium">Diversity Score</span>
                    <Badge variant="outline">
                      {summary.teamComposition.diversity}/5
                    </Badge>
                  </div>
                  <div class="text-xs text-muted-foreground mt-1">
                    {summary.teamComposition.diversity >= 4 ? 'High diversity' : 
                     summary.teamComposition.diversity >= 3 ? 'Good diversity' : 
                     'Low diversity'} in team rune choices
                  </div>
                </div>
              </div>
            {:else}
              <div class="text-center py-8 text-muted-foreground">
                <div class="w-8 h-8 mx-auto mb-2 opacity-50">👥</div>
                <p class="text-sm">No team rune data available</p>
              </div>
            {/if}
          </div>
        {/if}

        <!-- Recommendations Tab -->
        {#if selectedTab === 'recommendations'}
          <div class="space-y-4">
            <h4 class="text-sm font-medium">Rune Optimization Tips:</h4>
            <div class="space-y-3">
              <div class="p-3 bg-muted/30 rounded-lg">
                <div class="flex items-start gap-2">
                  <Sword class="w-4 h-4 mt-0.5 text-blue-500" />
                  <div>
                    <div class="text-sm font-medium">Primary Tree Selection</div>
                    <div class="text-xs text-muted-foreground mt-1">
                      Choose your primary tree based on your champion's main damage type and playstyle.
                    </div>
                  </div>
                </div>
              </div>

              <div class="p-3 bg-muted/30 rounded-lg">
                <div class="flex items-start gap-2">
                  <Shield class="w-4 h-4 mt-0.5 text-green-500" />
                  <div>
                    <div class="text-sm font-medium">Secondary Tree Synergy</div>
                    <div class="text-xs text-muted-foreground mt-1">
                      Select a different secondary tree to maximize stat bonuses and rune effects.
                    </div>
                  </div>
                </div>
              </div>

              <div class="p-3 bg-muted/30 rounded-lg">
                <div class="flex items-start gap-2">
                  <Target class="w-4 h-4 mt-0.5 text-purple-500" />
                  <div>
                    <div class="text-sm font-medium">Keystone Importance</div>
                    <div class="text-xs text-muted-foreground mt-1">
                      Your keystone rune is the most important choice and should match your champion's core playstyle.
                    </div>
                  </div>
                </div>
              </div>

              <div class="p-3 bg-muted/30 rounded-lg">
                <div class="flex items-start gap-2">
                  <Zap class="w-4 h-4 mt-0.5 text-yellow-500" />
                  <div>
                    <div class="text-sm font-medium">Adapt to Matchups</div>
                    <div class="text-xs text-muted-foreground mt-1">
                      Consider adjusting your runes based on the enemy team composition and your lane opponent.
                    </div>
                  </div>
                </div>
              </div>
            </div>
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
          <Zap class="w-5 h-5 text-muted-foreground" />
        </div>
        <div class="flex-1">
          <div class="font-bold text-muted-foreground">Rune Analysis</div>
          <div class="text-sm text-muted-foreground font-normal">
            No rune data available
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
          <Zap class="w-8 h-8 text-muted-foreground/50" />
        </div>
        <p class="text-sm">Waiting for rune data...</p>
      </div>
    </CardContent>
  </Card>
{/if}
