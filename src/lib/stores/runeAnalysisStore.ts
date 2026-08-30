import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { isOllamaFeatureEnabled } from './ollamaStatusStore';

// Rune Analysis State
export const allPlayersRunes = writable<Record<string, any>>({});
export const runeAnalysisData = writable<Record<string, any>>({});
export const runeRecommendations = writable<Record<string, any[]>>({});
export const currentPlayerRunes = writable<any>(null);
export const runeAiInsights = writable<any[]>([]);
export const runeAiInsightsLoading = writable(false);
export const runeAiInsightsError = writable<string | null>(null);

let runeAiCacheKey = '';

// Rune Analysis Actions
export const runeAnalysisActions = {
  // Update all players' runes
  updateAllPlayersRunes: (runes: Record<string, any>) => {
    allPlayersRunes.set(runes);
  },

  // Update rune analysis for a specific player
  updateRuneAnalysis: (playerId: string, analysis: any) => {
    runeAnalysisData.update(current => ({
      ...current,
      [playerId]: analysis
    }));

    if (playerId === 'current' || playerId === 'me') {
      currentPlayerRunes.set(analysis);
      void runeAnalysisActions.fetchAiInsights(analysis);
    }
  },

  buildAiCacheKey: (analysis: any) => {
    return [
      analysis?.primary_tree,
      analysis?.secondary_tree,
      analysis?.keystone,
      analysis?.total_runes,
    ].join(':');
  },

  fetchAiInsights: async (analysis?: any, options?: { force?: boolean }) => {
    const runeAnalysis = analysis ?? get(currentPlayerRunes);
    if (!runeAnalysis || !isOllamaFeatureEnabled('rune')) {
      runeAiInsights.set([]);
      runeAiInsightsError.set(null);
      return [];
    }

    const cacheKey = runeAnalysisActions.buildAiCacheKey(runeAnalysis);
    const force = options?.force ?? false;
    if (!force && cacheKey === runeAiCacheKey && get(runeAiInsights).length > 0) {
      return get(runeAiInsights);
    }

    const allRunes = get(allPlayersRunes);
    const teamTrees = Object.values(allRunes)
      .map((r: any) => r?.primary_tree)
      .filter(Boolean);

    const quality = runeAnalysisActions.analyzeRuneQuality(runeAnalysis);
    const payload = {
      analysis: {
        ...runeAnalysis,
        quality_score: quality.score,
        issues: quality.issues,
      },
      team_trees: teamTrees,
    };

    runeAiInsightsLoading.set(true);
    runeAiInsightsError.set(null);

    try {
      const insights = await invoke<any[]>('get_rune_ai_insights', { payload });
      runeAiCacheKey = cacheKey;
      runeAiInsights.set(Array.isArray(insights) ? insights : []);
      return get(runeAiInsights);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      runeAiInsightsError.set(message);
      if (force) runeAiInsights.set([]);
      return get(runeAiInsights);
    } finally {
      runeAiInsightsLoading.set(false);
    }
  },

  // Update rune recommendations for a specific player
  updateRuneRecommendations: (playerId: string, recommendations: any[]) => {
    runeRecommendations.update(current => ({
      ...current,
      [playerId]: recommendations
    }));
  },

  // Get rune tree color
  getRuneTreeColor: (tree: string) => {
    switch (tree.toLowerCase()) {
      case 'precision': return 'text-yellow-600';
      case 'domination': return 'text-red-600';
      case 'sorcery': return 'text-blue-600';
      case 'inspiration': return 'text-cyan-600';
      case 'resolve': return 'text-green-600';
      default: return 'text-gray-600';
    }
  },

  // Get rune tree icon
  getRuneTreeIcon: (tree: string) => {
    switch (tree.toLowerCase()) {
      case 'precision': return '⚔️';
      case 'domination': return '🗡️';
      case 'sorcery': return '🔮';
      case 'inspiration': return '💡';
      case 'resolve': return '🛡️';
      default: return '❓';
    }
  },

  // Get recommendation type color
  getRecommendationColor: (type: string) => {
    switch (type) {
      case 'error': return 'text-red-600';
      case 'warning': return 'text-yellow-600';
      case 'info': return 'text-blue-600';
      case 'success': return 'text-green-600';
      default: return 'text-gray-600';
    }
  },

  // Get recommendation priority color
  getPriorityColor: (priority: string) => {
    switch (priority) {
      case 'high': return 'text-red-600';
      case 'medium': return 'text-yellow-600';
      case 'low': return 'text-green-600';
      default: return 'text-gray-600';
    }
  },

  // Analyze rune setup quality
  analyzeRuneQuality: (analysis: any) => {
    if (!analysis) return { quality: 'unknown', score: 0, issues: [] };

    const issues: string[] = [];
    let score = 100;

    // Check for missing primary tree
    if (!analysis.primary_tree || analysis.primary_tree === 'Unknown') {
      issues.push('Missing primary rune tree');
      score -= 30;
    }

    // Check for missing secondary tree
    if (!analysis.secondary_tree || analysis.secondary_tree === 'Unknown') {
      issues.push('Missing secondary rune tree');
      score -= 20;
    }

    // Check for missing keystone
    if (!analysis.keystone || analysis.keystone === 0) {
      issues.push('Missing keystone rune');
      score -= 25;
    }

    // Check for tree duplication
    if (analysis.primary_tree === analysis.secondary_tree && analysis.primary_tree !== 'Unknown') {
      issues.push('Primary and secondary trees are the same');
      score -= 15;
    }

    // Check for insufficient runes
    if (analysis.total_runes < 6) {
      issues.push('Insufficient runes selected');
      score -= 10;
    }

    // Determine quality
    let quality: string;
    if (score >= 90) quality = 'excellent';
    else if (score >= 75) quality = 'good';
    else if (score >= 60) quality = 'average';
    else if (score >= 40) quality = 'poor';
    else quality = 'critical';

    return { quality, score, issues };
  },

  // Get rune setup summary
  getRuneSetupSummary: (analysis: any) => {
    if (!analysis) {
      return {
        primaryTree: 'Unknown',
        secondaryTree: 'Unknown',
        keystone: 'None',
        totalRunes: 0,
        quality: 'unknown'
      };
    }

    const quality = runeAnalysisActions.analyzeRuneQuality(analysis);

    return {
      primaryTree: analysis.primary_tree || 'Unknown',
      secondaryTree: analysis.secondary_tree || 'Unknown',
      keystone: analysis.keystone || 'None',
      totalRunes: analysis.total_runes || 0,
      quality: quality.quality,
      score: quality.score,
      issues: quality.issues
    };
  },

  // Get team rune composition
  getTeamRuneComposition: (allRunes: Record<string, any>) => {
    const composition: Record<string, number> = {};
    let totalPlayers = 0;

    Object.values(allRunes).forEach((runes: any) => {
      if (runes && runes.primary_tree) {
        const tree = runes.primary_tree;
        composition[tree] = (composition[tree] || 0) + 1;
        totalPlayers++;
      }
    });

    return {
      composition,
      totalPlayers,
      diversity: Object.keys(composition).length,
      mostCommon: Object.entries(composition).sort(([,a], [,b]) => (b as number) - (a as number))[0]?.[0] || 'Unknown'
    };
  }
};

// Derived stores
export const runeAnalysisSummary = derived(
  [allPlayersRunes, runeAnalysisData, currentPlayerRunes],
  ([allRunes, analysisData, currentRunes]) => {
    const currentAnalysis = currentRunes || analysisData['current'] || analysisData['me'];
    const summary = runeAnalysisActions.getRuneSetupSummary(currentAnalysis);
    const teamComposition = runeAnalysisActions.getTeamRuneComposition(allRunes);

    return {
      currentPlayer: summary,
      teamComposition,
      totalPlayers: Object.keys(allRunes).length,
      playersWithRunes: Object.values(analysisData).filter((a: any) => a && a.total_runes > 0).length
    };
  }
);

// Current player rune insights (core rules + optional AI)
export const runeInsights = derived(
  [currentPlayerRunes, runeAnalysisData, runeAiInsights],
  ([currentRunes, analysisData, aiInsights]) => {
    const analysis = currentRunes || analysisData['current'] || analysisData['me'];
    if (!analysis) return [];

    const core: Array<{ type: string; title: string; message: string; source?: string }> = [];
    const quality = runeAnalysisActions.analyzeRuneQuality(analysis);

    if (quality.quality === 'excellent') {
      core.push({
        type: 'success',
        title: 'Excellent Rune Setup',
        message: `Well-optimized setup (${quality.score}/100).`,
      });
    } else if (quality.quality === 'critical') {
      core.push({
        type: 'error',
        title: 'Critical Rune Issues',
        message: `Fix runes before loading — score ${quality.score}/100.`,
      });
    }

    if (!analysis.keystone || analysis.keystone === 0) {
      core.push({
        type: 'error',
        title: 'Missing Keystone',
        message: 'No keystone detected — pick one before the game starts.',
      });
    }

    if (analysis.total_runes < 4) {
      core.push({
        type: 'warning',
        title: 'Incomplete Setup',
        message: `Only ${analysis.total_runes} runes selected.`,
      });
    }

    return [...core, ...(Array.isArray(aiInsights) ? aiInsights : [])];
  }
);
