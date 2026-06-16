import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export const DEFAULT_MATCH_COUNT = 20;
export const MATCH_COUNT_STEP = 20;
export const MAX_MATCH_COUNT = 100;

// Match History State
export const matchHistoryData = writable<Record<string, any>>({});
export const currentPlayerStats = writable<any>(null);
export const championPerformance = writable<Record<string, any>>({});
export const recentMatches = writable<any[]>([]);
export const matchHistoryMatchCount = writable(DEFAULT_MATCH_COUNT);
export const matchHistoryLoading = writable(false);

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

      const requested =
        analysis.requested_match_count ?? analysis.total_matches ?? DEFAULT_MATCH_COUNT;
      matchHistoryMatchCount.set(
        Math.min(Math.max(requested, DEFAULT_MATCH_COUNT), MAX_MATCH_COUNT),
      );
      
      // Extract champion performance
      if (analysis.champion_performance) {
        championPerformance.set(analysis.champion_performance);
      }
      
      if (analysis.raw_matches && Array.isArray(analysis.raw_matches)) {
        recentMatches.set(analysis.raw_matches);
      }
    }
  },

  fetchAnalysis: async (options?: {
    matchCount?: number;
    forceRefresh?: boolean;
    puuid?: string;
  }) => {
    const matchCount = options?.matchCount ?? get(matchHistoryMatchCount);
    matchHistoryLoading.set(true);
    try {
      const analysis = await invoke('get_player_match_analysis', {
        puuid: options?.puuid ?? null,
        forceRefresh: options?.forceRefresh ?? false,
        matchCount,
      });
      matchHistoryActions.updateMatchHistory('current', analysis);
      return analysis;
    } finally {
      matchHistoryLoading.set(false);
    }
  },

  loadMoreMatches: async () => {
    const current = get(matchHistoryMatchCount);
    if (current >= MAX_MATCH_COUNT) return null;
    const next = Math.min(current + MATCH_COUNT_STEP, MAX_MATCH_COUNT);
    return matchHistoryActions.fetchAnalysis({ matchCount: next });
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

  // Get champions sorted by performance (optional limit for compact views)
  getTopChampions: (championPerformance: Record<string, any>, limit?: number) => {
    const sorted = Object.entries(championPerformance)
      .map(([championId, data]) => {
        const games = data.games || 0;
        const deaths = data.deaths || 0;
        return {
          championId,
          championName: data.champion_name || data.name || `Champion ${championId}`,
          ...data,
          winRate: games > 0 ? (data.wins / games) * 100 : 0,
          avgKDA: games > 0 ? ((data.kills + data.assists) / games) / Math.max(deaths / games, 1) : 0,
          avgImpact: data.avg_impact ?? (games > 0 ? (data.impact_total || 0) / games : 0),
        };
      })
      .sort((a, b) => {
        if (b.avgImpact !== a.avgImpact) return b.avgImpact - a.avgImpact;
        if (b.winRate !== a.winRate) return b.winRate - a.winRate;
        return b.games - a.games;
      });
    return limit != null ? sorted.slice(0, limit) : sorted;
  },

  // Get recent performance trend
  getPerformanceTrend: (recentMatches: any[]) => {
    if (recentMatches.length < 3) return { trend: 'insufficient', color: 'text-gray-600' };
    
    const last5 = recentMatches.slice(0, 5);
    const wins = last5.filter(match => match.stats?.win).length;
    
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
        avgCS: 0,
        avgDamage: 0,
        avgImpact: 0,
        avgVision: 0,
        performanceGrade: { grade: 'N/A', color: 'text-gray-600' },
        topChampions: [],
        allChampions: [],
        recentTrend: { trend: 'insufficient', color: 'text-gray-600' },
        roleStats: {},
        hasMoreMatches: false,
        matchCountLoaded: DEFAULT_MATCH_COUNT,
      };
    }

    const totalMatches = stats.total_matches || 0;
    const winRate = stats.win_rate || 0;
    const avgKDA = totalMatches > 0 ? 
      ((stats.avg_kills + stats.avg_assists) / Math.max(stats.avg_deaths, 1)) : 0;

    const allChampions = matchHistoryActions.getTopChampions(champions);
    
    return {
      totalMatches,
      winRate: Math.round(winRate),
      avgKDA: Math.round(avgKDA * 100) / 100,
      avgCS: Math.round((stats.avg_cs || 0) * 10) / 10,
      avgDamage: Math.round(stats.avg_damage || 0),
      avgImpact: Math.round((stats.avg_impact || 0) * 10) / 10,
      avgVision: Math.round((stats.avg_vision || 0) * 10) / 10,
      performanceGrade: matchHistoryActions.getPerformanceGrade(winRate, avgKDA),
      topChampions: matchHistoryActions.getTopChampions(champions, 5),
      allChampions,
      recentTrend: matchHistoryActions.getPerformanceTrend(matches),
      roleStats: stats.role_stats || {},
      hasMoreMatches: stats.has_more_matches ?? false,
      matchCountLoaded: stats.requested_match_count ?? totalMatches,
    };
  }
);

// Current player performance insights (backend insights + legacy fallbacks)
export const playerInsights = derived(
  [currentPlayerStats, championPerformance],
  ([stats, champions]) => {
    if (!stats) return [];

    if (Array.isArray(stats.insights) && stats.insights.length > 0) {
      return stats.insights;
    }

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
