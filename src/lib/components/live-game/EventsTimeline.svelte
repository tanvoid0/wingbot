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
  import ImageWithFallback from '$lib/components/ui/ImageWithFallback.svelte';
  import { 
    getChampionImageUrlSync, 
    getChampionFallback 
  } from '$lib/services/imageService';
  import { gameData, liveGameActions } from '$lib/stores/liveGameStore';
  import { onMount } from 'svelte';
  
  // Local state for stable event list
  let displayedEvents = $state<any[]>([]);
  let lastEventCount = $state(0);
  let newEventIds = $state(new Set<string>());
  let currentGameSession = $state<string | null>(null);

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

  // Helper function to get champion info from player name
  function getChampionInfo(playerName: string) {
    if (!$gameData?.allPlayers) return null;
    
    const player = $gameData.allPlayers.find(p => 
      p.summonerName === playerName || 
      p.riotIdGameName === playerName ||
      p.championName === playerName
    );
    
    return player ? {
      championName: player.championName,
      team: player.team
    } : null;
  }

  // Function to generate a game session identifier
  function generateGameSessionId() {
    if (!$gameData) return null;
    
    // Create a session ID based on game state that changes when a new game starts
    const gameMode = $gameData.gameData?.gameMode || 'unknown';
    const mapName = $gameData.gameData?.mapName || 'unknown';
    const playerCount = $gameData.allPlayers?.length || 0;
    const firstPlayer = $gameData.allPlayers?.[0];
    const playerNames = $gameData.allPlayers?.map(p => p.summonerName).sort().join(',') || '';
    
    // Use a combination of factors that would change between games
    return `${gameMode}_${mapName}_${playerCount}_${playerNames.slice(0, 50)}`;
  }

  // Function to detect if we're in a new game session
  function isNewGameSession() {
    const newSessionId = generateGameSessionId();
    return newSessionId && newSessionId !== currentGameSession;
  }

  // Function to reset events for a new game
  function resetEventsForNewGame() {
    displayedEvents = [];
    lastEventCount = 0;
    newEventIds.clear();
    currentGameSession = generateGameSessionId();
  }

  // Function to update displayed events (only append new ones)
  function updateDisplayedEvents() {
    if (!$gameData?.events?.Events) return;
    
    // Check if we're in a new game session
    if (isNewGameSession()) {
      resetEventsForNewGame();
    }
    
    const currentEvents = $gameData.events.Events;
    const currentCount = currentEvents.length;
    
    // Only update if we have new events
    if (currentCount > lastEventCount) {
      const newEvents = currentEvents.slice(lastEventCount);
      
      // Mark new events for animation
      newEvents.forEach(event => {
        const eventId = `${event.EventName}_${event.EventTime}_${(event as any).KillerName || 'unknown'}`;
        newEventIds.add(eventId);
        
        // Remove the "new" flag after animation
        setTimeout(() => {
          newEventIds.delete(eventId);
        }, 1500);
      });
      
      // Add new events to the displayed list
      displayedEvents = [...displayedEvents, ...newEvents];
      
      // Keep only the last 20 events to prevent memory issues
      if (displayedEvents.length > 20) {
        displayedEvents = displayedEvents.slice(-20);
      }
      
      lastEventCount = currentCount;
    }
  }

  // Watch for game data changes and update events
  $effect(() => {
    if ($gameData?.events?.Events) {
      updateDisplayedEvents();
    }
  });

  // Initialize events on mount
  onMount(() => {
    if ($gameData?.events?.Events) {
      displayedEvents = $gameData.events.Events.slice(-20);
      lastEventCount = $gameData.events.Events.length;
      currentGameSession = generateGameSessionId();
    }
  });
</script>

{#if $gameData && typeof $gameData === 'object' && 'events' in $gameData}
  <Card class="bg-gradient-to-br from-card/50 to-card/30 backdrop-blur-sm border-border/50 shadow-lg">
    <CardHeader class="pb-4">
      <CardTitle class="flex items-center gap-3 text-lg">
        <div class="p-2 rounded-lg bg-primary/10">
          <Clock class="w-5 h-5 text-primary" />
        </div>
        <div class="flex-1">
          <div class="font-bold">Game Events Timeline</div>
          <div class="text-sm text-muted-foreground font-normal">Real-time game events</div>
        </div>
        <Badge variant="outline" class="text-xs">
          Latest First
        </Badge>
      </CardTitle>
    </CardHeader>
    <CardContent>
      {#if displayedEvents.length > 0}
        <div class="space-y-3 max-h-96 sm:max-h-[28rem] lg:max-h-[32rem] overflow-y-auto pr-2 scrollbar-thin scrollbar-thumb-border scrollbar-track-transparent">
          {#each displayedEvents.slice().reverse() as event, index}
            {@const config = getEventConfig(event.EventName)}
            {@const IconComponent = config.icon}
            {@const eventId = `${event.EventName}_${event.EventTime}_${(event as any).KillerName || 'unknown'}`}
            {@const isNewEvent = newEventIds.has(eventId)}
            <div class="relative">
              <!-- Timeline connector line -->
              {#if index < displayedEvents.length - 1}
                <div class="absolute left-6 top-12 w-0.5 h-6 bg-border"></div>
              {/if}
              
              <div class="flex items-start gap-3 sm:gap-4 p-3 sm:p-4 rounded-xl border-2 {config.borderColor} {config.bgColor} hover:shadow-md transition-all duration-300 group">
                <!-- Timeline dot with icon -->
                <div class="flex-shrink-0 w-10 h-10 sm:w-12 sm:h-12 rounded-full bg-background border-2 {config.borderColor} flex items-center justify-center shadow-sm group-hover:scale-110 transition-all duration-300">
                  <IconComponent class="w-4 h-4 sm:w-5 sm:h-5 {config.iconColor} {
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
                      {@const killerInfo = getChampionInfo(event.KillerName)}
                      {@const victimInfo = getChampionInfo(event.VictimName)}
                      <div class="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-3 flex-wrap">
                        <!-- Killer -->
                        <div class="flex items-center gap-2">
                          {#if killerInfo}
                            <div class="w-6 h-6 rounded-full overflow-hidden border-2 border-white shadow-sm">
                              <ImageWithFallback
                                src={getChampionImageUrlSync(killerInfo.championName)}
                                alt={killerInfo.championName}
                                fallback={getChampionFallback(killerInfo.championName)}
                                class="w-full h-full"
                                width={24}
                                height={24}
                              />
                            </div>
                          {/if}
                          <span class="px-2 py-1 rounded-md text-xs sm:text-sm font-semibold {config.killerColor} animate-pulse">
                            {event.KillerName}
                          </span>
                        </div>
                        
                        <span class="text-muted-foreground text-xs sm:text-sm">eliminated</span>
                        
                        <!-- Victim -->
                        <div class="flex items-center gap-2">
                          {#if victimInfo}
                            <div class="w-6 h-6 rounded-full overflow-hidden border-2 border-white shadow-sm">
                              <ImageWithFallback
                                src={getChampionImageUrlSync(victimInfo.championName)}
                                alt={victimInfo.championName}
                                fallback={getChampionFallback(victimInfo.championName)}
                                class="w-full h-full"
                                width={24}
                                height={24}
                              />
                            </div>
                          {/if}
                          <span class="px-2 py-1 rounded-md text-xs sm:text-sm font-semibold {config.victimColor}">
                            {event.VictimName}
                          </span>
                        </div>
                        
                        {#if event.Assisters && event.Assisters.length > 0}
                          <div class="text-xs text-muted-foreground bg-muted/50 px-2 py-1 rounded">
                            assisted by {event.Assisters.join(', ')}
                          </div>
                        {/if}
                      </div>
                    {:else if event.EventName === 'FirstBlood'}
                      {@const killerInfo = getChampionInfo(event.KillerName)}
                      {@const victimInfo = getChampionInfo(event.VictimName)}
                      <div class="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-3 flex-wrap">
                        <Badge variant="destructive" class="text-xs font-bold animate-pulse animate-bounce w-fit">
                          FIRST BLOOD
                        </Badge>
                        
                        <!-- Killer -->
                        <div class="flex items-center gap-2">
                          {#if killerInfo}
                            <div class="w-6 h-6 rounded-full overflow-hidden border-2 border-white shadow-sm">
                              <ImageWithFallback
                                src={getChampionImageUrlSync(killerInfo.championName)}
                                alt={killerInfo.championName}
                                fallback={getChampionFallback(killerInfo.championName)}
                                class="w-full h-full"
                                width={24}
                                height={24}
                              />
                            </div>
                          {/if}
                          <span class="px-2 py-1 rounded-md text-xs sm:text-sm font-semibold {config.killerColor} animate-bounce">
                            {event.KillerName}
                          </span>
                        </div>
                        
                        <span class="text-muted-foreground text-xs sm:text-sm">eliminated</span>
                        
                        <!-- Victim -->
                        <div class="flex items-center gap-2">
                          {#if victimInfo}
                            <div class="w-6 h-6 rounded-full overflow-hidden border-2 border-white shadow-sm">
                              <ImageWithFallback
                                src={getChampionImageUrlSync(victimInfo.championName)}
                                alt={victimInfo.championName}
                                fallback={getChampionFallback(victimInfo.championName)}
                                class="w-full h-full"
                                width={24}
                                height={24}
                              />
                            </div>
                          {/if}
                          <span class="px-2 py-1 rounded-md text-xs sm:text-sm font-semibold {config.victimColor}">
                            {event.VictimName}
                          </span>
                        </div>
                      </div>
                    {:else if event.EventName === 'TurretKilled'}
                      {@const killerInfo = getChampionInfo(event.KillerName)}
                      <div class="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-3 flex-wrap">
                        <!-- Killer -->
                        <div class="flex items-center gap-2">
                          {#if killerInfo}
                            <div class="w-6 h-6 rounded-full overflow-hidden border-2 border-white shadow-sm">
                              <ImageWithFallback
                                src={getChampionImageUrlSync(killerInfo.championName)}
                                alt={killerInfo.championName}
                                fallback={getChampionFallback(killerInfo.championName)}
                                class="w-full h-full"
                                width={24}
                                height={24}
                              />
                            </div>
                          {/if}
                          <span class="px-2 py-1 rounded-md text-xs sm:text-sm font-semibold {config.killerColor} animate-pulse">
                            {event.KillerName}
                          </span>
                        </div>
                        
                        <span class="text-muted-foreground text-xs sm:text-sm">destroyed</span>
                        <span class="text-orange-600 font-semibold text-xs sm:text-sm animate-bounce">🏰 Turret</span>
                      </div>
                    {:else if event.EventName === 'FirstBrick'}
                      {@const killerInfo = getChampionInfo(event.KillerName)}
                      <div class="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-3 flex-wrap">
                        <Badge variant="secondary" class="text-xs font-bold bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200 animate-bounce w-fit">
                          FIRST BRICK
                        </Badge>
                        
                        <!-- Killer -->
                        <div class="flex items-center gap-2">
                          {#if killerInfo}
                            <div class="w-6 h-6 rounded-full overflow-hidden border-2 border-white shadow-sm">
                              <ImageWithFallback
                                src={getChampionImageUrlSync(killerInfo.championName)}
                                alt={killerInfo.championName}
                                fallback={getChampionFallback(killerInfo.championName)}
                                class="w-full h-full"
                                width={24}
                                height={24}
                              />
                            </div>
                          {/if}
                          <span class="px-2 py-1 rounded-md text-xs sm:text-sm font-semibold {config.killerColor} animate-pulse">
                            {event.KillerName}
                          </span>
                        </div>
                      </div>
                    {:else if event.EventName === 'Multikill'}
                      {@const killerInfo = getChampionInfo(event.KillerName)}
                      <div class="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-3 flex-wrap">
                        <Badge variant="secondary" class="text-xs font-bold bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200 animate-pulse animate-bounce w-fit">
                          {event.KillStreak} KILL STREAK
                        </Badge>
                        
                        <!-- Killer -->
                        <div class="flex items-center gap-2">
                          {#if killerInfo}
                            <div class="w-6 h-6 rounded-full overflow-hidden border-2 border-white shadow-sm">
                              <ImageWithFallback
                                src={getChampionImageUrlSync(killerInfo.championName)}
                                alt={killerInfo.championName}
                                fallback={getChampionFallback(killerInfo.championName)}
                                class="w-full h-full"
                                width={24}
                                height={24}
                              />
                            </div>
                          {/if}
                          <span class="px-2 py-1 rounded-md text-xs sm:text-sm font-semibold {config.killerColor} animate-bounce">
                            {event.KillerName}
                          </span>
                        </div>
                      </div>
                    {:else if event.EventName === 'HordeKill'}
                      {@const killerInfo = getChampionInfo(event.KillerName)}
                      <div class="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-3 flex-wrap">
                        <!-- Killer -->
                        <div class="flex items-center gap-2">
                          {#if killerInfo}
                            <div class="w-6 h-6 rounded-full overflow-hidden border-2 border-white shadow-sm">
                              <ImageWithFallback
                                src={getChampionImageUrlSync(killerInfo.championName)}
                                alt={killerInfo.championName}
                                fallback={getChampionFallback(killerInfo.championName)}
                                class="w-full h-full"
                                width={24}
                                height={24}
                              />
                            </div>
                          {/if}
                          <span class="px-2 py-1 rounded-md text-xs sm:text-sm font-semibold {config.killerColor} animate-pulse">
                            {event.KillerName}
                          </span>
                        </div>
                        
                        <span class="text-muted-foreground text-xs sm:text-sm">slayed</span>
                        <span class="text-green-600 font-semibold text-xs sm:text-sm animate-bounce">🐉 Baron Nashor</span>
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
        <div class="text-center py-12 text-muted-foreground">
          <div class="p-4 rounded-full bg-primary/10 w-fit mx-auto mb-4">
            <Clock class="w-8 h-8 text-primary/50" />
          </div>
          <p class="text-sm">No game events available</p>
        </div>
      {/if}
    </CardContent>
  </Card>
{/if}
