<script lang="ts">
  import { Users, Eye, Loader2 } from '@lucide/svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import ImageWithFallback from '$lib/components/ui/ImageWithFallback.svelte';
  import { wsGameState, wsIsSpectatorMode } from '$lib/stores/websocketStore';
  import { scoutProfiles, scoutLoading, scoutApiRequired, scoutActions } from '$lib/stores/scoutStore';
  import {
    extractScoutableFromAllPlayers,
    scoutEnemyKey,
    playerToScoutSlot,
  } from '$lib/utils/championSelectParser';
  import {
    formatMasteryPoints,
    getRankTierColor,
    getThreatLabel,
    type ScoutPlayerProfile,
  } from '$lib/utils/scout';
  import {
    getChampionImageUrlSync,
    getSummonerSpellImageUrlSync,
    getRankEmblemUrl,
  } from '$lib/services/imageService';

  let lastScoutKey = $state('');

  const spectatorGame = $derived(
    ($wsGameState?.spectator_game ?? null) as Record<string, unknown> | null,
  );

  const allPlayers = $derived(
    (spectatorGame?.allPlayers as Array<Record<string, unknown>> | undefined) ?? [],
  );

  const teams = $derived({
    blue: allPlayers.filter((p) => p.team === 'ORDER'),
    red: allPlayers.filter((p) => p.team === 'CHAOS'),
  });

  $effect(() => {
    if (!$wsIsSpectatorMode || allPlayers.length === 0) {
      lastScoutKey = '';
      return;
    }
    const key = allPlayers.map((p) => scoutEnemyKey(playerToScoutSlot(p))).sort().join('|');
    if (key && key !== lastScoutKey) {
      lastScoutKey = key;
      void scoutActions.scoutPlayers(extractScoutableFromAllPlayers(allPlayers), {
        includeChampionForm: false,
      });
    }
  });

  function profileFor(player: Record<string, unknown>): ScoutPlayerProfile | undefined {
    return $scoutProfiles[scoutEnemyKey(playerToScoutSlot(player))];
  }
</script>

{#if $wsIsSpectatorMode}
  <Card class="border-blue-500/20">
    <CardHeader class="pb-2">
      <CardTitle class="flex items-center gap-2 text-base">
        <Eye class="w-4 h-4 text-blue-500" />
        Spectator Roster
        {#if $scoutLoading}
          <Loader2 class="w-3.5 h-3.5 animate-spin text-muted-foreground" />
        {/if}
      </CardTitle>
    </CardHeader>
    <CardContent class="space-y-3">
      {#if $scoutApiRequired}
        <Alert class="py-2 border-amber-500/30 bg-amber-500/10">
          <AlertDescription class="text-sm">
            Add a Riot API key in Settings for spectator roster and rank data.
          </AlertDescription>
        </Alert>
      {/if}

      {#if allPlayers.length === 0}
        <p class="text-sm text-muted-foreground">
          Fetching game from Riot Spectator API… If this persists, confirm your API key and that you are spectating a live match.
        </p>
      {:else}
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {#each [{ label: 'Blue Team', color: 'blue', players: teams.blue }, { label: 'Red Team', color: 'red', players: teams.red }] as team}
            <div class="space-y-2">
              <div class="flex items-center gap-2">
                <Users class="w-4 h-4" />
                <h3 class="font-semibold text-sm">{team.label}</h3>
                <Badge variant="outline" class="text-xs">{team.players.length}</Badge>
              </div>
              {#each team.players as player (scoutEnemyKey(playerToScoutSlot(player)))}
                {@const profile = profileFor(player)}
                {@const rank = profile?.primary_rank ?? profile?.solo_rank}
                {@const threat = profile ? getThreatLabel(profile) : null}
                {@const spells = player.summonerSpells as Record<string, { displayName?: string }> | undefined}
                <div class="rounded-lg border bg-card/60 p-2.5 flex gap-2.5">
                  <ImageWithFallback
                    src={getChampionImageUrlSync(player.championName as string)}
                    alt={player.championName as string}
                    class="w-10 h-10 rounded-lg object-cover shrink-0"
                    fallback="?"
                  />
                  <div class="min-w-0 flex-1 space-y-1">
                    <p class="text-xs font-medium truncate">
                      {player.riotIdGameName
                        ? `${player.riotIdGameName}#${player.riotIdTagLine ?? ''}`
                        : (player.summonerName as string) ?? 'Unknown'}
                    </p>
                    <p class="text-[11px] text-muted-foreground">{player.championName as string}</p>
                    {#if rank}
                      <p class="text-[10px] {getRankTierColor(rank.tier)}">{rank.summary}</p>
                    {/if}
                    {#if profile?.champion_mastery}
                      <p class="text-[10px] text-muted-foreground">
                        M{profile.champion_mastery.level} · {formatMasteryPoints(profile.champion_mastery.points)}
                      </p>
                    {/if}
                    {#if threat}
                      <Badge variant="outline" class="text-[9px] h-5">{threat}</Badge>
                    {/if}
                  </div>
                  <div class="flex flex-col gap-0.5 shrink-0">
                    {#if spells?.summonerSpellOne?.displayName}
                      <ImageWithFallback
                        src={getSummonerSpellImageUrlSync(spells.summonerSpellOne.displayName)}
                        alt="Spell 1"
                        class="w-5 h-5 rounded"
                        fallback="?"
                      />
                    {/if}
                    {#if spells?.summonerSpellTwo?.displayName}
                      <ImageWithFallback
                        src={getSummonerSpellImageUrlSync(spells.summonerSpellTwo.displayName)}
                        alt="Spell 2"
                        class="w-5 h-5 rounded"
                        fallback="?"
                      />
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {/each}
        </div>
      {/if}
    </CardContent>
  </Card>
{/if}
