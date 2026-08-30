<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Skull } from '@lucide/svelte';

  type DeathTimer = {
    riot_id: string;
    champion: string;
    position: string;
    level: number;
    is_dead: boolean;
    respawn_in_sec: number;
  };

  let enemies = $state<DeathTimer[]>([]);
  let error = $state<string | null>(null);

  // Riot reports respawn as a snapshot, so count it down locally between polls
  // rather than showing a stale number for up to a full poll interval.
  let tickedAt = $state(Date.now());

  function remaining(enemy: DeathTimer): number {
    if (!enemy.is_dead) return 0;
    const elapsed = (Date.now() - tickedAt) / 1000;
    return Math.max(0, enemy.respawn_in_sec - elapsed);
  }

  function format(seconds: number): string {
    const total = Math.ceil(seconds);
    const mins = Math.floor(total / 60);
    const secs = total % 60;
    return mins > 0 ? `${mins}:${secs.toString().padStart(2, '0')}` : `${secs}s`;
  }

  onMount(() => {
    let stopped = false;

    async function poll() {
      try {
        const data = await invoke<{ enemies: DeathTimer[] }>('get_death_timers');
        if (stopped) return;
        enemies = data.enemies ?? [];
        tickedAt = Date.now();
        error = null;
      } catch (e) {
        if (!stopped) error = String(e);
      }
    }

    void poll();
    const pollId = setInterval(poll, 2000);
    // Separate, faster tick so the countdown stays smooth between polls.
    const tickId = setInterval(() => { tickedAt = tickedAt; enemies = enemies; }, 250);

    return () => {
      stopped = true;
      clearInterval(pollId);
      clearInterval(tickId);
    };
  });

  const dead = $derived(enemies.filter((e) => e.is_dead && remaining(e) > 0));
</script>

<Card>
  <CardHeader class="pb-3">
    <CardTitle class="flex items-center gap-2 text-base">
      <Skull class="w-4 h-4 text-destructive" />
      Enemy Death Timers
      {#if dead.length > 0}
        <span class="text-xs font-normal text-muted-foreground">{dead.length} down</span>
      {/if}
    </CardTitle>
  </CardHeader>
  <CardContent>
    {#if error}
      <p class="text-sm text-muted-foreground">Not in a live game.</p>
    {:else if dead.length === 0}
      <p class="text-sm text-muted-foreground">All enemies alive.</p>
    {:else}
      <ul class="space-y-2">
        {#each dead as enemy (enemy.riot_id + enemy.champion)}
          {@const left = remaining(enemy)}
          <li class="flex items-center justify-between gap-3">
            <div class="min-w-0">
              <div class="font-medium truncate">{enemy.champion}</div>
              <div class="text-xs text-muted-foreground truncate">
                {enemy.position || 'Unknown'} · Lv {enemy.level}
              </div>
            </div>
            <div
              class="font-mono tabular-nums text-lg {left < 10
                ? 'text-destructive'
                : 'text-foreground'}"
            >
              {format(left)}
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </CardContent>
</Card>
