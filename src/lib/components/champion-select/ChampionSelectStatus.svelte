<script lang="ts">
  import { Clock, Users, Shield, Sword, Target, AlertCircle } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { 
    wsChampionSelect,
    wsChampionSelectPhase,
    wsIsChampionSelectActive
  } from '$lib/stores/websocketStore';
  import { champSelectTimerSec } from '$lib/utils/championSelectParser';

  let timeRemaining = $state(0);

  $effect(() => {
    const timer = $wsChampionSelect?.timer;
    if (!$wsIsChampionSelectActive || !timer) {
      timeRemaining = 0;
      return;
    }

    const tick = () => {
      timeRemaining = champSelectTimerSec(timer);
    };
    tick();
    const intervalId = setInterval(tick, 100);
    return () => clearInterval(intervalId);
  });

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function getPhaseIcon(phase: string) {
    switch (phase) {
      case 'BAN':
      case 'BAN_PICK':
        return Shield;
      case 'PICK':
        return Sword;
      case 'FINALIZATION':
        return Target;
      case 'GAME_STARTING':
        return Sword;
      case 'PLANNING':
        return Users;
      case 'READY_CHECK':
        return AlertCircle;
      default:
        return Clock;
    }
  }
</script>

{#if $wsIsChampionSelectActive}
  <Card class="bg-gradient-to-br from-card/50 to-card/30 backdrop-blur-sm border-border/50 shadow-lg">
    <CardHeader class="pb-4">
      <CardTitle class="flex items-center gap-3 text-lg">
        <div class="p-2 rounded-lg bg-primary/10">
          {#if $wsChampionSelectPhase}
            {@const IconComponent = getPhaseIcon($wsChampionSelectPhase)}
            <IconComponent class="w-5 h-5 text-primary" />
          {:else}
            <Clock class="w-5 h-5 text-primary" />
          {/if}
        </div>
        <div class="flex-1">
          <div class="font-bold">Champion Select</div>
          <div class="text-sm text-muted-foreground font-normal">
            {$wsChampionSelectPhase}
          </div>
        </div>
        <Badge variant="outline" class="text-blue-600">
          {$wsChampionSelectPhase}
        </Badge>
      </CardTitle>
    </CardHeader>
    <CardContent>
      <div class="space-y-4">
        <!-- Timer -->
        {#if timeRemaining > 0}
          <div class="flex items-center justify-center p-4 bg-muted/50 rounded-lg">
            <div class="text-center">
              <div class="text-3xl font-mono font-bold text-primary">
                {formatTime(timeRemaining)}
              </div>
              <div class="text-sm text-muted-foreground mt-1">
                Time Remaining
              </div>
            </div>
          </div>
        {/if}

        <!-- Phase Information -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div class="p-3 bg-muted/30 rounded-lg">
            <div class="flex items-center gap-2 mb-2">
              <Users class="w-4 h-4 text-blue-500" />
              <span class="text-sm font-medium">Phase</span>
            </div>
            <div class="text-lg font-semibold text-blue-600">
              {$wsChampionSelectPhase}
            </div>
          </div>
          
          <div class="p-3 bg-muted/30 rounded-lg">
            <div class="flex items-center gap-2 mb-2">
              <Clock class="w-4 h-4 text-green-500" />
              <span class="text-sm font-medium">Status</span>
            </div>
            <div class="text-lg font-semibold text-green-600">
              Active
            </div>
          </div>
        </div>

        <!-- Quick Actions -->
        <div class="flex flex-wrap gap-2">
          <Badge variant="secondary" class="text-xs">
            <Shield class="w-3 h-3 mr-1" />
            Draft Phase
          </Badge>
          <Badge variant="secondary" class="text-xs">
            <Target class="w-3 h-3 mr-1" />
            Team Building
          </Badge>
          <Badge variant="secondary" class="text-xs">
            <Sword class="w-3 h-3 mr-1" />
            Strategy
          </Badge>
        </div>
      </div>
    </CardContent>
  </Card>
{:else}
  <Card class="bg-gradient-to-br from-card/50 to-card/30 backdrop-blur-sm border-border/50 shadow-lg opacity-60">
    <CardHeader class="pb-4">
      <CardTitle class="flex items-center gap-3 text-lg">
        <div class="p-2 rounded-lg bg-muted/50">
          <Clock class="w-5 h-5 text-muted-foreground" />
        </div>
        <div class="flex-1">
          <div class="font-bold text-muted-foreground">Champion Select</div>
          <div class="text-sm text-muted-foreground font-normal">
            Not in champion select
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
          <Clock class="w-8 h-8 text-muted-foreground/50" />
        </div>
        <p class="text-sm">Waiting for champion select to begin...</p>
      </div>
    </CardContent>
  </Card>
{/if}
