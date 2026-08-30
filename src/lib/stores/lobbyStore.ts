import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { isOllamaFeatureEnabled } from './ollamaStatusStore';

// Lobby Management State
export const lobbyData = writable<any>(null);
export const lobbyAnalysis = writable<any>(null);
export const readyCheckData = writable<any>(null);
export const lobbyStatus = writable<any>(null);
export const lobbyAiInsights = writable<any[]>([]);
export const lobbyAiInsightsLoading = writable(false);
export const lobbyAiInsightsError = writable<string | null>(null);

let lobbyAiCacheKey = '';

// Lobby Management Actions
export const lobbyActions = {
  // Update lobby data
  updateLobbyData: (data: any) => {
    lobbyData.set(data);
  },

  // Update lobby analysis
  updateLobbyAnalysis: (analysis: any) => {
    lobbyAnalysis.set(analysis);
    void lobbyActions.fetchAiInsights(analysis);
  },

  // Update ready check data
  updateReadyCheckData: (data: any) => {
    readyCheckData.set(data);
  },

  // Update lobby status
  updateLobbyStatus: (status: any) => {
    lobbyStatus.set(status);
  },

  // Get queue name from queue ID
  getQueueName: (queueId: number) => {
    const queueMap: Record<number, string> = {
      400: 'Draft Pick',
      420: 'Ranked Solo/Duo',
      430: 'Blind Pick',
      440: 'Ranked Flex',
      450: 'ARAM',
      700: 'Clash',
      830: 'Co-op vs AI (Intro)',
      840: 'Co-op vs AI (Beginner)',
      850: 'Co-op vs AI (Intermediate)',
      900: 'URF',
      1020: 'One for All',
      1300: 'Nexus Blitz',
      1400: 'Ultimate Spellbook',
      1900: 'URF (Return)',
      2000: 'Tutorial 1',
      2010: 'Tutorial 2',
      2020: 'Tutorial 3'
    };
    return queueMap[queueId] || `Queue ${queueId}`;
  },

  // Get game mode name
  getGameModeName: (gameMode: string) => {
    const modeMap: Record<string, string> = {
      'CLASSIC': 'Summoner\'s Rift',
      'ARAM': 'Howling Abyss',
      'TUTORIAL': 'Tutorial',
      'URF': 'Ultra Rapid Fire',
      'ONEFORALL': 'One for All',
      'NEXUSBLITZ': 'Nexus Blitz',
      'ULTBOOK': 'Ultimate Spellbook'
    };
    return modeMap[gameMode] || gameMode;
  },

  // Get ready check state color
  getReadyCheckStateColor: (state: string) => {
    switch (state) {
      case 'InProgress': return 'text-yellow-600';
      case 'EveryoneReady': return 'text-green-600';
      case 'StrangerNotReady': return 'text-orange-600';
      case 'Error': return 'text-red-600';
      default: return 'text-gray-600';
    }
  },

  // Get ready check state text
  getReadyCheckStateText: (state: string) => {
    switch (state) {
      case 'InProgress': return 'Ready Check Active';
      case 'EveryoneReady': return 'All Players Ready';
      case 'StrangerNotReady': return 'Waiting for Players';
      case 'Error': return 'Ready Check Error';
      default: return 'Unknown State';
    }
  },

  // Get member status color
  getMemberStatusColor: (isReady: boolean) => {
    return isReady ? 'text-green-600' : 'text-yellow-600';
  },

  // Get member status text
  getMemberStatusText: (isReady: boolean) => {
    return isReady ? 'Ready' : 'Not Ready';
  },

  // Get role icon
  getRoleIcon: (position: string) => {
    switch (position) {
      case 'TOP': return '🔺';
      case 'JUNGLE': return '🌿';
      case 'MIDDLE': return '⚔️';
      case 'BOTTOM': return '🏹';
      case 'UTILITY': return '🛡️';
      default: return '❓';
    }
  },

  // Get role name
  getRoleName: (position: string) => {
    switch (position) {
      case 'TOP': return 'Top';
      case 'JUNGLE': return 'Jungle';
      case 'MIDDLE': return 'Mid';
      case 'BOTTOM': return 'ADC';
      case 'UTILITY': return 'Support';
      default: return position;
    }
  },

  // Calculate lobby readiness
  calculateReadiness: (members: any[]) => {
    if (!members || members.length === 0) {
      return { ready: 0, total: 0, percentage: 0 };
    }

    const ready = members.filter(member => member.ready).length;
    const total = members.length;
    const percentage = Math.round((ready / total) * 100);

    return { ready, total, percentage };
  },

  // Get lobby insights
  getLobbyInsights: (analysis: any) => {
    if (!analysis || !analysis.team_analysis) return [];

    const insights = [];
    const teamAnalysis = analysis.team_analysis;

    // Readiness insight
    if (teamAnalysis.ready_percentage < 100) {
      insights.push({
        type: 'warning',
        title: 'Not All Ready',
        message: `Only ${teamAnalysis.ready_percentage.toFixed(0)}% of team members are ready.`
      });
    } else {
      insights.push({
        type: 'success',
        title: 'All Ready',
        message: 'All team members are ready to start!'
      });
    }

    // Team size insight
    if (teamAnalysis.total_members < 5) {
      insights.push({
        type: 'info',
        title: 'Incomplete Team',
        message: `Team has ${teamAnalysis.total_members} members. Need ${5 - teamAnalysis.total_members} more.`
      });
    }

    // Champion diversity insight
    if (teamAnalysis.champion_diversity < teamAnalysis.total_members) {
      insights.push({
        type: 'info',
        title: 'Champion Diversity',
        message: `Team has ${teamAnalysis.champion_diversity} unique champions. Consider diversity for better composition.`
      });
    }

    return insights;
  },

  buildAiCacheKey: (analysis: any) => {
    const team = analysis?.team_analysis;
    return [
      team?.total_members,
      team?.ready_members,
      team?.ready_percentage,
      analysis?.queue_id,
    ].join(':');
  },

  fetchAiInsights: async (analysis?: any, options?: { force?: boolean }) => {
    const lobby = analysis ?? get(lobbyAnalysis);
    if (!lobby?.team_analysis || !isOllamaFeatureEnabled('lobby')) {
      lobbyAiInsights.set([]);
      lobbyAiInsightsError.set(null);
      return [];
    }

    const cacheKey = lobbyActions.buildAiCacheKey(lobby);
    const force = options?.force ?? false;
    if (!force && cacheKey === lobbyAiCacheKey && get(lobbyAiInsights).length > 0) {
      return get(lobbyAiInsights);
    }

    lobbyAiInsightsLoading.set(true);
    lobbyAiInsightsError.set(null);

    try {
      const insights = await invoke<any[]>('get_lobby_ai_insights', { analysis: lobby });
      lobbyAiCacheKey = cacheKey;
      lobbyAiInsights.set(Array.isArray(insights) ? insights : []);
      return get(lobbyAiInsights);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      lobbyAiInsightsError.set(message);
      if (force) lobbyAiInsights.set([]);
      return get(lobbyAiInsights);
    } finally {
      lobbyAiInsightsLoading.set(false);
    }
  },
};

// Derived stores
export const lobbySummary = derived(
  [lobbyData, lobbyAnalysis, readyCheckData, lobbyStatus],
  ([lobby, analysis, readyCheck, status]) => {
    if (!lobby && !status) {
      return {
        isInLobby: false,
        queueName: 'Unknown',
        gameMode: 'Unknown',
        memberCount: 0,
        readyCount: 0,
        readinessPercentage: 0,
        readyCheckState: 'None',
        readyCheckTimer: 0
      };
    }

    const lobbyInfo = status?.lobby_data || lobby;
    const analysisInfo = analysis?.team_analysis;
    const readyCheckInfo = status?.ready_check_data || readyCheck;

    return {
      isInLobby: true,
      queueName: lobbyActions.getQueueName(status?.queue_id || lobbyInfo?.gameConfig?.queueId || 0),
      gameMode: lobbyActions.getGameModeName(status?.game_mode || lobbyInfo?.gameConfig?.gameMode || 'Unknown'),
      memberCount: analysisInfo?.total_members || lobbyInfo?.members?.length || 0,
      readyCount: analysisInfo?.ready_members || 0,
      readinessPercentage: analysisInfo?.ready_percentage || 0,
      readyCheckState: readyCheckInfo?.state || 'None',
      readyCheckTimer: readyCheckInfo?.timer || 0
    };
  }
);

// Current lobby insights (core rules + optional AI)
export const lobbyInsights = derived(
  [lobbyAnalysis, lobbyAiInsights],
  ([analysis, aiInsights]) => {
    const core = lobbyActions.getLobbyInsights(analysis);
    const trimmed = core.filter(
      (i) => i.title === 'Not All Ready' || i.title === 'All Ready' || i.title === 'Incomplete Team',
    );
    return [...trimmed, ...(Array.isArray(aiInsights) ? aiInsights : [])];
  }
);

// Lobby members with enhanced data
export const lobbyMembers = derived(
  [lobbyData, lobbyStatus, lobbyAnalysis],
  ([lobby, status, analysis]) => {
    const lobbyInfo = status?.lobby_data || lobby;
    const members = (analysis?.members ?? lobbyInfo?.members) || [];
    
    return members.map((member: any) => ({
      ...member,
      summonerName:
        member.summonerName ||
        member.riot_id ||
        (member.game_name && member.tag_line
          ? `${member.game_name}#${member.tag_line}`
          : member.gameName && member.tagLine
            ? `${member.gameName}#${member.tagLine}`
            : undefined),
      roleIcon: lobbyActions.getRoleIcon(member.position ?? member.firstPositionPreference),
      roleName: lobbyActions.getRoleName(member.position ?? member.firstPositionPreference),
      statusColor: lobbyActions.getMemberStatusColor(member.ready),
      statusText: lobbyActions.getMemberStatusText(member.ready)
    }));
  }
);

