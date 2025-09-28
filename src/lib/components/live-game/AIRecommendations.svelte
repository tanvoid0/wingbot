<script lang="ts">
  import { Zap, Target, Sword, Shield } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';

  interface Props {
    gameMode: string;
  }

  let { gameMode }: Props = $props();

  function isArenaMode() {
    return gameMode === 'Arena';
  }
</script>

<Card>
  <CardHeader>
    <CardTitle class="flex items-center gap-2">
      <Zap class="w-5 h-5" />
      AI Recommendations
    </CardTitle>
  </CardHeader>
  <CardContent>
    <div class="space-y-4">
      {#if isArenaMode()}
        <!-- Arena-specific recommendations -->
        <div class="p-4 border rounded-lg bg-purple-50 dark:bg-purple-950">
          <div class="flex items-center gap-2 mb-2">
            <Target class="w-4 h-4 text-purple-600" />
            <span class="font-medium text-purple-900 dark:text-purple-100">Arena Strategy</span>
          </div>
          <p class="text-sm text-purple-800 dark:text-purple-200">
            Focus on positioning and team coordination. Arena rounds are fast-paced - prioritize quick eliminations.
          </p>
        </div>
        
        <div class="p-4 border rounded-lg bg-green-50 dark:bg-green-950">
          <div class="flex items-center gap-2 mb-2">
            <Sword class="w-4 h-4 text-green-600" />
            <span class="font-medium text-green-900 dark:text-green-100">Combat Focus</span>
          </div>
          <p class="text-sm text-green-800 dark:text-green-200">
            Target the weakest enemy first to gain numerical advantage. Use the arena's circular layout to your advantage.
          </p>
        </div>
        
        <div class="p-4 border rounded-lg bg-yellow-50 dark:bg-yellow-950">
          <div class="flex items-center gap-2 mb-2">
            <Shield class="w-4 h-4 text-yellow-600" />
            <span class="font-medium text-yellow-900 dark:text-yellow-100">Round Management</span>
          </div>
          <p class="text-sm text-yellow-800 dark:text-yellow-200">
            Each round is independent. Don't be afraid to take risks - you'll respawn for the next round.
          </p>
        </div>
      {:else}
        <!-- Classic game recommendations -->
        <div class="p-4 border rounded-lg bg-blue-50 dark:bg-blue-950">
          <div class="flex items-center gap-2 mb-2">
            <Target class="w-4 h-4 text-blue-600" />
            <span class="font-medium text-blue-900 dark:text-blue-100">Objective Priority</span>
          </div>
          <p class="text-sm text-blue-800 dark:text-blue-200">
            Focus on securing dragon control. Your team has better positioning for the next dragon spawn.
          </p>
        </div>
        
        <div class="p-4 border rounded-lg bg-green-50 dark:bg-green-950">
          <div class="flex items-center gap-2 mb-2">
            <Sword class="w-4 h-4 text-green-600" />
            <span class="font-medium text-green-900 dark:text-green-100">Combat Advice</span>
          </div>
          <p class="text-sm text-green-800 dark:text-green-200">
            Enemy ADC is behind in gold. Look for opportunities to engage when they're farming alone.
          </p>
        </div>
        
        <div class="p-4 border rounded-lg bg-yellow-50 dark:bg-yellow-950">
          <div class="flex items-center gap-2 mb-2">
            <Shield class="w-4 h-4 text-yellow-600" />
            <span class="font-medium text-yellow-900 dark:text-yellow-100">Defensive Play</span>
          </div>
          <p class="text-sm text-yellow-800 dark:text-yellow-200">
            Enemy team is grouping for a push. Consider defending under tower or looking for a flank.
          </p>
        </div>
      {/if}
    </div>
  </CardContent>
</Card>
