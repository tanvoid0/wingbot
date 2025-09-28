<script lang="ts">
  import { 
    Clock, 
    Sword, 
    Target, 
    Zap, 
    Shield, 
    Gamepad2, 
    AlertCircle,
    Trophy,
    Flame,
    Crown,
    Skull,
    Building2,
    Mountain
  } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { gameData, liveGameActions } from '$lib/stores/liveGameStore';

  // Event type configurations for better visual hierarchy
  const eventConfigs = {
    'ChampionKill': {
      icon: Sword,
      iconColor: 'text-red-500',
      killerColor: 'text-red-600 bg-red-50 dark:bg-red-950/20',
      victimColor: 'text-red-500 bg-red-50 dark:bg-red-950/20',
      bgColor: 'bg-red-50/30 dark:bg-red-950/10',
      borderColor: 'border-red-200 dark:border-red-800'
    },
    'FirstBlood': {
      icon: Skull,
      iconColor: 'text-red-600',
      killerColor: 'text-red-700 bg-red-100 dark:bg-red-900/30',
      victimColor: 'text-red-600 bg-red-50 dark:bg-red-950/20',
      bgColor: 'bg-red-100/50 dark:bg-red-950/20',
      borderColor: 'border-red-300 dark:border-red-700'
    },
    'TurretKilled': {
      icon: Building2,
      iconColor: 'text-orange-500',
      killerColor: 'text-orange-600 bg-orange-50 dark:bg-orange-950/20',
      victimColor: 'text-orange-500',
      bgColor: 'bg-orange-50/30 dark:bg-orange-950/10',
      borderColor: 'border-orange-200 dark:border-orange-800'
    },
    'FirstBrick': {
      icon: Crown,
      iconColor: 'text-yellow-500',
      killerColor: 'text-yellow-600 bg-yellow-50 dark:bg-yellow-950/20',
      victimColor: 'text-yellow-500',
      bgColor: 'bg-yellow-50/30 dark:bg-yellow-950/10',
      borderColor: 'border-yellow-200 dark:border-yellow-800'
    },
    'Multikill': {
      icon: Flame,
      iconColor: 'text-purple-500',
      killerColor: 'text-purple-600 bg-purple-50 dark:bg-purple-950/20',
      victimColor: 'text-purple-500',
      bgColor: 'bg-purple-50/30 dark:bg-purple-950/10',
      borderColor: 'border-purple-200 dark:border-purple-800'
    },
    'HordeKill': {
      icon: Mountain,
      iconColor: 'text-green-500',
      killerColor: 'text-green-600 bg-green-50 dark:bg-green-950/20',
      victimColor: 'text-green-500',
      bgColor: 'bg-green-50/30 dark:bg-green-950/10',
      borderColor: 'border-green-200 dark:border-green-800'
    },
    'GameStart': {
      icon: Gamepad2,
      iconColor: 'text-blue-500',
      killerColor: 'text-blue-600',
      victimColor: 'text-blue-500',
      bgColor: 'bg-blue-50/30 dark:bg-blue-950/10',
      borderColor: 'border-blue-200 dark:border-blue-800'
    },
    'MonsSpawning': {
      icon: Target,
      iconColor: 'text-indigo-500',
      killerColor: 'text-indigo-600',
      victimColor: 'text-indigo-500',
      bgColor: 'bg-indigo-50/30 dark:bg-indigo-950/10',
      borderColor: 'border-indigo-200 dark:border-indigo-800'
    }
  };

  function getEventConfig(eventName: string) {
    return eventConfigs[eventName as keyof typeof eventConfigs] || {
      icon: AlertCircle,
      iconColor: 'text-gray-500',
      killerColor: 'text-gray-600',
      victimColor: 'text-gray-500',
      bgColor: 'bg-gray-50/30 dark:bg-gray-950/10',
      borderColor: 'border-gray-200 dark:border-gray-800'
    };
  }
</script>

{#if $gameData && typeof $gameData === 'object' && 'events' in $gameData}
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Clock class="w-5 h-5" />
        Game Events Timeline
        <Badge variant="outline" class="text-xs ml-auto">
          Latest First
        </Badge>
      </CardTitle>
    </CardHeader>
    <CardContent>
      {#if ($gameData as any).events?.Events && ($gameData as any).events.Events.length > 0}
        <div class="space-y-3 max-h-96 overflow-y-auto pr-2">
          {#each ($gameData as any).events.Events.slice(-20).reverse() as event, index}
            {@const config = getEventConfig(event.EventName)}
            {@const IconComponent = config.icon}
            <div class="relative">
              <!-- Timeline connector line -->
              {#if index < ($gameData as any).events.Events.slice(-20).length - 1}
                <div class="absolute left-6 top-12 w-0.5 h-6 bg-border"></div>
              {/if}
              
              <div class="flex items-start gap-4 p-4 rounded-xl border-2 {config.borderColor} {config.bgColor} hover:shadow-md transition-all duration-300 group">
                <!-- Timeline dot with icon -->
                <div class="flex-shrink-0 w-12 h-12 rounded-full bg-background border-2 {config.borderColor} flex items-center justify-center shadow-sm group-hover:scale-110 transition-all duration-300">
                  <IconComponent class="w-5 h-5 {config.iconColor} {
                    event.EventName === 'ChampionKill' ? 'animate-pulse' :
                    event.EventName === 'FirstBlood' ? 'animate-bounce animate-pulse' :
                    event.EventName === 'TurretKilled' ? 'animate-pulse' :
                    event.EventName === 'FirstBrick' ? 'animate-bounce' :
                    event.EventName === 'Multikill' ? 'animate-pulse animate-bounce' :
                    event.EventName === 'HordeKill' ? 'animate-pulse' :
                    event.EventName === 'GameStart' ? 'animate-pulse' :
                    event.EventName === 'MonsSpawning' ? 'animate-pulse' :
                    ''
                  }" />
                </div>
                
                <!-- Event content -->
                <div class="flex-1 min-w-0">
                  <!-- Timestamp -->
                  <div class="flex items-center gap-2 mb-2">
                    <div 
                      class="text-xs font-mono text-muted-foreground cursor-help" 
                      title="Exact time: {liveGameActions.formatGameTime(event.EventTime || 0)}"
                    >
                      {liveGameActions.formatEventTimeWithContext(event.EventTime || 0)}
                    </div>
                    {#if index === 0}
                      <Badge variant="secondary" class="text-xs bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200 animate-pulse">
                        Latest
                      </Badge>
                    {/if}
                  </div>
                  
                  <!-- Event description -->
                  <div class="space-y-1">
                    {#if event.EventName === 'ChampionKill'}
                      <div class="flex items-center gap-2 flex-wrap">
                        <span class="px-2 py-1 rounded-md text-sm font-semibold {config.killerColor} animate-pulse">
                          {event.KillerName}
                        </span>
                        <span class="text-muted-foreground text-sm">eliminated</span>
                        <span class="px-2 py-1 rounded-md text-sm font-semibold {config.victimColor}">
                          {event.VictimName}
                        </span>
                        {#if event.Assisters && event.Assisters.length > 0}
                          <div class="text-xs text-muted-foreground bg-muted/50 px-2 py-1 rounded">
                            assisted by {event.Assisters.join(', ')}
                          </div>
                        {/if}
                      </div>
                    {:else if event.EventName === 'FirstBlood'}
                      <div class="flex items-center gap-2 flex-wrap">
                        <Badge variant="destructive" class="text-xs font-bold animate-pulse animate-bounce">
                          FIRST BLOOD
                        </Badge>
                        <span class="px-2 py-1 rounded-md text-sm font-semibold {config.killerColor} animate-bounce">
                          {event.KillerName}
                        </span>
                        <span class="text-muted-foreground text-sm">eliminated</span>
                        <span class="px-2 py-1 rounded-md text-sm font-semibold {config.victimColor}">
                          {event.VictimName}
                        </span>
                      </div>
                    {:else if event.EventName === 'TurretKilled'}
                      <div class="flex items-center gap-2 flex-wrap">
                        <span class="px-2 py-1 rounded-md text-sm font-semibold {config.killerColor} animate-pulse">
                          {event.KillerName}
                        </span>
                        <span class="text-muted-foreground text-sm">destroyed</span>
                        <span class="text-orange-600 font-semibold text-sm animate-bounce">🏰 Turret</span>
                      </div>
                    {:else if event.EventName === 'FirstBrick'}
                      <div class="flex items-center gap-2 flex-wrap">
                        <Badge variant="secondary" class="text-xs font-bold bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200 animate-bounce">
                          FIRST BRICK
                        </Badge>
                        <span class="px-2 py-1 rounded-md text-sm font-semibold {config.killerColor} animate-pulse">
                          {event.KillerName}
                        </span>
                      </div>
                    {:else if event.EventName === 'Multikill'}
                      <div class="flex items-center gap-2 flex-wrap">
                        <Badge variant="secondary" class="text-xs font-bold bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200 animate-pulse animate-bounce">
                          {event.KillStreak} KILL STREAK
                        </Badge>
                        <span class="px-2 py-1 rounded-md text-sm font-semibold {config.killerColor} animate-bounce">
                          {event.KillerName}
                        </span>
                      </div>
                    {:else if event.EventName === 'HordeKill'}
                      <div class="flex items-center gap-2 flex-wrap">
                        <span class="px-2 py-1 rounded-md text-sm font-semibold {config.killerColor} animate-pulse">
                          {event.KillerName}
                        </span>
                        <span class="text-muted-foreground text-sm">slayed</span>
                        <span class="text-green-600 font-semibold text-sm animate-bounce">🐉 Baron Nashor</span>
                        {#if event.Stolen}
                          <Badge variant="destructive" class="text-xs font-bold animate-pulse animate-bounce">
                            STOLEN!
                          </Badge>
                        {/if}
                      </div>
                    {:else if event.EventName === 'GameStart'}
                      <div class="flex items-center gap-2">
                        <span class="text-blue-600 font-semibold text-sm animate-pulse">🎮 Game Started</span>
                      </div>
                    {:else if event.EventName === 'MonsSpawning'}
                      <div class="flex items-center gap-2">
                        <span class="text-indigo-600 font-semibold text-sm animate-pulse">👹 Monsters Spawning</span>
                      </div>
                    {:else}
                      <div class="flex items-center gap-2">
                        <span class="text-muted-foreground text-sm">{event.EventName}</span>
                      </div>
                    {/if}
                  </div>
                </div>
              </div>
            </div>
          {/each}
        </div>
      {:else}
        <div class="text-center py-8 text-muted-foreground">
          <Clock class="w-12 h-12 mx-auto mb-4 opacity-50" />
          <p>No game events available</p>
        </div>
      {/if}
    </CardContent>
  </Card>
{/if}
