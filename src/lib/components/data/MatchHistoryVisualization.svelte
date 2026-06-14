<script lang="ts">
  import { onMount } from 'svelte';
  import { wsMatchHistory } from '$lib/stores/websocketStore';
  
  let matchHistory: any[] = [];
  let selectedMatch: any = null;
  let stats = {
    totalMatches: 0,
    wins: 0,
    losses: 0,
    winRate: 0,
    avgKDA: { kills: 0, deaths: 0, assists: 0 },
    avgCS: 0,
    avgDamage: 0,
    avgGold: 0
  };
  
  let championStats: Map<string, any> = new Map();
  let roleStats: Map<string, any> = new Map();
  
  onMount(() => {
    // Subscribe to match history updates
    const unsubscribe = wsMatchHistory.subscribe((data: any) => {
      if (data) {
        matchHistory = data.matches || [];
        calculateStats();
      }
    });
    
    return unsubscribe;
  });
  
  function calculateStats() {
    if (matchHistory.length === 0) return;
    
    let totalKills = 0, totalDeaths = 0, totalAssists = 0;
    let totalCS = 0, totalDamage = 0, totalGold = 0;
    
    championStats.clear();
    roleStats.clear();
    
    matchHistory.forEach(match => {
      if (match.stats) {
        if (match.stats.win) stats.wins++;
        else stats.losses++;
        
        totalKills += match.stats.kills || 0;
        totalDeaths += match.stats.deaths || 0;
        totalAssists += match.stats.assists || 0;
        totalCS += match.stats.cs || 0;
        totalDamage += match.stats.damage || 0;
        totalGold += match.stats.gold || 0;
        
        // Champion stats
        const champion = match.championName || 'Unknown';
        if (!championStats.has(champion)) {
          championStats.set(champion, {
            name: champion,
            games: 0,
            wins: 0,
            kills: 0,
            deaths: 0,
            assists: 0,
            cs: 0,
            damage: 0,
            gold: 0
          });
        }
        
        const champStats = championStats.get(champion)!;
        champStats.games++;
        if (match.stats.win) champStats.wins++;
        champStats.kills += match.stats.kills || 0;
        champStats.deaths += match.stats.deaths || 0;
        champStats.assists += match.stats.assists || 0;
        champStats.cs += match.stats.cs || 0;
        champStats.damage += match.stats.damage || 0;
        champStats.gold += match.stats.gold || 0;
        
        // Role stats
        const role = match.role || 'Unknown';
        if (!roleStats.has(role)) {
          roleStats.set(role, {
            name: role,
            games: 0,
            wins: 0,
            kills: 0,
            deaths: 0,
            assists: 0
          });
        }
        
        const roleStat = roleStats.get(role)!;
        roleStat.games++;
        if (match.stats.win) roleStat.wins++;
        roleStat.kills += match.stats.kills || 0;
        roleStat.deaths += match.stats.deaths || 0;
        roleStat.assists += match.stats.assists || 0;
      }
    });
    
    stats.totalMatches = matchHistory.length;
    stats.winRate = stats.totalMatches > 0 ? (stats.wins / stats.totalMatches) * 100 : 0;
    stats.avgKDA = {
      kills: stats.totalMatches > 0 ? totalKills / stats.totalMatches : 0,
      deaths: stats.totalMatches > 0 ? totalDeaths / stats.totalMatches : 0,
      assists: stats.totalMatches > 0 ? totalAssists / stats.totalMatches : 0
    };
    stats.avgCS = stats.totalMatches > 0 ? totalCS / stats.totalMatches : 0;
    stats.avgDamage = stats.totalMatches > 0 ? totalDamage / stats.totalMatches : 0;
    stats.avgGold = stats.totalMatches > 0 ? totalGold / stats.totalMatches : 0;
  }
  
  function selectMatch(match: any) {
    selectedMatch = match;
  }
  
  function getChampionImage(championName: string): string {
    return `https://ddragon.leagueoflegends.com/cdn/13.24.1/img/champion/${championName}.png`;
  }
  
  function getItemImage(itemId: number): string {
    return `https://ddragon.leagueoflegends.com/cdn/13.24.1/img/item/${itemId}.png`;
  }
  
  function formatKDA(kills: number, deaths: number, assists: number): string {
    return `${kills}/${deaths}/${assists}`;
  }
  
  function formatNumber(num: number): string {
    if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
    if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
    return num.toFixed(0);
  }
  
  function getWinRateColor(winRate: number): string {
    if (winRate >= 60) return 'var(--success-color)';
    if (winRate >= 50) return 'var(--warning-color)';
    return 'var(--error-color)';
  }
</script>

<div class="match-history-visualization">
  <!-- Stats Overview -->
  <div class="stats-overview">
    <div class="stat-card">
      <div class="stat-value">{stats.totalMatches}</div>
      <div class="stat-label">Total Matches</div>
    </div>
    
    <div class="stat-card">
      <div class="stat-value" style="color: {getWinRateColor(stats.winRate)}">
        {stats.winRate.toFixed(1)}%
      </div>
      <div class="stat-label">Win Rate</div>
    </div>
    
    <div class="stat-card">
      <div class="stat-value">
        {formatKDA(stats.avgKDA.kills, stats.avgKDA.deaths, stats.avgKDA.assists)}
      </div>
      <div class="stat-label">Avg KDA</div>
    </div>
    
    <div class="stat-card">
      <div class="stat-value">{stats.avgCS.toFixed(0)}</div>
      <div class="stat-label">Avg CS</div>
    </div>
    
    <div class="stat-card">
      <div class="stat-value">{formatNumber(stats.avgDamage)}</div>
      <div class="stat-label">Avg Damage</div>
    </div>
    
    <div class="stat-card">
      <div class="stat-value">{formatNumber(stats.avgGold)}</div>
      <div class="stat-label">Avg Gold</div>
    </div>
  </div>
  
  <!-- Champion Performance -->
  <div class="champion-performance">
    <h3>Champion Performance</h3>
    <div class="champion-grid">
      {#each Array.from(championStats.values()).slice(0, 6) as champion}
        <div class="champion-card">
          <img src={getChampionImage(champion.name)} class="champion-image" alt={champion.name} />
          <div class="champion-info">
            <div class="champion-name">{champion.name}</div>
            <div class="champion-stats">
              <div class="champion-games">{champion.games} games</div>
              <div class="champion-winrate" style="color: {getWinRateColor((champion.wins / champion.games) * 100)}">
                {((champion.wins / champion.games) * 100).toFixed(0)}% WR
              </div>
            </div>
            <div class="champion-kda">
              {formatKDA(champion.kills / champion.games, champion.deaths / champion.games, champion.assists / champion.games)}
            </div>
          </div>
        </div>
      {/each}
    </div>
  </div>
  
  <!-- Role Performance -->
  <div class="role-performance">
    <h3>Role Performance</h3>
    <div class="role-grid">
      {#each Array.from(roleStats.values()) as role}
        <div class="role-card">
          <div class="role-icon">🎯</div>
          <div class="role-info">
            <div class="role-name">{role.name}</div>
            <div class="role-stats">
              <div class="role-games">{role.games} games</div>
              <div class="role-winrate" style="color: {getWinRateColor((role.wins / role.games) * 100)}">
                {((role.wins / role.games) * 100).toFixed(0)}% WR
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  </div>
  
  <!-- Match History List -->
  <div class="match-history-list">
    <h3>Recent Matches</h3>
    <div class="matches-container">
      {#each matchHistory.slice(0, 10) as match}
        <div 
          class="match-card {match.stats?.win ? 'win' : 'loss'}" 
          role="button"
          tabindex="0"
          on:click={() => selectMatch(match)}
          on:keydown={(e) => e.key === 'Enter' && selectMatch(match)}
        >
          <div class="match-result">
            <div class="result-text">{match.stats?.win ? 'VICTORY' : 'DEFEAT'}</div>
            <div class="match-duration">{match.duration || '0:00'}</div>
          </div>
          
          <div class="match-champion">
            <img src={getChampionImage(match.championName || 'Unknown')} class="champion-icon" alt={match.championName} />
            <div class="champion-details">
              <div class="champion-name">{match.championName || 'Unknown'}</div>
              <div class="match-role">{match.role || 'Unknown'}</div>
            </div>
          </div>
          
          <div class="match-stats">
            <div class="kda">
              {formatKDA(match.stats?.kills || 0, match.stats?.deaths || 0, match.stats?.assists || 0)}
            </div>
            <div class="cs">{match.stats?.cs || 0} CS</div>
            <div class="damage">{formatNumber(match.stats?.damage || 0)} DMG</div>
          </div>
          
          <div class="match-items">
            {#each (match.items || []).slice(0, 3) as item}
              <img src={getItemImage(item)} class="item-icon" alt="Item" />
            {/each}
          </div>
        </div>
      {/each}
    </div>
  </div>
  
  <!-- Match Details Modal -->
  {#if selectedMatch}
    <div 
      class="match-details-modal" 
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
      tabindex="-1"
      on:click={() => selectedMatch = null}
      on:keydown={(e) => e.key === 'Escape' && (selectedMatch = null)}
    >
      <div class="modal-content" role="document">
        <div class="modal-header">
          <h2 id="modal-title">Match Details</h2>
          <button class="close-btn" on:click={() => selectedMatch = null} aria-label="Close modal">×</button>
        </div>
        
        <div class="modal-body">
          <div class="match-overview">
            <div class="match-result-large {selectedMatch.stats?.win ? 'win' : 'loss'}">
              {selectedMatch.stats?.win ? 'VICTORY' : 'DEFEAT'}
            </div>
            <div class="match-info">
              <div class="champion-large">
                <img src={getChampionImage(selectedMatch.championName || 'Unknown')} class="champion-image-large" alt={selectedMatch.championName || 'Unknown'} />
                <div class="champion-name-large">{selectedMatch.championName || 'Unknown'}</div>
              </div>
              <div class="match-stats-large">
                <div class="kda-large">
                  {formatKDA(selectedMatch.stats?.kills || 0, selectedMatch.stats?.deaths || 0, selectedMatch.stats?.assists || 0)}
                </div>
                <div class="stats-grid">
                  <div class="stat-item">
                    <span class="stat-label">CS</span>
                    <span class="stat-value">{selectedMatch.stats?.cs || 0}</span>
                  </div>
                  <div class="stat-item">
                    <span class="stat-label">Damage</span>
                    <span class="stat-value">{formatNumber(selectedMatch.stats?.damage || 0)}</span>
                  </div>
                  <div class="stat-item">
                    <span class="stat-label">Gold</span>
                    <span class="stat-value">{formatNumber(selectedMatch.stats?.gold || 0)}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .match-history-visualization {
    display: flex;
    flex-direction: column;
    gap: 2rem;
    padding: 1.5rem;
    background: var(--bg-primary);
    color: var(--text-primary);
  }
  
  .stats-overview {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 1rem;
  }
  
  .stat-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: 1.5rem;
    text-align: center;
    transition: var(--transition-normal);
  }
  
  .stat-card:hover {
    border-color: var(--border-hover);
    box-shadow: var(--shadow-lg);
  }
  
  .stat-value {
    font-size: 2rem;
    font-weight: var(--font-weight-bold);
    color: var(--accent-color);
    margin-bottom: 0.5rem;
  }
  
  .stat-label {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 1px;
  }
  
  .champion-performance,
  .role-performance,
  .match-history-list {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: 1.5rem;
  }
  
  .champion-performance h3,
  .role-performance h3,
  .match-history-list h3 {
    margin: 0 0 1.5rem 0;
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-semibold);
  }
  
  .champion-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
  }
  
  .champion-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    transition: var(--transition-normal);
  }
  
  .champion-card:hover {
    border-color: var(--border-hover);
    box-shadow: var(--shadow-md);
  }
  
  .champion-image {
    width: 50px;
    height: 50px;
    border-radius: var(--radius-md);
    border: 2px solid var(--border-color);
  }
  
  .champion-info {
    flex: 1;
  }
  
  .champion-name {
    font-weight: var(--font-weight-semibold);
    margin-bottom: 0.25rem;
  }
  
  .champion-stats {
    display: flex;
    gap: 1rem;
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }
  
  .champion-kda {
    font-size: var(--font-size-sm);
    color: var(--text-tertiary);
  }
  
  .role-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 1rem;
  }
  
  .role-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    transition: var(--transition-normal);
  }
  
  .role-card:hover {
    border-color: var(--border-hover);
    box-shadow: var(--shadow-md);
  }
  
  .role-icon {
    font-size: 2rem;
  }
  
  .role-info {
    flex: 1;
  }
  
  .role-name {
    font-weight: var(--font-weight-semibold);
    margin-bottom: 0.25rem;
  }
  
  .role-stats {
    display: flex;
    gap: 1rem;
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }
  
  .matches-container {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .match-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: var(--transition-normal);
  }
  
  .match-card:hover {
    border-color: var(--border-hover);
    box-shadow: var(--shadow-md);
  }
  
  .match-card.win {
    border-left: 4px solid var(--success-color);
  }
  
  .match-card.loss {
    border-left: 4px solid var(--error-color);
  }
  
  .match-result {
    text-align: center;
    min-width: 80px;
  }
  
  .result-text {
    font-weight: var(--font-weight-bold);
    font-size: var(--font-size-sm);
    margin-bottom: 0.25rem;
  }
  
  .match-card.win .result-text {
    color: var(--success-color);
  }
  
  .match-card.loss .result-text {
    color: var(--error-color);
  }
  
  .match-duration {
    font-size: var(--font-size-xs);
    color: var(--text-tertiary);
  }
  
  .match-champion {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    min-width: 150px;
  }
  
  .champion-icon {
    width: 40px;
    height: 40px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-color);
  }
  
  .champion-details {
    flex: 1;
  }
  
  .champion-name {
    font-weight: var(--font-weight-medium);
    margin-bottom: 0.25rem;
  }
  
  .match-role {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }
  
  .match-stats {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 100px;
  }
  
  .kda {
    font-weight: var(--font-weight-semibold);
    color: var(--accent-color);
  }
  
  .cs,
  .damage {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }
  
  .match-items {
    display: flex;
    gap: 2px;
  }
  
  .item-icon {
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-color);
  }
  
  /* Modal Styles */
  .match-details-modal {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: var(--z-modal);
  }
  
  .modal-content {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    max-width: 500px;
    width: 90%;
    max-height: 80vh;
    overflow: auto;
  }
  
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.5rem;
    border-bottom: 1px solid var(--border-color);
  }
  
  .modal-header h2 {
    margin: 0;
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-semibold);
  }
  
  .close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 0.25rem;
    border-radius: var(--radius-sm);
    transition: var(--transition-normal);
  }
  
  .close-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  
  .modal-body {
    padding: 1.5rem;
  }
  
  .match-overview {
    text-align: center;
  }
  
  .match-result-large {
    font-size: 2rem;
    font-weight: var(--font-weight-bold);
    margin-bottom: 1rem;
  }
  
  .match-result-large.win {
    color: var(--success-color);
  }
  
  .match-result-large.loss {
    color: var(--error-color);
  }
  
  .match-info {
    display: flex;
    align-items: center;
    justify-content: space-around;
    gap: 2rem;
  }
  
  .champion-large {
    text-align: center;
  }
  
  .champion-image-large {
    width: 80px;
    height: 80px;
    border-radius: var(--radius-md);
    border: 2px solid var(--border-color);
    margin-bottom: 0.5rem;
  }
  
  .champion-name-large {
    font-weight: var(--font-weight-semibold);
  }
  
  .match-stats-large {
    text-align: center;
  }
  
  .kda-large {
    font-size: 1.5rem;
    font-weight: var(--font-weight-bold);
    color: var(--accent-color);
    margin-bottom: 1rem;
  }
  
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }
  
  .stat-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  
  .stat-item .stat-label {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }
  
  .stat-item .stat-value {
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
  }
  
  /* Responsive Design */
  @media (max-width: 768px) {
    .match-card {
      flex-direction: column;
      align-items: flex-start;
      gap: 0.75rem;
    }
    
    .match-champion {
      min-width: auto;
    }
    
    .match-stats {
      min-width: auto;
    }
    
    .match-info {
      flex-direction: column;
      gap: 1rem;
    }
    
    .stats-grid {
      grid-template-columns: repeat(2, 1fr);
    }
  }
</style>
