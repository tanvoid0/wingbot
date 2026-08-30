<script lang="ts">
  import { Users, Shield, Sword } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { wsGameData, wsLiveGameData } from '$lib/stores/websocketStore';
  import { scoutProfiles, scoutActions } from '$lib/stores/scoutStore';
  import { extractScoutableFromAllPlayers } from '$lib/utils/championSelectParser';
  import PlayerCard from './PlayerCard.svelte';
  import type { Player } from '$lib/types/game';

  let gameData = $derived($wsGameData || $wsLiveGameData);
  let teams = $derived(
    gameData
      ? {
          blue:
            gameData.allPlayers?.filter((player: Player) => player.team === 'ORDER') || [],
          red:
            gameData.allPlayers?.filter((player: Player) => player.team === 'CHAOS') || [],
        }
      : { blue: [] as Player[], red: [] as Player[] },
  );

  let teamStats = $derived({
    blue: {
      totalKills: teams.blue.reduce((sum, player) => sum + (player.scores?.kills || 0), 0),
      totalDeaths: teams.blue.reduce((sum, player) => sum + (player.scores?.deaths || 0), 0),
      totalAssists: teams.blue.reduce((sum, player) => sum + (player.scores?.assists || 0), 0),
      totalGold: teams.blue.reduce((sum, player) => sum + (player.scores?.creepScore || 0), 0),
    },
    red: {
      totalKills: teams.red.reduce((sum, player) => sum + (player.scores?.kills || 0), 0),
      totalDeaths: teams.red.reduce((sum, player) => sum + (player.scores?.deaths || 0), 0),
      totalAssists: teams.red.reduce((sum, player) => sum + (player.scores?.assists || 0), 0),
      totalGold: teams.red.reduce((sum, player) => sum + (player.scores?.creepScore || 0), 0),
    },
  });

  let lastScoutKey = $state('');

  $effect(() => {
    const all = [...teams.blue, ...teams.red] as unknown as Array<Record<string, unknown>>;
    if (all.length === 0) return;
    const key = all.map((p) => `${p.summonerName}:${p.championName}`).sort().join('|');
    if (key === lastScoutKey) return;
    lastScoutKey = key;
    void scoutActions.scoutPlayers(extractScoutableFromAllPlayers(all), {
      includeChampionForm: false,
    });
  });
</script>

{#if gameData?.allPlayers && gameData.allPlayers.length > 0}
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Users class="w-5 h-5" />
        Team Comparison
      </CardTitle>
    </CardHeader>
    <CardContent>
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div class="space-y-4">
          <div class="flex items-center gap-2 mb-4">
            <div class="w-4 h-4 bg-blue-500 rounded"></div>
            <h3 class="text-lg font-semibold">Blue Team</h3>
            <Badge variant="outline" class="bg-blue-50 text-blue-700 dark:bg-blue-900 dark:text-blue-300">
              {teams.blue.length} players
            </Badge>
          </div>
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
          <div class="space-y-2">
            {#each teams.blue as player (player.summonerName)}
              {#if player?.summonerName}
                <PlayerCard {player} teamColor="blue" scoutProfile={$scoutProfiles} />
              {/if}
            {/each}
          </div>
        </div>

        <div class="space-y-4">
          <div class="flex items-center gap-2 mb-4">
            <div class="w-4 h-4 bg-red-500 rounded"></div>
            <h3 class="text-lg font-semibold">Red Team</h3>
            <Badge variant="outline" class="bg-red-50 text-red-700 dark:bg-red-900 dark:text-red-300">
              {teams.red.length} players
            </Badge>
          </div>
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
          <div class="space-y-2">
            {#each teams.red as player (player.summonerName)}
              {#if player?.summonerName}
                <PlayerCard {player} teamColor="red" scoutProfile={$scoutProfiles} />
              {/if}
            {/each}
          </div>
        </div>
      </div>
    </CardContent>
  </Card>
{/if}
