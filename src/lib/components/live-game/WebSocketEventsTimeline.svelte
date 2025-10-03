<script lang="ts">
  import { Clock, Zap, Sword, Shield, Target } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { wsGameData, wsActions } from '$lib/stores/websocketStore';

  // Process events from game data
  $: events = ($wsGameData?.events?.Events || []) as any[];
  
  // Sort events by timestamp (most recent first)
  $: sortedEvents = [...events].sort((a: any, b: any) => (b.EventTime || 0) - (a.EventTime || 0));

  // Get event icon based on event type
  function getEventIcon(eventType: string) {
    switch (eventType) {
      case 'ChampionKill':
        return Sword;
      case 'Ace':
        return Target;
      case 'FirstBlood':
        return Zap;
      case 'TurretKilled':
        return Shield;
      default:
        return Clock;
    }
  }

  // Get event color based on event type
  function getEventColor(eventType: string) {
    switch (eventType) {
      case 'ChampionKill':
        return 'text-red-500';
      case 'Ace':
        return 'text-purple-500';
      case 'FirstBlood':
        return 'text-yellow-500';
      case 'TurretKilled':
        return 'text-blue-500';
      default:
        return 'text-gray-500';
    }
  }

  // Format event time
  function formatEventTime(eventTime: number) {
    return wsActions.formatGameTime(eventTime);
  }

  // Get event description
  function getEventDescription(event: any) {
    switch (event.EventName) {
      case 'ChampionKill':
        return `${event.KillerName} killed ${event.VictimName}`;
      case 'Ace':
        return `${event.AcingTeam} team got an ACE!`;
      case 'FirstBlood':
        return `${event.KillerName} got First Blood!`;
      case 'TurretKilled':
        return `${event.KillerName} destroyed a turret`;
      default:
        return event.EventName;
    }
  }
</script>

{#if $wsGameData?.events?.Events && $wsGameData.events.Events.length > 0}
<Card>
  <CardHeader>
    <CardTitle class="flex items-center gap-2">
      <Clock class="w-5 h-5" />
      Events Timeline
    </CardTitle>
  </CardHeader>
  <CardContent>
    {#if sortedEvents.length === 0}
      <div class="text-center py-8 text-muted-foreground">
        <Clock class="w-12 h-12 mx-auto mb-4 opacity-50" />
        <p>No events recorded yet</p>
      </div>
    {:else}
      <div class="space-y-3 max-h-96 sm:max-h-[28rem] lg:max-h-[32rem] overflow-y-auto">
        {#each sortedEvents.slice(0, 20) as event, index (index)}
          {@const EventIcon = getEventIcon(event.EventName)}
          <div class="flex items-center gap-3 p-3 border rounded-lg hover:bg-muted/50 transition-colors">
            <div class="flex-shrink-0">
              <EventIcon class="w-5 h-5 {getEventColor(event.EventName)}" />
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <Badge variant="outline" class="text-xs">
                  {formatEventTime(event.EventTime || 0)}
                </Badge>
                <span class="text-sm font-medium text-foreground">
                  {getEventDescription(event)}
                </span>
              </div>
              {#if event.EventName === 'ChampionKill' && event.Assisters?.length > 0}
                <div class="text-xs text-muted-foreground">
                  Assisted by: {event.Assisters.join(', ')}
                </div>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </CardContent>
</Card>
{/if}
