import { writable, derived } from 'svelte/store';

// Item Tracking State
export const allPlayersItems = writable<Record<string, any>>({});
export const activePlayerItems = writable<any[]>([]);
export const itemRecommendations = writable<Record<string, any[]>>({});
export const buildAnalysis = writable<Record<string, any>>({});

// Item Analysis Functions
export const itemTrackingActions = {
  // Update all players' items
  updateAllPlayersItems: (itemsData: Record<string, any>) => {
    allPlayersItems.set(itemsData);
    
    // Analyze builds for each player
    const analysis: Record<string, any> = {};
    Object.entries(itemsData).forEach(([playerId, playerItems]) => {
      analysis[playerId] = itemTrackingActions.analyzePlayerBuild(playerItems);
    });
    buildAnalysis.set(analysis);
  },

  // Analyze a player's current build
  analyzePlayerBuild: (playerItems: any) => {
    if (!playerItems || !Array.isArray(playerItems)) {
      return {
        totalItems: 0,
        completedItems: 0,
        goldEfficiency: 0,
        buildPath: [],
        recommendations: []
      };
    }

    const completedItems = playerItems.filter((item: any) => item && item.itemID && item.itemID !== 0);
    const totalItems = playerItems.length;
    const completedCount = completedItems.length;

    // Calculate gold efficiency (simplified)
    const goldEfficiency = completedCount > 0 ? (completedCount / totalItems) * 100 : 0;

    // Analyze build path
    const buildPath = completedItems.map((item: any) => ({
      id: item.itemID,
      name: item.displayName || `Item ${item.itemID}`,
      cost: item.totalCost || 0,
      slot: item.slot || 0
    }));

    // Generate recommendations based on current build
    const recommendations = itemTrackingActions.generateRecommendations(completedItems);

    return {
      totalItems,
      completedItems: completedCount,
      goldEfficiency: Math.round(goldEfficiency),
      buildPath,
      recommendations
    };
  },

  // Generate item recommendations
  generateRecommendations: (currentItems: any[]) => {
    const recommendations: any[] = [];
    
    // Basic recommendations based on item count
    if (currentItems.length < 2) {
      recommendations.push({
        type: 'early_game',
        priority: 'high',
        message: 'Focus on completing your first core item',
        items: ['Dorans items', 'Boots', 'Core item components']
      });
    } else if (currentItems.length < 4) {
      recommendations.push({
        type: 'mid_game',
        priority: 'medium',
        message: 'Consider defensive items or situational picks',
        items: ['Defensive items', 'Magic resistance', 'Armor']
      });
    } else {
      recommendations.push({
        type: 'late_game',
        priority: 'low',
        message: 'Optimize for team fights and objectives',
        items: ['Elixirs', 'Situational items', 'Consumables']
      });
    }

    return recommendations;
  },

  // Get item efficiency score
  getItemEfficiency: (items: any[]) => {
    if (!items || items.length === 0) return 0;
    
    const completedItems = items.filter(item => item && item.itemID && item.itemID !== 0);
    return Math.round((completedItems.length / items.length) * 100);
  },

  // Format item data for display
  formatItemData: (item: any) => {
    if (!item) return null;
    
    return {
      id: item.itemID || 0,
      name: item.displayName || `Item ${item.itemID}`,
      cost: item.totalCost || 0,
      slot: item.slot || 0,
      isCompleted: item.itemID && item.itemID !== 0,
      isConsumable: item.consumable || false,
      isTrinket: item.trinket || false
    };
  },

  // Get build summary for a player
  getBuildSummary: (playerId: string) => {
    let summary: {
      playerId: string;
      totalItems: number;
      completedItems: number;
      goldEfficiency: number;
      buildStage: string;
      recommendations: any[];
    } = {
      playerId,
      totalItems: 0,
      completedItems: 0,
      goldEfficiency: 0,
      buildStage: 'early',
      recommendations: []
    };

    allPlayersItems.subscribe(items => {
      const playerItems = items[playerId];
      if (playerItems) {
        const analysis = itemTrackingActions.analyzePlayerBuild(playerItems);
        summary = {
          playerId,
          totalItems: analysis.totalItems,
          completedItems: analysis.completedItems,
          goldEfficiency: analysis.goldEfficiency,
          buildStage: analysis.completedItems < 2 ? 'early' : 
                     analysis.completedItems < 4 ? 'mid' : 'late',
          recommendations: analysis.recommendations as any[]
        };
      }
    });

    return summary;
  }
};

// Derived stores
export const itemTrackingSummary = derived(
  [allPlayersItems, buildAnalysis],
  ([items, analysis]) => {
    const playerCount = Object.keys(items).length;
    const totalPlayers = playerCount;
    const playersWithItems = Object.values(analysis).filter((a: any) => a.completedItems > 0).length;
    
    return {
      totalPlayers,
      playersWithItems,
      averageEfficiency: totalPlayers > 0 ? 
        Math.round(Object.values(analysis).reduce((sum: number, a: any) => sum + a.goldEfficiency, 0) / totalPlayers) : 0
    };
  }
);

// Active player items (for current player focus)
export const activePlayerItemAnalysis = derived(
  [allPlayersItems, buildAnalysis],
  ([items, analysis]) => {
    // For now, return the first player's data as active player
    // This could be enhanced to track the actual active player
    const firstPlayerId = Object.keys(items)[0];
    if (firstPlayerId && analysis[firstPlayerId]) {
      return {
        playerId: firstPlayerId,
        ...analysis[firstPlayerId]
      };
    }
    return null;
  }
);
