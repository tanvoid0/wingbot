import { writable, derived } from 'svelte/store';

// Match History State
export const matchHistoryData = writable<Record<string, any>>({});
export const currentPlayerStats = writable<any>(null);
export const championPerformance = writable<Record<string, any>>({});
export const recentMatches = writable<any[]>([]);

// Match History Actions
export const matchHistoryActions = {
  // Update match history data
  updateMatchHistory: (playerId: string, analysis: any) => {
    matchHistoryData.update(current => ({
      ...current,
      [playerId]: analysis
    }));

    // Update current player stats if this is the current player
    if (playerId === 'current') {
      currentPlayerStats.set(analysis);
      
      // Extract champion performance
      if (analysis.champion_performance) {
        championPerformance.set(analysis.champion_performance);
      }
      
      // Extract recent matches (last 10)
      if (analysis.raw_matches && Array.isArray(analysis.raw_matches)) {
        recentMatches.set(analysis.raw_matches.slice(0, 10));
      }
    }
  },

  // Get win rate color
  getWinRateColor: (winRate: number) => {
    if (winRate >= 60) return 'text-green-600';
    if (winRate >= 50) return 'text-yellow-600';
    if (winRate >= 40) return 'text-orange-600';
    return 'text-red-600';
  },

  // Get KDA color
  getKDAColor: (kda: number) => {
    if (kda >= 3.0) return 'text-green-600';
    if (kda >= 2.0) return 'text-yellow-600';
    if (kda >= 1.5) return 'text-orange-600';
    return 'text-red-600';
  },

  // Format KDA
  formatKDA: (kills: number, deaths: number, assists: number) => {
    const kda = deaths > 0 ? ((kills + assists) / deaths).toFixed(2) : (kills + assists).toFixed(2);
    return `${kills}/${deaths}/${assists} (${kda})`;
  },

  // Get performance grade
  getPerformanceGrade: (winRate: number, avgKDA: number) => {
    if (winRate >= 60 && avgKDA >= 2.5) return { grade: 'S', color: 'text-purple-600' };
    if (winRate >= 55 && avgKDA >= 2.0) return { grade: 'A', color: 'text-green-600' };
    if (winRate >= 50 && avgKDA >= 1.5) return { grade: 'B', color: 'text-yellow-600' };
    if (winRate >= 45 && avgKDA >= 1.0) return { grade: 'C', color: 'text-orange-600' };
    return { grade: 'D', color: 'text-red-600' };
  },

  // Get champion mastery level
  getChampionMastery: (championData: any) => {
    if (!championData) return { level: 0, text: 'Beginner' };
    
    const games = championData.games || 0;
    const wins = championData.wins || 0;
    const winRate = games > 0 ? (wins / games) * 100 : 0;
    
    if (games >= 50 && winRate >= 60) return { level: 5, text: 'Master' };
    if (games >= 30 && winRate >= 55) return { level: 4, text: 'Expert' };
    if (games >= 20 && winRate >= 50) return { level: 3, text: 'Skilled' };
    if (games >= 10 && winRate >= 45) return { level: 2, text: 'Competent' };
    if (games >= 5) return { level: 1, text: 'Learning' };
    return { level: 0, text: 'Beginner' };
  },

  // Get top performing champions
  getTopChampions: (championPerformance: Record<string, any>, limit: number = 5) => {
    return Object.entries(championPerformance)
      .map(([championId, data]) => ({
        championId,
        ...data,
        winRate: data.games > 0 ? (data.wins / data.games) * 100 : 0,
        avgKDA: data.games > 0 ? ((data.kills + data.assists) / data.games) / Math.max(data.deaths / data.games, 1) : 0
      }))
      .sort((a, b) => {
        // Sort by win rate first, then by games played
        if (b.winRate !== a.winRate) return b.winRate - a.winRate;
        return b.games - a.games;
      })
      .slice(0, limit);
  },

  // Get recent performance trend
  getPerformanceTrend: (recentMatches: any[]) => {
    if (recentMatches.length < 5) return { trend: 'insufficient', color: 'text-gray-600' };
    
    const last5 = recentMatches.slice(0, 5);
    const wins = last5.filter(match => {
      // Find player in participants and check if they won
      const participants = match.participants || [];
      const player = participants.find((p: any) => p.puuid);
      return player && player.stats && player.stats.win;
    }).length;
    
    const winRate = (wins / 5) * 100;
    
    if (winRate >= 80) return { trend: 'excellent', color: 'text-green-600' };
    if (winRate >= 60) return { trend: 'good', color: 'text-blue-600' };
    if (winRate >= 40) return { trend: 'average', color: 'text-yellow-600' };
    return { trend: 'poor', color: 'text-red-600' };
  }
};

// Derived stores
export const matchHistorySummary = derived(
  [currentPlayerStats, championPerformance, recentMatches],
  ([stats, champions, matches]) => {
    if (!stats) {
      return {
        totalMatches: 0,
        winRate: 0,
        avgKDA: 0,
        performanceGrade: { grade: 'N/A', color: 'text-gray-600' },
        topChampions: [],
        recentTrend: { trend: 'insufficient', color: 'text-gray-600' }
      };
    }

    const totalMatches = stats.total_matches || 0;
    const winRate = stats.win_rate || 0;
    const avgKDA = totalMatches > 0 ? 
      ((stats.avg_kills + stats.avg_assists) / Math.max(stats.avg_deaths, 1)) : 0;
    
    return {
      totalMatches,
      winRate: Math.round(winRate),
      avgKDA: Math.round(avgKDA * 100) / 100,
      performanceGrade: matchHistoryActions.getPerformanceGrade(winRate, avgKDA),
      topChampions: matchHistoryActions.getTopChampions(champions, 5),
      recentTrend: matchHistoryActions.getPerformanceTrend(matches)
    };
  }
);

// Current player performance insights
export const playerInsights = derived(
  [currentPlayerStats, championPerformance],
  ([stats, champions]) => {
    if (!stats) return [];

    const insights = [];
    
    // Win rate insight
    const winRate = stats.win_rate || 0;
    if (winRate >= 60) {
      insights.push({
        type: 'positive',
        title: 'Strong Performance',
        message: `Excellent ${winRate.toFixed(1)}% win rate!`
      });
    } else if (winRate < 40) {
      insights.push({
        type: 'negative',
        title: 'Improvement Needed',
        message: `Win rate of ${winRate.toFixed(1)}% suggests room for improvement.`
      });
    }

    // KDA insight
    const avgKDA = (stats.avg_kills + stats.avg_assists) / Math.max(stats.avg_deaths, 1);
    if (avgKDA >= 2.5) {
      insights.push({
        type: 'positive',
        title: 'Excellent KDA',
        message: `Outstanding ${avgKDA.toFixed(2)} KDA ratio!`
      });
    } else if (avgKDA < 1.0) {
      insights.push({
        type: 'negative',
        title: 'KDA Concerns',
        message: `KDA of ${avgKDA.toFixed(2)} indicates need for better positioning.`
      });
    }

    // Champion diversity insight
    const championCount = Object.keys(champions).length;
    if (championCount >= 10) {
      insights.push({
        type: 'neutral',
        title: 'Champion Diversity',
        message: `Playing ${championCount} different champions shows good adaptability.`
      });
    } else if (championCount <= 3) {
      insights.push({
        type: 'warning',
        title: 'Limited Pool',
        message: `Only ${championCount} champions played. Consider expanding your champion pool.`
      });
    }

    return insights;
  }
);
