<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import ImageWithFallback from '$lib/components/ui/ImageWithFallback.svelte';
  import { 
    getChampionImageUrlSync, 
    getItemImageUrlSync, 
    getChampionFallback, 
    getItemFallback,
    getSummonerSpellImageUrlSync,
    getSummonerSpellFallback,
    getRoleIcon,
    getKeystoneImageUrlSync,
    getKeystoneFallback
  } from '$lib/services/imageService';
  import { onMount, onDestroy } from 'svelte';
  import type { Player, Item } from '$lib/types/game';
  import { gamePhase } from '$lib/stores/liveGameStore';

  interface Props {
    player: Player;
    teamColor: 'blue' | 'red';
  }

  let { player, teamColor }: Props = $props();

  // Respawn timer state
  let respawnTimer = $state(player.respawnTimer || 0);
  let timerInterval: number | null = null;

  const teamGradients = {
    blue: 'from-blue-50/50 to-transparent dark:from-blue-950/20',
    red: 'from-red-50/50 to-transparent dark:from-red-950/20'
  };

  const teamBackgrounds = {
    blue: 'bg-blue-100/50 dark:bg-blue-900/30',
    red: 'bg-red-100/50 dark:bg-red-900/30'
  };

  const avatarColors = {
    blue: 'from-blue-500 to-blue-600',
    red: 'from-red-500 to-red-600'
  };

  // Calculate total gold spent
  const totalGoldSpent = player.items?.reduce((total, item) => total + (item.price || 0), 0) || 0;

  // Format respawn timer
  function formatRespawnTimer(seconds: number): string {
    if (seconds <= 0) return '0s';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return mins > 0 ? `${mins}:${secs.toString().padStart(2, '0')}` : `${secs}s`;
  }

  // Start/stop respawn timer
  onMount(() => {
    if (player.isDead && player.respawnTimer > 0) {
      respawnTimer = player.respawnTimer;
      timerInterval = setInterval(() => {
        // Check if game has ended
        let currentPhase = '';
        const unsubscribe = gamePhase.subscribe(p => currentPhase = p);
        unsubscribe();
        
        if (currentPhase.includes('Victory') || currentPhase.includes('Game Ended')) {
          // Game has ended, stop the timer
          if (timerInterval) {
            clearInterval(timerInterval);
            timerInterval = null;
          }
          return;
        }
        
        respawnTimer -= 1;
        if (respawnTimer <= 0) {
          respawnTimer = 0;
          if (timerInterval) {
            clearInterval(timerInterval);
            timerInterval = null;
          }
        }
      }, 1000);
    }
  });

  onDestroy(() => {
    if (timerInterval) {
      clearInterval(timerInterval);
    }
  });
</script>

<div class="group relative overflow-hidden border rounded-lg bg-gradient-to-r {teamGradients[teamColor]} hover:shadow-md transition-all duration-200 {player.isDead ? 'ring-2 ring-red-500/50 shadow-red-500/20 shadow-lg bg-red-50/20 dark:bg-red-950/20' : teamBackgrounds[teamColor]}">
  <div class="flex items-center justify-between p-4">
    <div class="flex items-center gap-4">
      <!-- Champion Avatar -->
      <div class="relative">
        <div class="w-12 h-12 rounded-full overflow-hidden shadow-lg border-2 border-white/20 {player.isDead ? 'ring-2 ring-red-500/70 shadow-red-500/30 shadow-lg' : ''}">
          <ImageWithFallback
            src={getChampionImageUrlSync(player.championName)}
            alt={player.championName || 'Unknown Champion'}
            fallback={getChampionFallback(player.championName)}
            class="w-full h-full {player.isDead ? 'grayscale opacity-60' : ''}"
            width={48}
            height={48}
          />
        </div>
        {#if player.isDead}
          <!-- Large death indicator -->
          <div class="absolute -top-2 -right-2 w-8 h-8 bg-red-600 rounded-full flex items-center justify-center shadow-lg ring-2 ring-red-400/50 animate-pulse">
            <span class="text-sm text-white font-bold">💀</span>
          </div>
          <!-- Respawn timer overlay -->
          {#if respawnTimer > 0}
            <div class="absolute -bottom-1 -left-1 bg-orange-600 text-white text-xs rounded-full px-2 py-1 shadow-lg ring-2 ring-orange-400/50 font-bold">
              {formatRespawnTimer(respawnTimer)}
            </div>
          {/if}
        {/if}
      </div>
      
      <!-- Player Info -->
      <div class="flex-1">
        <div class="flex items-center gap-2 mb-1">
          <h4 class="font-semibold {player.isDead ? 'text-red-600 dark:text-red-400' : 'text-foreground'}">{player.championName || 'Unknown'}</h4>
          <Badge variant="outline" class="text-xs px-2 py-0.5 {player.isDead ? 'border-red-300 text-red-600 dark:border-red-700 dark:text-red-400' : ''}">
            <span class="mr-1">{getRoleIcon(player.position)}</span>
            {player.position || 'Unknown'}
          </Badge>
          {#if player.isDead}
            <Badge variant="destructive" class="text-xs animate-pulse">
              DEAD
            </Badge>
          {/if}
        </div>
        
        <!-- KDA and Level -->
        <div class="flex items-center gap-4 text-sm {player.isDead ? 'opacity-70' : ''}">
          <div class="flex items-center gap-1">
            <span class="text-green-600 font-medium">{player.scores?.kills || 0}</span>
            <span class="text-muted-foreground">/</span>
            <span class="text-red-600 font-medium {player.isDead ? 'animate-pulse' : ''}">{player.scores?.deaths || 0}</span>
            <span class="text-muted-foreground">/</span>
            <span class="text-blue-600 font-medium">{player.scores?.assists || 0}</span>
          </div>
          <div class="text-muted-foreground">Level {player.level}</div>
          <div class="text-muted-foreground">CS: {player.scores?.creepScore || 0}</div>
          <div class="text-muted-foreground">Ward: {Math.round((player.scores?.wardScore || 0) * 10) / 10}</div>
        </div>
        
        <!-- Summoner Spells -->
        {#if player.summonerSpells}
          <div class="flex items-center gap-2 mt-1">
            <div class="flex items-center gap-1 text-xs">
              <span class="text-muted-foreground">Spells:</span>
              <div class="flex gap-1">
                <div class="w-5 h-5 rounded border overflow-hidden" title={player.summonerSpells.summonerSpellOne?.displayName || 'Unknown'}>
                  <ImageWithFallback
                    src={getSummonerSpellImageUrlSync(player.summonerSpells.summonerSpellOne?.displayName)}
                    alt={player.summonerSpells.summonerSpellOne?.displayName || 'Unknown'}
                    fallback={getSummonerSpellFallback(player.summonerSpells.summonerSpellOne?.displayName)}
                    class="w-full h-full"
                    width={20}
                    height={20}
                  />
                </div>
                <div class="w-5 h-5 rounded border overflow-hidden" title={player.summonerSpells.summonerSpellTwo?.displayName || 'Unknown'}>
                  <ImageWithFallback
                    src={getSummonerSpellImageUrlSync(player.summonerSpells.summonerSpellTwo?.displayName)}
                    alt={player.summonerSpells.summonerSpellTwo?.displayName || 'Unknown'}
                    fallback={getSummonerSpellFallback(player.summonerSpells.summonerSpellTwo?.displayName)}
                    class="w-full h-full"
                    width={20}
                    height={20}
                  />
                </div>
              </div>
            </div>
          </div>
        {/if}
        
        <!-- Player Name and Skin -->
        <div class="text-xs text-muted-foreground mt-1">
          <div>{player.riotIdGameName || player.summonerName || 'Unknown Player'}</div>
          {#if player.skinName && player.skinName !== ''}
            <div class="text-xs text-muted-foreground/80 italic">{player.skinName}</div>
          {/if}
        </div>
      </div>
    </div>
    
    <!-- Gold and Items -->
    <div class="text-right">
      <div class="font-bold text-yellow-600 mb-1">
        {totalGoldSpent.toLocaleString()}g
      </div>
      <div class="text-xs text-muted-foreground mb-2">Gold Spent</div>
      <div class="flex gap-1">
        {#each (player.items || []).slice(0, 6) as item}
          <div class="relative w-6 h-6 rounded border overflow-hidden {!item.canUse ? 'opacity-60' : ''}" 
               title="{item.displayName || `Item ${item.itemID}`}{item.consumable ? ` (${item.count}x)` : ''}{!item.canUse ? ' - On Cooldown' : ''}">
            <ImageWithFallback
              src={getItemImageUrlSync(item.itemID)}
              alt={item.displayName || `Item ${item.itemID}`}
              fallback={getItemFallback(item.itemID)}
              class="w-full h-full"
              width={24}
              height={24}
            />
            {#if item.consumable && item.count > 1}
              <div class="absolute -top-1 -right-1 bg-blue-600 text-white text-xs rounded-full w-4 h-4 flex items-center justify-center font-bold">
                {item.count}
              </div>
            {/if}
            {#if !item.canUse && item.itemID !== 0}
              <div class="absolute inset-0 bg-black/30 flex items-center justify-center">
                <div class="w-2 h-2 bg-red-500 rounded-full"></div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  </div>
  
  <!-- Runes -->
  {#if player.runes}
    <div class="px-4 pb-3 border-t bg-muted/30">
      <div class="flex items-center gap-4 text-xs text-muted-foreground pt-2">
        <div class="flex items-center gap-2">
          <span class="font-medium">Keystone:</span>
          <div class="flex items-center gap-1">
            <div class="w-5 h-5 rounded border overflow-hidden" title={player.runes.keystone?.displayName || 'Unknown'}>
              <ImageWithFallback
                src={getKeystoneImageUrlSync(player.runes.keystone?.id)}
                alt={player.runes.keystone?.displayName || 'Unknown'}
                fallback={getKeystoneFallback(player.runes.keystone?.displayName)}
                class="w-full h-full"
                width={20}
                height={20}
              />
            </div>
            <span class="text-xs">{player.runes.keystone?.displayName || 'Unknown'}</span>
          </div>
        </div>
        <div class="flex items-center gap-1">
          <span class="font-medium">Primary:</span>
          <span>{player.runes.primaryRuneTree?.displayName || 'Unknown'}</span>
        </div>
        <div class="flex items-center gap-1">
          <span class="font-medium">Secondary:</span>
          <span>{player.runes.secondaryRuneTree?.displayName || 'Unknown'}</span>
        </div>
      </div>
    </div>
  {/if}
</div>
