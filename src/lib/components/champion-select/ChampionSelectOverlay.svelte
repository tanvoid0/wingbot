<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { wsChampionSelect, wsGamePhase } from '$lib/stores/websocketStore';
  import { championSelectStore } from '$lib/stores/championSelectStore';
  import { getChampionImageUrlSync, getSummonerSpellImageUrlSync } from '$lib/services/imageService';
  import ImageWithFallback from '$lib/components/ui/ImageWithFallback.svelte';
  
  let championSelectData: any = null;
  let gamePhase: string = 'unknown';
  let timeRemaining: number = 0;
  let currentPhase: string = 'unknown';
  
  // Team data
  let blueTeam: {
    name: string;
    score: number;
    bans: any[];
    picks: any[];
    coach: string;
  } = {
    name: 'Blue Team',
    score: 0,
    bans: [],
    picks: [],
    coach: ''
  };
  
  let redTeam: {
    name: string;
    score: number;
    bans: any[];
    picks: any[];
    coach: string;
  } = {
    name: 'Red Team', 
    score: 0,
    bans: [],
    picks: [],
    coach: ''
  };
  
  let unsubscribe: (() => void)[] = [];
  
  onMount(() => {
    // Subscribe to champion select updates
    const unsub1 = wsChampionSelect.subscribe((data: any) => {
      if (data) {
        championSelectData = data;
        updateTeamData();
      }
    });
    
    const unsub2 = wsGamePhase.subscribe(phase => {
      gamePhase = phase;
    });
    
    unsubscribe = [unsub1, unsub2];
  });
  
  onDestroy(() => {
    unsubscribe.forEach(unsub => unsub());
  });
  
  function updateTeamData() {
    if (!championSelectData) return;
    
    console.log('Champion Select Data:', championSelectData);
    
    // Extract team data from champion select session
    // LCU API provides teams in the 'teams' array with teamId 100 (blue) and 200 (red)
    if (championSelectData.teams) {
      const blueTeamData = championSelectData.teams.find((team: any) => team.teamId === 100);
      const redTeamData = championSelectData.teams.find((team: any) => team.teamId === 200);
      
      if (blueTeamData) {
        blueTeam = {
          name: 'Your Team',
          score: 0,
          bans: blueTeamData.bans || [],
          picks: blueTeamData.picks || [],
          coach: ''
        };
      }
      
      if (redTeamData) {
        redTeam = {
          name: 'Enemy Team',
          score: 0,
          bans: redTeamData.bans || [],
          picks: redTeamData.picks || [],
          coach: ''
        };
      }
    }
    
    // Handle alternative data structure (myTeam/theirTeam)
    if (championSelectData.myTeam) {
      blueTeam = {
        name: 'Your Team',
        score: 0,
        bans: championSelectData.bans?.filter((ban: any) => ban.teamId === 100) || [],
        picks: championSelectData.myTeam || [],
        coach: ''
      };
    }
    
    if (championSelectData.theirTeam) {
      redTeam = {
        name: 'Enemy Team',
        score: 0,
        bans: championSelectData.bans?.filter((ban: any) => ban.teamId === 200) || [],
        picks: championSelectData.theirTeam || [],
        coach: ''
      };
    }
    
    // Update timer and phase
    if (championSelectData.timer) {
      timeRemaining = championSelectData.timer.adjustedTimeLeftInPhase || 0;
      currentPhase = championSelectData.timer.phase || 'unknown';
    }
  }
  
  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
  
  function getChampionImageUrl(championName: string): string {
    return getChampionImageUrlSync(championName) || '';
  }
  
  function getSpellImageUrl(spellName: string): string {
    return getSummonerSpellImageUrlSync(spellName) || '';
  }
</script>

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
    <div class="team-header">
      <h2 class="team-name">{blueTeam.name}</h2>
      <div class="team-score">{blueTeam.score}</div>
    </div>
    
    <!-- Bans Section -->
    <div class="bans-section">
      <h3 class="section-title">Bans</h3>
      <div class="bans-container">
        {#each blueTeam.bans as ban, index}
          <div class="ban-slot">
            {#if ban.championName}
              <ImageWithFallback 
                src={getChampionImageUrl(ban.championName)} 
                alt="Ban {index + 1}" 
                class="ban-image"
                fallback="?"
              />
            {:else}
              <div class="empty-slot">?</div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
    
    <!-- Picks Section -->
    <div class="picks-section">
      <h3 class="section-title">Picks</h3>
      <div class="picks-container">
        {#each blueTeam.picks as pick, index}
          <div class="pick-slot">
            {#if pick.championName}
              <ImageWithFallback 
                src={getChampionImageUrl(pick.championName)} 
                alt="Pick {index + 1}" 
                class="pick-image"
                fallback="?"
              />
              <div class="summoner-spells">
                {#if pick.spell1Name}
                  <ImageWithFallback 
                    src={getSpellImageUrl(pick.spell1Name)} 
                    class="spell" 
                    alt="Spell 1" 
                    fallback="?"
                  />
                {/if}
                {#if pick.spell2Name}
                  <ImageWithFallback 
                    src={getSpellImageUrl(pick.spell2Name)} 
                    class="spell" 
                    alt="Spell 2" 
                    fallback="?"
                  />
                {/if}
              </div>
            {:else}
              <div class="empty-slot">?</div>
            {/if}
          </div>
        {/each}
      </div>
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
    
    <!-- Game Phase Indicator -->
    <div class="game-phase-indicator">
      <div class="phase-dot {gamePhase === 'champion_select' ? 'active' : ''}"></div>
      <span>Champion Select</span>
    </div>
  </div>
  
  <!-- Red Team -->
  <div class="team red-team">
    <div class="team-header">
      <h2 class="team-name">{redTeam.name}</h2>
      <div class="team-score">{redTeam.score}</div>
    </div>
    
    <!-- Bans Section -->
    <div class="bans-section">
      <h3 class="section-title">Bans</h3>
      <div class="bans-container">
        {#each redTeam.bans as ban, index}
          <div class="ban-slot">
            {#if ban.championName}
              <ImageWithFallback 
                src={getChampionImageUrl(ban.championName)} 
                alt="Ban {index + 1}" 
                class="ban-image"
                fallback="?"
              />
            {:else}
              <div class="empty-slot">?</div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
    
    <!-- Picks Section -->
    <div class="picks-section">
      <h3 class="section-title">Picks</h3>
      <div class="picks-container">
        {#each redTeam.picks as pick, index}
          <div class="pick-slot">
            {#if pick.championName}
              <ImageWithFallback 
                src={getChampionImageUrl(pick.championName)} 
                alt="Pick {index + 1}" 
                class="pick-image"
                fallback="?"
              />
              <div class="summoner-spells">
                {#if pick.spell1Name}
                  <ImageWithFallback 
                    src={getSpellImageUrl(pick.spell1Name)} 
                    class="spell" 
                    alt="Spell 1" 
                    fallback="?"
                  />
                {/if}
                {#if pick.spell2Name}
                  <ImageWithFallback 
                    src={getSpellImageUrl(pick.spell2Name)} 
                    class="spell" 
                    alt="Spell 2" 
                    fallback="?"
                  />
                {/if}
              </div>
            {:else}
              <div class="empty-slot">?</div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  </div>
  {/if}
</div>

<style>
  .champion-select-overlay {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    background: linear-gradient(135deg, var(--bg-primary) 0%, var(--bg-secondary) 100%);
    padding: 2rem;
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xl);
    min-height: 400px;
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
    gap: 1.5rem;
    max-width: 300px;
  }
  
  .team-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-color);
    transition: var(--transition-normal);
    position: relative;
  }
  
  .team-header:hover {
    border-color: var(--border-hover);
    box-shadow: var(--shadow-md);
  }
  
  .team-name {
    font-size: 1.5rem;
    font-weight: bold;
    margin: 0;
  }
  
  .team-score {
    font-size: 2rem;
    font-weight: bold;
    color: #4a90e2;
  }
  
  .blue-team .team-score {
    color: #4a90e2;
  }
  
  .red-team .team-score {
    color: #e74c3c;
  }
  
  .section-title {
    font-size: 1rem;
    margin: 0 0 0.5rem 0;
    color: #ccc;
    text-transform: uppercase;
    letter-spacing: 1px;
  }
  
  .bans-container, .picks-container {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  
  .ban-slot, .pick-slot {
    position: relative;
    width: 60px;
    height: 60px;
    border-radius: 8px;
    overflow: hidden;
    border: 2px solid #333;
    background: rgba(255, 255, 255, 0.1);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .blue-team .ban-slot,
  .blue-team .pick-slot {
    border-color: #4a90e2;
  }
  
  .red-team .ban-slot,
  .red-team .pick-slot {
    border-color: #e74c3c;
  }
  
  .ban-image, .pick-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  
  .empty-slot {
    font-size: 1.5rem;
    color: #666;
    font-weight: bold;
  }
  
  .summoner-spells {
    position: absolute;
    bottom: -2px;
    right: -2px;
    display: flex;
    gap: 1px;
  }
  
  .spell {
    width: 16px;
    height: 16px;
    border-radius: 2px;
    border: 1px solid #333;
  }
  
  .timer-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-width: 200px;
    gap: 1rem;
  }
  
  .phase-info {
    text-align: center;
    background: rgba(255, 255, 255, 0.1);
    padding: 1rem 2rem;
    border-radius: 12px;
    backdrop-filter: blur(10px);
  }
  
  .current-phase {
    font-size: 0.9rem;
    color: #ccc;
    margin-bottom: 0.5rem;
    letter-spacing: 1px;
  }
  
  .phase-timer {
    font-size: 2.5rem;
    font-weight: bold;
    color: #4a90e2;
    font-family: 'Nova Flat', monospace;
  }
  
  .phase-timer.critical {
    color: #e74c3c;
    animation: pulse 1s infinite;
  }
  
  .game-phase-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.9rem;
    color: #ccc;
  }
  
  .phase-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #666;
    transition: all 0.3s ease;
  }
  
  .phase-dot.active {
    background: #4a90e2;
    box-shadow: 0 0 10px #4a90e2;
  }
  
  @keyframes pulse {
    0%, 100% {
      transform: scale(1);
    }
    50% {
      transform: scale(1.1);
    }
  }
  
  /* Responsive design */
  @media (max-width: 1200px) {
    .champion-select-overlay {
      flex-direction: column;
      gap: 2rem;
    }
    
    .timer-section {
      order: -1;
      flex-direction: row;
      min-width: auto;
    }
    
    .team {
      max-width: none;
    }
  }
</style>
