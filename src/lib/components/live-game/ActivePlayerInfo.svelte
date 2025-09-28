<script lang="ts">
  import { Target } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import type { Ability } from '$lib/types/game';
  import { gameData, isSpectatorMode } from '$lib/stores/liveGameStore';
</script>

{#if $gameData?.activePlayer && !$isSpectatorMode}
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Target class="w-5 h-5" />
        Your Champion: {$gameData.activePlayer.championName || 'Unknown'}
      </CardTitle>
    </CardHeader>
    <CardContent>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <!-- Stats -->
        <div class="space-y-3">
          <h4 class="font-semibold text-sm text-muted-foreground">COMBAT STATS</h4>
          <div class="space-y-2">
            <div class="flex justify-between">
              <span class="text-sm">Attack Damage:</span>
              <span class="font-medium">{Math.round($gameData.activePlayer.championStats?.attackDamage || 0)}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-sm">Ability Power:</span>
              <span class="font-medium">{Math.round($gameData.activePlayer.championStats?.abilityPower || 0)}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-sm">Armor:</span>
              <span class="font-medium">{Math.round($gameData.activePlayer.championStats?.armor || 0)}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-sm">Magic Resist:</span>
              <span class="font-medium">{Math.round($gameData.activePlayer.championStats?.magicResist || 0)}</span>
            </div>
          </div>
        </div>
        
        <!-- Health & Resources -->
        <div class="space-y-3">
          <h4 class="font-semibold text-sm text-muted-foreground">HEALTH & RESOURCES</h4>
          <div class="space-y-2">
            <div class="flex justify-between">
              <span class="text-sm">Current Health:</span>
              <span class="font-medium">{Math.round($gameData.activePlayer.championStats?.currentHealth || 0)}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-sm">Max Health:</span>
              <span class="font-medium">{Math.round($gameData.activePlayer.championStats?.maxHealth || 0)}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-sm">Current Gold:</span>
              <span class="font-medium">{$gameData.activePlayer.currentGold || 0}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-sm">Move Speed:</span>
              <span class="font-medium">{Math.round($gameData.activePlayer.championStats?.moveSpeed || 0)}</span>
            </div>
          </div>
        </div>
        
        <!-- Abilities -->
        <div class="space-y-3">
          <h4 class="font-semibold text-sm text-muted-foreground">ABILITIES</h4>
          <div class="space-y-2">
            {#each Object.entries($gameData.activePlayer.abilities || {}) as [key, ability]}
              {#if key !== 'Passive'}
                <div class="flex justify-between">
                  <span class="text-sm">{key}:</span>
                  <span class="font-medium">Level {(ability as Ability).abilityLevel || 0}</span>
                </div>
              {/if}
            {/each}
          </div>
        </div>
      </div>
    </CardContent>
  </Card>
{/if}
