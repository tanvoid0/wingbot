<script lang="ts">
  import { Users, Clock, CheckCircle, AlertCircle, Play, X, Info, Target, Crown } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { Progress } from '$lib/components/ui/progress';
  import { 
    lobbyData,
    lobbyAnalysis,
    readyCheckData,
    lobbyStatus,
    lobbyActions,
    lobbySummary,
    lobbyInsights,
    lobbyMembers
  } from '$lib/stores/lobbyStore';
  import { onMount } from 'svelte';

  // Local state
  let selectedTab = $state<'overview' | 'members' | 'ready'>('overview');
  let showInsights = $state(true);
  let isAcceptingReady = $state(false);
  let isDecliningReady = $state(false);

  // Derived values from stores
  let summary = $derived($lobbySummary);
  let insights = $derived($lobbyInsights);
  let members = $derived($lobbyMembers);

  function getInsightIcon(type: string) {
    switch (type) {
      case 'success': return CheckCircle;
      case 'warning': return AlertCircle;
      case 'info': return Info;
      default: return Info;
    }
  }

  function getInsightColor(type: string) {
    switch (type) {
      case 'success': return 'text-green-600';
      case 'warning': return 'text-yellow-600';
      case 'info': return 'text-blue-600';
      default: return 'text-gray-600';
    }
  }

  async function acceptReadyCheck() {
    isAcceptingReady = true;
    try {
      // This would call the WebSocket action to accept ready check
      // For now, we'll just simulate the action
      console.log('Accepting ready check...');
      await new Promise(resolve => setTimeout(resolve, 1000));
    } finally {
      isAcceptingReady = false;
    }
  }

  async function declineReadyCheck() {
    isDecliningReady = true;
    try {
      // This would call the WebSocket action to decline ready check
      // For now, we'll just simulate the action
      console.log('Declining ready check...');
      await new Promise(resolve => setTimeout(resolve, 1000));
    } finally {
      isDecliningReady = false;
    }
  }

  function formatTime(seconds: number): string {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes.toString().padStart(2, '0')}:${remainingSeconds.toString().padStart(2, '0')}`;
  }
</script>

{#if summary.isInLobby}
  <Card class="bg-gradient-to-br from-card/50 to-card/30 backdrop-blur-sm border-border/50 shadow-lg">
    <CardHeader class="pb-4">
      <CardTitle class="flex items-center gap-3 text-lg">
        <div class="p-2 rounded-lg bg-primary/10">
          <Users class="w-5 h-5 text-primary" />
        </div>
        <div class="flex-1">
          <div class="font-bold">Lobby Management</div>
          <div class="text-sm text-muted-foreground font-normal">
            {summary.queueName} • {summary.gameMode}
          </div>
        </div>
        <Badge variant="outline" class="text-xs">
          {summary.memberCount}/5 Players
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
            onclick={() => selectedTab = 'members'}
            class="flex-1 px-3 py-2 text-sm font-medium rounded-md transition-colors
              {selectedTab === 'members' 
                ? 'bg-background text-foreground shadow-sm' 
                : 'text-muted-foreground hover:text-foreground'}"
          >
            Members
          </button>
          <button
            onclick={() => selectedTab = 'ready'}
            class="flex-1 px-3 py-2 text-sm font-medium rounded-md transition-colors
              {selectedTab === 'ready' 
                ? 'bg-background text-foreground shadow-sm' 
                : 'text-muted-foreground hover:text-foreground'}"
          >
            Ready Check
          </button>
        </div>

        <!-- Overview Tab -->
        {#if selectedTab === 'overview'}
          <div class="space-y-6">
            <!-- Lobby Status -->
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div class="p-3 bg-muted/30 rounded-lg">
                <div class="flex items-center gap-2 mb-2">
                  <Users class="w-4 h-4 text-blue-500" />
                  <span class="text-sm font-medium">Team Size</span>
                </div>
                <div class="text-2xl font-bold text-blue-600">
                  {summary.memberCount}/5
                </div>
                <div class="text-xs text-muted-foreground mt-1">
                  {summary.memberCount >= 5 ? 'Full team' : `${5 - summary.memberCount} needed`}
                </div>
              </div>

              <div class="p-3 bg-muted/30 rounded-lg">
                <div class="flex items-center gap-2 mb-2">
                  <CheckCircle class="w-4 h-4 text-green-500" />
                  <span class="text-sm font-medium">Readiness</span>
                </div>
                <div class="text-2xl font-bold text-green-600">
                  {summary.readinessPercentage}%
                </div>
                <Progress value={summary.readinessPercentage} class="h-2 mt-2" />
              </div>

              <div class="p-3 bg-muted/30 rounded-lg">
                <div class="flex items-center gap-2 mb-2">
                  <Target class="w-4 h-4 text-purple-500" />
                  <span class="text-sm font-medium">Queue</span>
                </div>
                <div class="text-lg font-semibold text-purple-600">
                  {summary.queueName}
                </div>
                <div class="text-xs text-muted-foreground mt-1">
                  {summary.gameMode}
                </div>
              </div>
            </div>

            <!-- Lobby Insights -->
            {#if showInsights && insights.length > 0}
              <div class="space-y-2">
                <div class="flex items-center justify-between">
                  <h4 class="text-sm font-medium">Lobby Insights:</h4>
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
                Show Lobby Insights
              </button>
            {/if}
          </div>
        {/if}

        <!-- Members Tab -->
        {#if selectedTab === 'members'}
          <div class="space-y-4">
            <h4 class="text-sm font-medium">Team Members:</h4>
            {#if members.length > 0}
              <div class="space-y-3">
                {#each members as member}
                  <div class="p-3 bg-muted/30 rounded-lg">
                    <div class="flex items-center justify-between">
                      <div class="flex items-center gap-3">
                        <div class="text-lg">{member.roleIcon}</div>
                        <div>
                          <div class="font-medium">{member.summonerName || 'Unknown'}</div>
                          <div class="text-xs text-muted-foreground">
                            {member.roleName} • Champion {member.championId || 'Not Selected'}
                          </div>
                        </div>
                      </div>
                      <div class="flex items-center gap-2">
                        {#if member.isLeader}
                          <Crown class="w-4 h-4 text-yellow-500" />
                        {/if}
                        <Badge variant="outline" class={member.statusColor}>
                          {member.statusText}
                        </Badge>
                      </div>
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="text-center py-8 text-muted-foreground">
                <Users class="w-8 h-8 mx-auto mb-2 opacity-50" />
                <p class="text-sm">No team members found</p>
              </div>
            {/if}
          </div>
        {/if}

        <!-- Ready Check Tab -->
        {#if selectedTab === 'ready'}
          <div class="space-y-4">
            {#if summary.readyCheckState !== 'None'}
              <div class="p-4 bg-muted/30 rounded-lg">
                <div class="flex items-center justify-between mb-4">
                  <div class="flex items-center gap-2">
                    <Clock class="w-5 h-5 text-primary" />
                    <span class="font-medium">Ready Check Status</span>
                  </div>
                  <Badge variant="outline" class={lobbyActions.getReadyCheckStateColor(summary.readyCheckState)}>
                    {lobbyActions.getReadyCheckStateText(summary.readyCheckState)}
                  </Badge>
                </div>

                {#if summary.readyCheckTimer > 0}
                  <div class="text-center mb-4">
                    <div class="text-3xl font-bold text-primary">
                      {formatTime(summary.readyCheckTimer)}
                    </div>
                    <div class="text-sm text-muted-foreground">Time Remaining</div>
                  </div>
                {/if}

                <div class="flex gap-2 justify-center">
                  <Button
                    onclick={acceptReadyCheck}
                    disabled={isAcceptingReady || isDecliningReady}
                    class="flex items-center gap-2"
                  >
                    <Play class="w-4 h-4" />
                    {isAcceptingReady ? 'Accepting...' : 'Accept'}
                  </Button>
                  <Button
                    variant="outline"
                    onclick={declineReadyCheck}
                    disabled={isAcceptingReady || isDecliningReady}
                    class="flex items-center gap-2"
                  >
                    <X class="w-4 h-4" />
                    {isDecliningReady ? 'Declining...' : 'Decline'}
                  </Button>
                </div>
              </div>
            {:else}
              <div class="text-center py-8 text-muted-foreground">
                <div class="p-4 rounded-full bg-muted/50 w-fit mx-auto mb-4">
                  <Clock class="w-8 h-8 text-muted-foreground/50" />
                </div>
                <p class="text-sm">No ready check active</p>
                <p class="text-xs text-muted-foreground mt-1">
                  Ready checks will appear here when they become available
                </p>
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
          <Users class="w-5 h-5 text-muted-foreground" />
        </div>
        <div class="flex-1">
          <div class="font-bold text-muted-foreground">Lobby Management</div>
          <div class="text-sm text-muted-foreground font-normal">
            No lobby data available
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
          <Users class="w-8 h-8 text-muted-foreground/50" />
        </div>
        <p class="text-sm">Waiting for lobby data...</p>
        <p class="text-xs text-muted-foreground mt-1">
          Join a lobby or queue to see management options
        </p>
      </div>
    </CardContent>
  </Card>
{/if}
