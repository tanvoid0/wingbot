<script lang="ts">
  import { Users, Shield } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import PlayerCard from './PlayerCard.svelte';
  import type { Player } from '$lib/types/game';
  import { gameData } from '$lib/stores/liveGameStore';
</script>

<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
  <!-- Your Team -->
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Users class="w-5 h-5" />
        Your Team
      </CardTitle>
    </CardHeader>
    <CardContent>
      {#if $gameData?.allPlayers}
        <div class="space-y-3">
          {#each $gameData.allPlayers.filter((player: Player) => player.team === 'ORDER') as player}
            <PlayerCard {player} teamColor="blue" />
          {/each}
        </div>
      {:else}
        <div class="text-center py-8 text-muted-foreground">
          <Users class="w-12 h-12 mx-auto mb-4 opacity-50" />
          <p>No team data available</p>
        </div>
      {/if}
    </CardContent>
  </Card>

  <!-- Enemy Team -->
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Shield class="w-5 h-5" />
        Enemy Team
      </CardTitle>
    </CardHeader>
    <CardContent>
      {#if $gameData?.allPlayers}
        <div class="space-y-3">
          {#each $gameData.allPlayers.filter((player: Player) => player.team === 'CHAOS') as player}
            <PlayerCard {player} teamColor="red" />
          {/each}
        </div>
      {:else}
        <div class="text-center py-8 text-muted-foreground">
          <Shield class="w-12 h-12 mx-auto mb-4 opacity-50" />
          <p>No enemy data available</p>
        </div>
      {/if}
    </CardContent>
  </Card>
</div>
