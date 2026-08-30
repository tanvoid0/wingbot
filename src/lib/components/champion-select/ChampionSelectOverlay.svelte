<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Flame } from '@lucide/svelte';
  import { wsChampionSelect, wsGamePhase } from '$lib/stores/websocketStore';
  import { scoutProfiles } from '$lib/stores/scoutStore';
  import {
    getChampionImageUrlByIdSync,
    getChampionImageUrlSync,
    getSummonerSpellImageUrlByIdSync,
    getChampionDataCached,
    getRankEmblemUrl,
  } from '$lib/services/imageService';
  import {
    champSelectTimerSec,
    isTimerOnlyChampSelectChange,
    parseChampionSelectSession,
    resolveChampionDisplayName,
    scoutEnemyKey,
    type ParsedChampionSelect,
    type ChampSelectSlot,
  } from '$lib/utils/championSelectParser';
  import {
    getRankTierColor,
    getThreatLabel,
    type ScoutPlayerProfile,
  } from '$lib/utils/scout';
  import ImageWithFallback from '$lib/components/ui/ImageWithFallback.svelte';
  
  let championSelectData: Record<string, unknown> | null = null;
  let parsed: ParsedChampionSelect | null = null;
  let gamePhase: string = 'unknown';
  let championNamesById: Record<number, string> = {};
  let profiles: Record<string, ScoutPlayerProfile> = {};
  
  let blueTeam = {
    name: 'Your Team',
    score: 0,
    bans: [] as ChampSelectSlot[],
    picks: [] as ChampSelectSlot[],
    coach: '',
  };
  
  let redTeam = {
    name: 'Enemy Team',
    score: 0,
    bans: [] as ChampSelectSlot[],
    picks: [] as ChampSelectSlot[],
    coach: '',
  };
  
  let timeRemaining = 0;
  let currentPhase = 'unknown';
  let timerSnapshot: Record<string, unknown> | null = null;
  let countdownInterval: ReturnType<typeof setInterval> | null = null;
  
  let unsubscribe: (() => void)[] = [];
  let lastSessionSnapshot: Record<string, unknown> | null = null;

  function syncTimerFromSnapshot() {
    timeRemaining = champSelectTimerSec(timerSnapshot);
  }

  function startTimerCountdown() {
    if (countdownInterval) {
      clearInterval(countdownInterval);
      countdownInterval = null;
    }
    if (!timerSnapshot) return;
    syncTimerFromSnapshot();
    countdownInterval = setInterval(syncTimerFromSnapshot, 100);
  }

  function stopTimerCountdown() {
    if (countdownInterval) {
      clearInterval(countdownInterval);
      countdownInterval = null;
    }
    timerSnapshot = null;
    timeRemaining = 0;
  }

  function applyTimerOnlyUpdate(data: Record<string, unknown>) {
    championSelectData = data;
    timerSnapshot =
      data.timer && typeof data.timer === 'object'
        ? (data.timer as Record<string, unknown>)
        : null;
    if (timerSnapshot && typeof timerSnapshot.phase === 'string') {
      currentPhase = timerSnapshot.phase;
    }
    if (timerSnapshot) {
      syncTimerFromSnapshot();
    }
  }
  
  function applyChampionSelectData(data: Record<string, unknown> | null) {
    if (!data) {
      lastSessionSnapshot = null;
      championSelectData = null;
      parsed = null;
      blueTeam = { ...blueTeam, bans: [], picks: [] };
      redTeam = { ...redTeam, bans: [], picks: [] };
      stopTimerCountdown();
      return;
    }

    if (
      lastSessionSnapshot &&
      isTimerOnlyChampSelectChange(lastSessionSnapshot, data)
    ) {
      lastSessionSnapshot = data;
      applyTimerOnlyUpdate(data);
      return;
    }

    lastSessionSnapshot = data;
    championSelectData = data;
    parsed = parseChampionSelectSession(data);
    
    if (!parsed) {
      blueTeam = { ...blueTeam, bans: [], picks: [] };
      redTeam = { ...redTeam, bans: [], picks: [] };
      stopTimerCountdown();
      return;
    }
    
    blueTeam = {
      name: 'Your Team',
      score: 0,
      bans: parsed.blueTeam.bans,
      picks: parsed.blueTeam.picks,
      coach: '',
    };
    redTeam = {
      name: 'Enemy Team',
      score: 0,
      bans: parsed.redTeam.bans,
      picks: parsed.redTeam.picks,
      coach: '',
    };
    timerSnapshot =
      data?.timer && typeof data.timer === 'object'
        ? (data.timer as Record<string, unknown>)
        : null;
    currentPhase = parsed.phase;
    startTimerCountdown();
  }
  
  onMount(async () => {
    try {
      const data = await getChampionDataCached();
      const map: Record<number, string> = {};
      for (const champ of Object.values(data)) {
        const key = parseInt((champ as { key?: string }).key ?? '', 10);
        const name = (champ as { name?: string }).name;
        if (key && name) map[key] = name;
      }
      championNamesById = map;
    } catch {
      // champion names are optional UI polish
    }

    const unsub1 = wsChampionSelect.subscribe((data: Record<string, unknown> | null) => {
      applyChampionSelectData(data);
    });
    
    const unsub2 = wsGamePhase.subscribe((phase) => {
      gamePhase = phase;
    });

    const unsub3 = scoutProfiles.subscribe((next) => {
      profiles = next;
    });
    
    unsubscribe = [unsub1, unsub2, unsub3];
  });
  
  onDestroy(() => {
    stopTimerCountdown();
    unsubscribe.forEach((unsub) => unsub());
  });
  
  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function formatPosition(pos: string | undefined): string {
    if (!pos) return '';
    const map: Record<string, string> = {
      TOP: 'TOP',
      JUNGLE: 'JUNGLE',
      MIDDLE: 'MID',
      BOTTOM: 'BOTTOM',
      UTILITY: 'SUPPORT',
    };
    return map[pos.toUpperCase()] ?? pos.toUpperCase();
  }
  
  function championImage(slot: ChampSelectSlot): string {
    if (slot.championId > 0) {
      return (
        getChampionImageUrlByIdSync(slot.championId) ||
        getChampionImageUrlSync(championName(slot)) ||
        ''
      );
    }
    if (slot.championName) {
      return getChampionImageUrlSync(slot.championName) || '';
    }
    return '';
  }

  function championName(slot: ChampSelectSlot): string {
    return resolveChampionDisplayName(slot, championNamesById);
  }

  function scoutProfile(slot: ChampSelectSlot | undefined): ScoutPlayerProfile | undefined {
    if (!slot) return undefined;
    return profiles[scoutEnemyKey(slot)];
  }
  
  function spellImage(spellId: number | undefined): string {
    if (!spellId) return '';
    return getSummonerSpellImageUrlByIdSync(spellId) ?? '';
  }

  function displayName(slot: ChampSelectSlot, index: number, isEnemy: boolean): string {
    if (slot.gameName) return slot.gameName;
    if (isEnemy) return `Enemy ${index + 1}`;
    return slot.summonerName ?? `Player ${index + 1}`;
  }
</script>

{#snippet banRow(bans: ChampSelectSlot[], team: 'blue' | 'red')}
  <div class="bans-row" class:enemy={team === 'red'}>
    {#each Array(5) as _, index}
      {@const ban = bans[index]}
      <div class="ban-slot">
        {#if ban?.championId}
          <ImageWithFallback 
            src={championImage(ban)} 
            alt={championName(ban) || `Ban ${index + 1}`}
            class="ban-image"
            fallback="?"
          />
        {:else}
          <div class="empty-ban"></div>
        {/if}
      </div>
    {/each}
  </div>
{/snippet}

{#snippet playerRow(pick: ChampSelectSlot, index: number, team: 'blue' | 'red')}
  {@const isEnemy = team === 'red'}
  {@const profile = isEnemy ? scoutProfile(pick) : undefined}
  {@const rank = profile?.primary_rank ?? profile?.solo_rank}
  {@const threat = profile ? getThreatLabel(profile) : null}
  <div
    class="player-row"
    class:enemy={isEnemy}
    class:hover-intent={pick?.isHovered}
    class:active-turn={pick?.isActive}
    class:picking={pick?.isActive && !pick?.championId}
  >
  {#if isEnemy}
    <div class="player-info">
      {#if pick?.position}
        <span class="role-label">{formatPosition(pick.position)}</span>
      {/if}
      {#if pick?.championId}
        <span class="champ-name">{championName(pick) || 'Unknown'}</span>
      {:else if pick?.isActive}
        <span class="picking-label">Picking…</span>
      {:else}
        <span class="picking-label muted">Waiting</span>
      {/if}
      <span class="summoner-name">{displayName(pick, index, isEnemy)}</span>
      {#if rank || threat}
        <div class="scout-badges">
          {#if rank}
            <span class="scout-rank {getRankTierColor(rank.tier)}">
              {#if getRankEmblemUrl(rank.tier)}
                <ImageWithFallback
                  src={getRankEmblemUrl(rank.tier)}
                  alt={rank.tier}
                  class="rank-emblem"
                  fallback={rank.tier.charAt(0)}
                  width={12}
                  height={12}
                />
              {/if}
              {rank.summary}
            </span>
          {/if}
          {#if threat}
            <span class="scout-threat" class:one-trick={profile?.is_one_trick}>
              {#if profile?.is_one_trick}
                <Flame class="threat-icon" />
              {/if}
              {threat}
            </span>
          {/if}
        </div>
      {/if}
    </div>
    <div class="portrait-wrap">
      {#if pick?.championId}
        <ImageWithFallback 
          src={championImage(pick)} 
          alt="Pick {index + 1}" 
          class="portrait"
          fallback="?"
          width={44}
          height={44}
        />
      {:else}
        <div class="empty-portrait">?</div>
      {/if}
    </div>
    <div class="spell-stack">
      {#if pick?.spell1Id}
        <ImageWithFallback src={spellImage(pick.spell1Id)} class="spell" alt="Spell 1" fallback="?" width={18} height={18} />
      {:else}
        <div class="spell empty-spell"></div>
      {/if}
      {#if pick?.spell2Id}
        <ImageWithFallback src={spellImage(pick.spell2Id)} class="spell" alt="Spell 2" fallback="?" width={18} height={18} />
      {:else}
        <div class="spell empty-spell"></div>
      {/if}
    </div>
  {:else}
    <div class="spell-stack">
      {#if pick?.spell1Id}
        <ImageWithFallback src={spellImage(pick.spell1Id)} class="spell" alt="Spell 1" fallback="?" width={18} height={18} />
      {:else}
        <div class="spell empty-spell"></div>
      {/if}
      {#if pick?.spell2Id}
        <ImageWithFallback src={spellImage(pick.spell2Id)} class="spell" alt="Spell 2" fallback="?" width={18} height={18} />
      {:else}
        <div class="spell empty-spell"></div>
      {/if}
    </div>
    <div class="portrait-wrap">
      {#if pick?.championId}
        <ImageWithFallback 
          src={championImage(pick)} 
          alt="Pick {index + 1}" 
          class="portrait"
          fallback="?"
          width={44}
          height={44}
        />
      {:else}
        <div class="empty-portrait">?</div>
      {/if}
    </div>
    <div class="player-info">
      {#if pick?.position}
        <span class="role-label">{formatPosition(pick.position)}</span>
      {/if}
      {#if pick?.championId}
        <span class="champ-name">{championName(pick) || 'Unknown'}</span>
      {:else if pick?.isActive}
        <span class="picking-label">Picking…</span>
      {:else}
        <span class="picking-label muted">Waiting</span>
      {/if}
      <span class="summoner-name">{displayName(pick, index, isEnemy)}</span>
    </div>
  {/if}
  </div>
{/snippet}

<div class="champion-select-overlay">
  {#if !championSelectData}
    <div class="no-data-message">
      <h3>No Champion Select Data</h3>
      <p>Waiting for champion select data from League client...</p>
      <div class="debug-info">
        <p><strong>Debug Info:</strong></p>
        <p>Game Phase: {gamePhase}</p>
        <p>Current Phase: {currentPhase}</p>
        <p>Time Remaining: {timeRemaining}</p>
      </div>
    </div>
  {:else}
    <!-- Blue Team -->
    <div class="team blue-team">
      {@render banRow(blueTeam.bans, 'blue')}

      <div class="picks-list">
        {#each blueTeam.picks as pick, index}
          {@render playerRow(pick, index, 'blue')}
        {/each}
      </div>
    </div>
  
    <!-- Center Timer Section -->
    <div class="timer-section">
      <div class="phase-info">
        <div class="current-phase">{currentPhase.replace('_', ' ').toUpperCase()}</div>
        <div class="phase-timer {timeRemaining < 10 ? 'critical' : ''}">
          {formatTime(timeRemaining)}
        </div>
      </div>
      
      <div class="game-phase-indicator">
        <div class="phase-dot {gamePhase === 'ChampSelect' || gamePhase === 'champion_select' ? 'active' : ''}"></div>
        <span>Champion Select</span>
      </div>
    </div>
  
    <!-- Red Team -->
    <div class="team red-team">
      {@render banRow(redTeam.bans, 'red')}

      <div class="picks-list">
        {#each redTeam.picks as pick, index}
          {@render playerRow(pick, index, 'red')}
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .champion-select-overlay {
    display: flex;
    justify-content: space-between;
    align-items: stretch;
    gap: 1.5rem;
    background: linear-gradient(135deg, var(--bg-primary) 0%, var(--bg-secondary) 100%);
    padding: 1.25rem 1.5rem;
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xl);
    min-height: 360px;
    font-family: 'Saira Condensed', sans-serif;
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    position: relative;
    overflow: hidden;
  }
  
  .no-data-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    text-align: center;
    padding: 2rem;
  }
  
  .no-data-message h3 {
    font-size: 1.5rem;
    margin-bottom: 1rem;
    color: var(--text-primary);
  }
  
  .no-data-message p {
    color: var(--text-secondary);
    margin-bottom: 1rem;
  }
  
  .debug-info {
    background: var(--bg-secondary);
    padding: 1rem;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-color);
    margin-top: 1rem;
    text-align: left;
  }
  
  .debug-info p {
    margin: 0.25rem 0;
    font-size: 0.9rem;
  }
  
  .champion-select-overlay::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    background: linear-gradient(90deg, var(--accent-color), var(--success-color), var(--warning-color), var(--error-color));
    opacity: 0.8;
  }
  
  .team {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    min-width: 0;
    max-width: 320px;
  }

  /* Bans — single horizontal row like in-game */
  .bans-row {
    display: flex;
    flex-wrap: nowrap;
    gap: 4px;
    justify-content: flex-start;
  }

  .bans-row.enemy {
    justify-content: flex-end;
  }

  .ban-slot {
    flex: 0 0 34px;
    width: 34px;
    height: 34px;
    border-radius: 4px;
    overflow: hidden;
    border: 1px solid rgba(74, 144, 226, 0.6);
    background: rgba(0, 0, 0, 0.4);
  }

  .bans-row.enemy .ban-slot {
    border-color: rgba(231, 76, 60, 0.6);
  }

  .ban-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    filter: grayscale(0.3);
  }

  .empty-ban {
    width: 100%;
    height: 100%;
    background: rgba(255, 255, 255, 0.05);
  }

  /* Picks — vertical player rows like in-game */
  .picks-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 1;
  }

  .player-row {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 52px;
    padding: 4px 6px;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid transparent;
    transition: border-color 0.2s, opacity 0.2s;
  }

  .player-row.enemy {
    flex-direction: row;
    justify-content: flex-end;
  }

  .player-row.hover-intent {
    opacity: 0.7;
    border-style: dashed;
    border-color: rgba(255, 255, 255, 0.3);
  }

  .player-row.active-turn {
    border-color: rgba(200, 170, 110, 0.85);
    box-shadow: 0 0 14px rgba(200, 170, 110, 0.25);
    background: rgba(200, 170, 110, 0.08);
  }

  .player-row.active-turn .portrait-wrap {
    border-color: #c8aa6e;
    box-shadow: 0 0 10px rgba(200, 170, 110, 0.45);
  }

  .player-row.picking .empty-portrait {
    animation: pulse-glow 2s ease-in-out infinite;
    border: 2px solid #c8aa6e;
    border-radius: 50%;
  }

  .blue-team .player-row {
    border-left: 2px solid rgba(74, 144, 226, 0.5);
  }

  .red-team .player-row {
    border-right: 2px solid rgba(231, 76, 60, 0.5);
  }

  .spell-stack {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex-shrink: 0;
  }

  .spell {
    width: 18px;
    height: 18px;
    border-radius: 2px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    object-fit: cover;
  }

  .empty-spell {
    background: rgba(255, 255, 255, 0.06);
  }

  .portrait-wrap {
    flex-shrink: 0;
    width: 44px;
    height: 44px;
    border-radius: 50%;
    overflow: hidden;
    border: 2px solid rgba(74, 144, 226, 0.7);
    background: rgba(0, 0, 0, 0.5);
  }

  .red-team .portrait-wrap {
    border-color: rgba(231, 76, 60, 0.7);
  }

  .portrait {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .empty-portrait {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.1rem;
    color: #555;
    font-weight: bold;
  }

  .player-info {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
    flex: 1;
  }

  .player-row.enemy .player-info {
    align-items: flex-end;
    text-align: right;
  }

  .role-label {
    font-size: 0.65rem;
    letter-spacing: 0.08em;
    color: #8a9bb0;
    text-transform: uppercase;
    line-height: 1.2;
  }

  .champ-name {
    font-size: 0.95rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: #e8edf5;
    line-height: 1.2;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .picking-label {
    font-size: 0.8rem;
    color: #c8aa6e;
    font-style: italic;
    line-height: 1.2;
  }

  .picking-label.muted {
    color: #6a8ab0;
  }

  .summoner-name {
    font-size: 0.72rem;
    color: #7a8a9a;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.2;
  }

  .scout-badges {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 2px;
    justify-content: flex-end;
  }

  .player-row:not(.enemy) .scout-badges {
    justify-content: flex-start;
  }

  .scout-rank {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: 0.62rem;
    font-weight: 600;
    letter-spacing: 0.02em;
  }

  :global(.rank-emblem) {
    width: 12px;
    height: 12px;
    object-fit: contain;
    flex-shrink: 0;
  }

  .scout-threat {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    font-size: 0.58rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: #a0a8b8;
    padding: 1px 4px;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.06);
  }

  .scout-threat.one-trick {
    color: #e8954a;
    background: rgba(232, 149, 74, 0.12);
  }

  :global(.threat-icon) {
    width: 10px;
    height: 10px;
  }
  
  .timer-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    min-width: 160px;
    gap: 1rem;
  }
  
  .phase-info {
    text-align: center;
    background: rgba(255, 255, 255, 0.08);
    padding: 1rem 1.5rem;
    border-radius: 12px;
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.08);
  }
  
  .current-phase {
    font-size: 0.8rem;
    color: #aaa;
    margin-bottom: 0.4rem;
    letter-spacing: 0.12em;
  }
  
  .phase-timer {
    font-size: 2.25rem;
    font-weight: bold;
    color: #4a90e2;
    font-family: 'Nova Flat', monospace;
    line-height: 1;
  }
  
  .phase-timer.critical {
    color: #e74c3c;
    animation: pulse 1s infinite;
  }
  
  .game-phase-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.8rem;
    color: #888;
  }
  
  .phase-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #555;
    transition: all 0.3s ease;
  }
  
  .phase-dot.active {
    background: #4a90e2;
    box-shadow: 0 0 10px #4a90e2;
  }
  
  @keyframes pulse {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.08); }
  }

  @keyframes pulse-glow {
    0%, 100% { box-shadow: 0 0 0 0 rgba(200, 170, 110, 0); }
    50% { box-shadow: 0 0 10px 3px rgba(200, 170, 110, 0.45); }
  }
  
  @media (max-width: 900px) {
    .champion-select-overlay {
      flex-direction: column;
      align-items: stretch;
    }
    
    .timer-section {
      order: -1;
      flex-direction: row;
      justify-content: center;
      min-width: auto;
    }

    .team {
      max-width: none;
    }
  }
</style>
