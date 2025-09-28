<script lang="ts">
  import { AlertCircle, ChevronDown, ChevronRight, Eye, Copy, Check } from '@lucide/svelte';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import type { 
    ComprehensiveGameState, 
    LiveGameData, 
    GameStats, 
    PlayerScoresData
  } from '$lib/types/game';

  interface Props {
    gameState: ComprehensiveGameState | null;
    gameData: LiveGameData | null;
    liveGameStats: GameStats | null;
    playerScores: PlayerScoresData | null;
    isDebugExpanded: boolean;
    isSpectatorMode: boolean;
    copiedStates: Record<string, boolean>;
    onToggleDebug: () => void;
    onCopyToClipboard: (data: any, key: string) => void;
  }

  let {
    gameState,
    gameData,
    liveGameStats,
    playerScores,
    isDebugExpanded,
    isSpectatorMode,
    copiedStates,
    onToggleDebug,
    onCopyToClipboard
  }: Props = $props();
</script>

<Card>
  <CardHeader>
    <CardTitle 
      class="flex items-center justify-between cursor-pointer hover:bg-muted/50 rounded-md p-2 -m-2 transition-colors"
      onclick={onToggleDebug}
    >
      <div class="flex items-center gap-2">
        <AlertCircle class="w-5 h-5" />
        Debug Information
      </div>
      <div class="flex items-center gap-2">
        {#if isSpectatorMode}
          <Badge variant="secondary" class="text-xs">
            <Eye class="w-3 h-3 mr-1" />
            Spectator
          </Badge>
        {/if}
        {#if isDebugExpanded}
          <ChevronDown class="w-4 h-4" />
        {:else}
          <ChevronRight class="w-4 h-4" />
        {/if}
      </div>
    </CardTitle>
  </CardHeader>
  {#if isDebugExpanded}
    <CardContent>
      <div class="space-y-6 text-sm">
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
    </CardContent>
  {/if}
</Card>
