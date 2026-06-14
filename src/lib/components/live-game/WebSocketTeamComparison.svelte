<script lang="ts">
  import { Users, Shield, Sword } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { wsGameData, wsLiveGameData } from '$lib/stores/websocketStore';
  import PlayerCard from './PlayerCard.svelte';

  // Process game data to separate teams
  $: gameData = $wsGameData || $wsLiveGameData;
  $: teams = gameData ? {
    blue: gameData.allPlayers?.filter((player: any) => player.team === 'ORDER') || [],
    red: gameData.allPlayers?.filter((player: any) => player.team === 'CHAOS') || []
  } : { blue: [], red: [] };

  // Calculate team stats
  $: teamStats = {
    blue: {
      totalKills: teams.blue.reduce((sum: number, player: any) => sum + (player.scores?.kills || 0), 0),
      totalDeaths: teams.blue.reduce((sum: number, player: any) => sum + (player.scores?.deaths || 0), 0),
      totalAssists: teams.blue.reduce((sum: number, player: any) => sum + (player.scores?.assists || 0), 0),
      totalGold: teams.blue.reduce((sum: number, player: any) => sum + (player.scores?.creepScore || 0), 0)
    },
    red: {
      totalKills: teams.red.reduce((sum: number, player: any) => sum + (player.scores?.kills || 0), 0),
      totalDeaths: teams.red.reduce((sum: number, player: any) => sum + (player.scores?.deaths || 0), 0),
      totalAssists: teams.red.reduce((sum: number, player: any) => sum + (player.scores?.assists || 0), 0),
      totalGold: teams.red.reduce((sum: number, player: any) => sum + (player.scores?.creepScore || 0), 0)
    }
  };
</script>

{#if $wsGameData?.allPlayers && $wsGameData.allPlayers.length > 0}
<Card>
  <CardHeader>
    <CardTitle class="flex items-center gap-2">
      <Users class="w-5 h-5" />
      Team Comparison
    </CardTitle>
  </CardHeader>
  <CardContent>
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- Blue Team -->
      <div class="space-y-4">
        <div class="flex items-center gap-2 mb-4">
          <div class="w-4 h-4 bg-blue-500 rounded"></div>
          <h3 class="text-lg font-semibold">Blue Team</h3>
          <Badge variant="outline" class="bg-blue-50 text-blue-700 dark:bg-blue-900 dark:text-blue-300">
            {teams.blue.length} players
          </Badge>
        </div>
        
        <!-- Team Stats -->
        <div class="grid grid-cols-2 gap-2 text-sm">
          <div class="flex justify-between">
            <span class="text-muted-foreground">Kills:</span>
            <span class="font-medium">{teamStats.blue.totalKills}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-muted-foreground">Deaths:</span>
            <span class="font-medium">{teamStats.blue.totalDeaths}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-muted-foreground">Assists:</span>
            <span class="font-medium">{teamStats.blue.totalAssists}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-muted-foreground">CS:</span>
            <span class="font-medium">{teamStats.blue.totalGold}</span>
          </div>
        </div>
        
        <!-- Players -->
        <div class="space-y-2">
          {#each teams.blue as player (player.summonerName)}
            {#if player && player.summonerName}
              <PlayerCard {player} teamColor="blue" />
            {/if}
          {/each}
        </div>
      </div>
      
      <!-- Red Team -->
      <div class="space-y-4">
        <div class="flex items-center gap-2 mb-4">
          <div class="w-4 h-4 bg-red-500 rounded"></div>
          <h3 class="text-lg font-semibold">Red Team</h3>
          <Badge variant="outline" class="bg-red-50 text-red-700 dark:bg-red-900 dark:text-red-300">
            {teams.red.length} players
          </Badge>
        </div>
        
        <!-- Team Stats -->
        <div class="grid grid-cols-2 gap-2 text-sm">
          <div class="flex justify-between">
            <span class="text-muted-foreground">Kills:</span>
            <span class="font-medium">{teamStats.red.totalKills}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-muted-foreground">Deaths:</span>
            <span class="font-medium">{teamStats.red.totalDeaths}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-muted-foreground">Assists:</span>
            <span class="font-medium">{teamStats.red.totalAssists}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-muted-foreground">CS:</span>
            <span class="font-medium">{teamStats.red.totalGold}</span>
          </div>
        </div>
        
        <!-- Players -->
        <div class="space-y-2">
          {#each teams.red as player (player.summonerName)}
            {#if player && player.summonerName}
              <PlayerCard {player} teamColor="red" />
            {/if}
          {/each}
        </div>
      </div>
    </div>
  </CardContent>
</Card>
{/if}
