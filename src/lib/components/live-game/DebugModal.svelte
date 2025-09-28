<script lang="ts">
  import { X, Copy, Check } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';
  import type { 
    ComprehensiveGameState, 
    LiveGameData, 
    GameStats, 
    PlayerScoresData
  } from '$lib/types/game';

  interface Props {
    isOpen: boolean;
    gameState: ComprehensiveGameState | null;
    gameData: LiveGameData | null;
    liveGameStats: GameStats | null;
    playerScores: PlayerScoresData | null;
    isSpectatorMode: boolean;
    copiedStates: Record<string, boolean>;
    onClose: () => void;
    onCopyToClipboard: (data: any, key: string) => Promise<void>;
  }

  let {
    isOpen,
    gameState,
    gameData,
    liveGameStats,
    playerScores,
    isSpectatorMode,
    copiedStates,
    onClose,
    onCopyToClipboard
  }: Props = $props();

  // Debug logging
  $effect(() => {
    console.log('DebugModal isOpen changed to:', isOpen);
    console.log('DebugModal props:', { isOpen, gameState: !!gameState, gameData: !!gameData });
  });
</script>

{#if isOpen}
  <!-- Modal Backdrop -->
  <div 
    class="fixed inset-0 z-50 bg-black/50 flex items-center justify-center p-4" 
    onclick={onClose}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <!-- Modal Content -->
    <div 
      class="bg-background border rounded-lg shadow-lg max-w-4xl w-full max-h-[80vh] overflow-hidden" 
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Modal Header -->
      <div class="flex items-center justify-between p-6 border-b">
        <h2 class="text-lg font-semibold flex items-center gap-2">
          <span>Debug Information</span>
          {#if isSpectatorMode}
            <span class="text-xs bg-orange-100 text-orange-800 px-2 py-1 rounded">Spectator Mode</span>
          {/if}
        </h2>
        <Button variant="ghost" size="sm" onclick={onClose}>
          <X class="w-4 h-4" />
        </Button>
      </div>
      
      <!-- Modal Content -->
      <div class="p-6 overflow-y-auto space-y-6 text-sm">
      <!-- Game State -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <h4 class="font-semibold text-foreground">Game State</h4>
          <Button
            variant="outline"
            size="sm"
            onclick={() => onCopyToClipboard(gameState, 'gameState')}
            class="h-7 px-3 text-xs"
          >
            {#if copiedStates.gameState}
              <Check class="w-3 h-3 mr-1" />
              Copied!
            {:else}
              <Copy class="w-3 h-3 mr-1" />
              Copy
            {/if}
          </Button>
        </div>
        <div class="bg-muted/50 rounded-lg p-3 border">
          <pre class="text-xs overflow-auto max-h-40 whitespace-pre-wrap">{JSON.stringify(gameState, null, 2)}</pre>
        </div>
      </div>

      <!-- Game Data -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <h4 class="font-semibold text-foreground">Game Data</h4>
          <Button
            variant="outline"
            size="sm"
            onclick={() => onCopyToClipboard(gameData, 'gameData')}
            class="h-7 px-3 text-xs"
          >
            {#if copiedStates.gameData}
              <Check class="w-3 h-3 mr-1" />
              Copied!
            {:else}
              <Copy class="w-3 h-3 mr-1" />
              Copy
            {/if}
          </Button>
        </div>
        <div class="bg-muted/50 rounded-lg p-3 border">
          <pre class="text-xs overflow-auto max-h-40 whitespace-pre-wrap">{JSON.stringify(gameData, null, 2)}</pre>
        </div>
      </div>

      <!-- Live Game Stats -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <h4 class="font-semibold text-foreground">Live Game Stats</h4>
          <Button
            variant="outline"
            size="sm"
            onclick={() => onCopyToClipboard(liveGameStats, 'liveGameStats')}
            class="h-7 px-3 text-xs"
          >
            {#if copiedStates.liveGameStats}
              <Check class="w-3 h-3 mr-1" />
              Copied!
            {:else}
              <Copy class="w-3 h-3 mr-1" />
              Copy
            {/if}
          </Button>
        </div>
        <div class="bg-muted/50 rounded-lg p-3 border">
          <pre class="text-xs overflow-auto max-h-40 whitespace-pre-wrap">{JSON.stringify(liveGameStats, null, 2)}</pre>
        </div>
      </div>

      <!-- Player Scores -->
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <h4 class="font-semibold text-foreground">Player Scores</h4>
          <Button
            variant="outline"
            size="sm"
            onclick={() => onCopyToClipboard(playerScores, 'playerScores')}
            class="h-7 px-3 text-xs"
          >
            {#if copiedStates.playerScores}
              <Check class="w-3 h-3 mr-1" />
              Copied!
            {:else}
              <Copy class="w-3 h-3 mr-1" />
              Copy
            {/if}
          </Button>
        </div>
        <div class="bg-muted/50 rounded-lg p-3 border">
          <pre class="text-xs overflow-auto max-h-40 whitespace-pre-wrap">{JSON.stringify(playerScores, null, 2)}</pre>
        </div>
      </div>
    </div>
    </div>
  </div>
{/if}
