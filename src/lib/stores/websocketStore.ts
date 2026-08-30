import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { 
  LiveGameData, 
  LiveClientGameData,
  GameStats, 
  PlayerScoresData, 
  ComprehensiveGameState 
} from '$lib/types/game';
import {
  champSelectTimerSec,
  activeChampSelectActionType,
  refineChampSelectPhase,
} from '$lib/utils/championSelectParser';

/** Phases where champ-select session data from LCU is stale and must be ignored. */
export const POST_CHAMP_SELECT_PHASES = new Set([
  'InProgress',
  'GameStart',
  'in_game',
  'EndOfGame',
  'WaitingForStats',
  'PreEndOfGame',
]);

export function isPostChampSelectPhase(phase: string | null | undefined): boolean {
  return !!phase && POST_CHAMP_SELECT_PHASES.has(phase);
}

/** True while loading into game after champ select ends (LCU phase may lag). */
const CHAMP_SELECT_TRANSITION_MS = 120_000;
let champSelectEndedAt: number | null = null;

export function isRecentChampSelectTransition(
  maxMs: number = CHAMP_SELECT_TRANSITION_MS,
): boolean {
  return champSelectEndedAt !== null && Date.now() - champSelectEndedAt < maxMs;
}

export function isInMatchOrLoadingPhase(phase: string | null | undefined): boolean {
  if (isPostChampSelectPhase(phase)) return true;
  const preMatchPhases = ['None', 'Lobby', 'Matchmaking', 'ReadyCheck', 'TerminatedInError'];
  if (phase && preMatchPhases.includes(phase)) return false;
  if (phase === 'ChampSelect' || phase === 'champion_select') return false;
  return isRecentChampSelectTransition();
}

function setChampionSelectStores(cs: Record<string, unknown>) {
  champSelectEndedAt = null;
  wsChampionSelect.set(cs);
  wsChampionSelectData.set(cs);
  wsIsChampionSelectActive.set(true);
  const timerPhase = (cs.timer as { phase?: string } | undefined)?.phase;
  wsChampionSelectPhase.set(
    timerPhase
      ? refineChampSelectPhase(timerPhase, activeChampSelectActionType(cs.actions))
      : 'Active',
  );
  wsChampionSelectTimer.set(champSelectTimerSec(cs.timer as Parameters<typeof champSelectTimerSec>[0]));
}

function clearChampionSelectStores() {
  const hadSession = get(wsIsChampionSelectActive) || get(wsChampionSelectData);
  if (hadSession) {
    champSelectEndedAt = Date.now();
  }
  wsChampionSelect.set(null);
  wsChampionSelectData.set(null);
  wsIsChampionSelectActive.set(false);
  wsChampionSelectPhase.set('None');
  wsChampionSelectTimer.set(0);
}

// WebSocket connection state
export const wsConnected = writable(false);
export const wsConnecting = writable(false);
export const wsConnectionError = writable<string | null>(null);
export const wsReconnectAttempts = writable(0);
export const maxReconnectAttempts = 5;
export const wsLCUConnecting = writable(false);

// WebSocket server state
export const wsServerRunning = writable(false);
export const wsServerPort = writable<number | null>(null);

// Game data stores (same as liveGameStore but for WebSocket)
export const wsGameData = writable<LiveClientGameData | null>(null);
export const wsLiveGameData = writable<LiveClientGameData | null>(null);
export const wsLiveGameStats = writable<GameStats | null>(null);
export const wsPlayerScores = writable<PlayerScoresData | null>(null);
export const wsGameState = writable<ComprehensiveGameState | null>(null);

// UI State
export const wsIsLoading = writable(false);
export const wsError = writable<string | null>(null);
export const wsLastErrorState = writable<string | null>(null);
export const wsErrorStable = writable(false);

// Game Info
export const wsGameTime = writable('00:00');
export const wsGamePhase = writable('No Live Game');
export const wsGameMode = writable('Unknown');
export const wsIsSpectatorMode = writable(false);

// Champion Select Data
export const wsChampionSelect = writable<any>(null);
export const wsChampionSelectData = writable<any>(null);
export const wsChampionSelectPhase = writable<string>('None');
export const wsChampionSelectTimer = writable<number>(0);
export const wsIsChampionSelectActive = writable<boolean>(false);

// Game Events
export const wsGameEvents = writable<any[]>([]);

// Item Tracking Data
export const wsAllPlayersItems = writable<Record<string, any>>({});
export const wsItemRecommendations = writable<Record<string, any[]>>({});
export const wsBuildAnalysis = writable<Record<string, any>>({});

// Match History Data
export const wsMatchHistory = writable<Record<string, any>>({});
export const wsMatchHistoryData = writable<Record<string, any>>({});
export const wsPlayerStatsAnalysis = writable<Record<string, any>>({});

// Rune Analysis Data
export const wsAllPlayersRunes = writable<Record<string, any>>({});
export const wsRuneAnalysisData = writable<Record<string, any>>({});
export const wsRuneRecommendations = writable<Record<string, any[]>>({});

// Lobby Management Data
export const wsLobbyData = writable<any>(null);
export const wsLobbyAnalysis = writable<any>(null);
export const wsReadyCheckData = writable<any>(null);
export const wsLobbyStatus = writable<any>(null);

// Connection Health and LCU WebSocket Data
export const wsConnectionHealth = writable<any>(null);
export const wsLCUWebSocketEvents = writable<any[]>([]);
export const wsLCUWebSocketConnected = writable<boolean>(false);

// Game Time Tracking
let wsGameTimeInterval: ReturnType<typeof setInterval> | null = null;
let wsGameStartTime: number | null = null;
let wsLastApiGameTime: number | null = null;
let wsLastApiSyncTime: number | null = null;

// WebSocket connection
let ws: WebSocket | null = null;
let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
let isListening = false;
let connectionHealthCheck: ReturnType<typeof setInterval> | null = null;

// WebSocket message types - matches backend Rust enum
interface WebSocketMessage {
  Subscribe?: { topics: string[] };
  Unsubscribe?: { topics: string[] };
  Ping?: {};
  Pong?: {};
  GameStateUpdate?: { data: any };
  GameStateDelta?: { changes: any };
  LCUConnectionUpdate?: { connected: boolean; status: string };
  LiveGameDataUpdate?: { data: any };
  LiveGameDataDelta?: { changes: any };
  PlayerStatsUpdate?: { player_id: string; stats: any };
  GameTimeUpdate?: { game_time: number };
  GameEvent?: { event_type: string; data: any };
  ChampionSelectUpdate?: { data: any };
  ChampionSelectDelta?: { changes: any };
  PlayerItemsUpdate?: { player_id: string; items: any };
  AllPlayersItemsUpdate?: { items: any };
  BuildRecommendation?: { player_id: string; recommendations: any };
  MatchHistoryUpdate?: { player_id: string; analysis: any };
  PlayerStatsAnalysis?: { player_id: string; stats: any };
  AllPlayersRunesUpdate?: { runes: any };
  RuneAnalysisUpdate?: { player_id: string; analysis: any };
  RuneRecommendation?: { player_id: string; recommendations: any };
  LobbyUpdate?: { data: any };
  LobbyAnalysis?: { analysis: any };
  ReadyCheckUpdate?: { data: any };
  ReadyCheckAction?: { action: string; result: any };
  LCUWebSocketEvent?: { event: any };
  ConnectionHealthUpdate?: { health: any };
  Error?: { message: string };
}

// Stable error handling
function setStableError(errorMessage: string | null) {
  let currentError: string | null = null;
  let currentLastError: string | null = null;
  
  const unsubscribeError = wsError.subscribe(e => currentError = e);
  const unsubscribeLastError = wsLastErrorState.subscribe(e => currentLastError = e);
  
  unsubscribeError();
  unsubscribeLastError();
  
  // Only update error if it's different from the last stable state
  if (errorMessage !== currentLastError) {
    wsError.set(errorMessage);
    wsLastErrorState.set(errorMessage);
    wsErrorStable.set(true);
    
    // Reset stability flag after a delay to allow for future changes
    setTimeout(() => {
      wsErrorStable.set(false);
    }, 2000);
  }
}

// Game time tracking functions
function startGameTimeTicker() {
  stopGameTimeTicker();
  wsGameTimeInterval = setInterval(updateGameTime, 1000);
}

function stopGameTimeTicker() {
  if (wsGameTimeInterval) {
    clearInterval(wsGameTimeInterval);
    wsGameTimeInterval = null;
  }
}

function updateGameTime() {
  if (!wsGameStartTime) {
    return;
  }
  const elapsed = Math.floor((Date.now() - wsGameStartTime) / 1000);
  const formatted = formatGameTime(elapsed);
  wsGameTime.set(formatted);
}

function syncGameTimeWithApi(apiGameTime: number) {
  console.log('[WebSocket] Syncing game time with API:', apiGameTime);
  wsLastApiGameTime = apiGameTime;
  wsLastApiSyncTime = Date.now();
  
  // Start local timer from API time
  wsGameStartTime = Date.now() - (apiGameTime * 1000);
  console.log('[WebSocket] Set game start time:', wsGameStartTime, 'for API time:', apiGameTime);
  updateGameTime();
}

function getSyncStatus(): string {
  if (!wsLastApiSyncTime) return 'local';
  
  const timeSinceSync = Date.now() - wsLastApiSyncTime;
  if (timeSinceSync < 5000) return 'synced'; // Less than 5 seconds
  if (timeSinceSync < 30000) return 'stale'; // Less than 30 seconds
  return 'local';
}

function formatGameTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

// WebSocket connection management
async function startWebSocketServer(): Promise<boolean> {
  try {
    const result = await invoke('start_websocket_server', { port: 8765 });
    console.log('[WebSocket] Server started:', result);
    wsServerRunning.set(true);
    wsServerPort.set(8765);
    return true;
  } catch (error) {
    console.error('[WebSocket] Failed to start server:', error);
    setStableError(`Failed to start WebSocket server: ${error}`);
    return false;
  }
}

async function stopWebSocketServer(): Promise<void> {
  try {
    await invoke('stop_websocket_server');
    console.log('[WebSocket] Server stopped');
    wsServerRunning.set(false);
    wsServerPort.set(null);
  } catch (error) {
    console.error('[WebSocket] Failed to stop server:', error);
  }
}

async function getWebSocketServerStatus(): Promise<void> {
  try {
    const status = await invoke('get_websocket_server_status') as any;
    wsServerRunning.set(status.running);
    wsServerPort.set(status.port);
  } catch (error) {
    console.error('[WebSocket] Failed to get server status:', error);
  }
}

export function connectWebSocket(): Promise<void> {
  return new Promise((resolve, reject) => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      resolve();
      return;
    }

    wsConnecting.set(true);
    wsConnectionError.set(null);

    const wsUrl = `ws://127.0.0.1:8765`;
    console.log('[WebSocket] Connecting to:', wsUrl);

    ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      console.log('[WebSocket] Connected successfully');
      wsConnected.set(true);
      wsConnecting.set(false);
      wsReconnectAttempts.set(0);
      setStableError(null);
      
      // Subscribe to all topics
      sendMessage({ Subscribe: { topics: ['all', 'game_state', 'live_game', 'lcu_connection', 'game_events'] } });
      
      // Send a ping to test the connection
      setTimeout(() => {
        sendMessage({ Ping: {} });
      }, 1000);
      
      // Connection health check is handled by the WebSocket server
      
      resolve();
    };

    ws.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);
        handleWebSocketMessage(message);
      } catch (error) {
        console.error('[WebSocket] Failed to parse message:', error);
      }
    };

    ws.onclose = (event) => {
      console.log('[WebSocket] Connection closed:', event.code, event.reason);
      wsConnected.set(false);
      wsConnecting.set(false);
      
      if (event.code !== 1000 && isListening) { // Not a normal closure
        scheduleReconnect();
      }
      
      reject(new Error(`WebSocket closed: ${event.code} ${event.reason}`));
    };

    ws.onerror = (error) => {
      console.error('[WebSocket] Connection error:', error);
      wsConnecting.set(false);
      wsConnectionError.set('WebSocket connection error');
      setStableError('WebSocket connection error');
      reject(error);
    };
  });
}

function scheduleReconnect() {
  if (!isListening) return;
  
  const attempts = get(wsReconnectAttempts);
  if (attempts >= maxReconnectAttempts) {
    setStableError('WebSocket connection failed after maximum retry attempts');
    return;
  }

  const delay = Math.min(1000 * Math.pow(2, attempts), 30000); // Exponential backoff, max 30s
  console.log(`[WebSocket] Reconnecting in ${delay}ms (attempt ${attempts + 1}/${maxReconnectAttempts})`);
  
  wsReconnectAttempts.set(attempts + 1);
  
  reconnectTimeout = setTimeout(() => {
    if (isListening) {
      connectWebSocket().catch(console.error);
    }
  }, delay);
}

function sendMessage(message: WebSocketMessage) {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify(message));
  } else {
    console.warn('[WebSocket] Cannot send message - not connected');
  }
}

function applyGameSnapshot(data: ComprehensiveGameState) {
  wsGameState.set(data);

  if (data.game_phase) {
    wsGamePhase.set(data.game_phase);
    const preMatchPhases = ['None', 'Lobby', 'Matchmaking', 'ReadyCheck', 'TerminatedInError'];
    if (preMatchPhases.includes(data.game_phase)) {
      champSelectEndedAt = null;
    }
  }

  wsIsSpectatorMode.set(data.is_spectator_mode ?? false);

  const phase = data.game_phase ?? get(wsGamePhase);
  if (data.champion_select && !isPostChampSelectPhase(phase)) {
    setChampionSelectStores(data.champion_select as Record<string, unknown>);
    if (phase === 'Unknown' || phase === 'ChampSelect' || phase === 'champion_select') {
      wsGamePhase.set('ChampSelect');
    }
  } else {
    clearChampionSelectStores();
  }

  if (data.live_game) {
    champSelectEndedAt = null;
    wsLiveGameData.set(data.live_game as LiveClientGameData);
    wsGameData.set(data.live_game as LiveClientGameData);
  } else if (data.is_spectator_mode && data.spectator_game) {
    champSelectEndedAt = null;
    const spectator = data.spectator_game as LiveClientGameData;
    wsLiveGameData.set(spectator);
    wsGameData.set(spectator);
  } else if (!data.live_client_available) {
    wsLiveGameData.set(null);
    wsGameData.set(null);
  }

  if (data.live_stats) {
    wsLiveGameStats.set(data.live_stats);
  } else if (!data.live_client_available) {
    wsLiveGameStats.set(null);
  }

  if (data.player_scores) {
    wsPlayerScores.set(data.player_scores);
  } else if (!data.live_client_available) {
    wsPlayerScores.set(null);
  }

  if (data.live_game) {
    const mode = (data.live_game as { gameData?: { gameMode?: string } }).gameData?.gameMode;
    if (mode) wsGameMode.set(mode);
  }
}

function handleWebSocketMessage(message: WebSocketMessage) {
  if (message.GameStateUpdate) {
    const data = message.GameStateUpdate.data as ComprehensiveGameState;
    console.log('[WebSocket] GameStateUpdate received:', data);
    applyGameSnapshot(data);
  } else if (message.GameStateDelta) {
    const changes = message.GameStateDelta.changes;
    console.log('[WebSocket] GameStateDelta received:', changes);
    // Apply delta changes to current state
    wsGameState.update(currentState => {
      if (!currentState) return changes;
      return { ...currentState, ...changes };
    });

    const deltaPhase = (changes.game_phase ?? get(wsGamePhase)) as string;
    if (changes.champion_select && !isPostChampSelectPhase(deltaPhase)) {
      setChampionSelectStores(changes.champion_select as Record<string, unknown>);
      if (deltaPhase === 'Unknown' || deltaPhase === 'ChampSelect' || deltaPhase === 'champion_select') {
        wsGamePhase.set('ChampSelect');
      }
    } else if (isPostChampSelectPhase(deltaPhase)) {
      clearChampionSelectStores();
    }
    if (changes.live_game) {
      wsLiveGameData.set(changes.live_game);
      wsGameData.set(changes.live_game);
    } else if (changes.spectator_game) {
      wsLiveGameData.set(changes.spectator_game);
      wsGameData.set(changes.spectator_game);
    }
    
    // Update game phase and mode
    if (changes.game_phase) {
      wsGamePhase.set(changes.game_phase);
    }
    if (changes.gameMode) {
      wsGameMode.set(changes.gameMode);
    }
    
    // Enhanced spectator mode detection for GameStateDelta
    const currentState = get(wsGameState);
    const updatedState = { ...currentState, ...changes };
    
    let isSpectator = false;
    
    // Method 1: Check updated state for spectator mode
    if (updatedState.game_phase === 'in_game' && updatedState.live_client_available === false) {
      isSpectator = true;
      console.log('[WebSocket] Spectator mode detected via GameStateDelta - in game but live client unavailable');
    }
    
    // Method 2: Check if is_spectator_mode is explicitly set
    if (updatedState.is_spectator_mode === true) {
      isSpectator = true;
      console.log('[WebSocket] Spectator mode detected via GameStateDelta - is_spectator_mode flag');
    }
    
    // Method 3: Check for spectator mode in changes
    if (changes.is_spectator_mode === true) {
      isSpectator = true;
      console.log('[WebSocket] Spectator mode detected via GameStateDelta - changes.is_spectator_mode');
    }
    
    wsIsSpectatorMode.set(isSpectator);
    if (isSpectator) {
      setStableError('Spectator mode - limited data available');
      console.log('[WebSocket] Spectator mode confirmed via GameStateDelta');
    } else if (changes.game_phase !== undefined || changes.live_client_available !== undefined) {
      setStableError(null);
    }
  }
  
  if (message.GameTimeUpdate) {
    const gameTime = message.GameTimeUpdate.game_time;
    console.log('[WebSocket] GameTimeUpdate received:', gameTime);
    syncGameTimeWithApi(gameTime);
    // Ensure ticker is running
    startGameTimeTicker();
  }
  
  if (message.LCUConnectionUpdate) {
    const { connected, status } = message.LCUConnectionUpdate;
    console.log('[WebSocket] LCU Connection:', connected, status);
    
    // Handle different connection states more gracefully
    if (!connected) {
      if (status.includes('not connected') && status.includes('League running')) {
        // This is a temporary state during initial connection
        wsLCUConnecting.set(true);
        const currentGameData = get(wsGameData);
        if (!currentGameData || !currentGameData.allPlayers || currentGameData.allPlayers.length === 0) {
          // Show a friendly connecting message instead of an error
          setStableError(`Connecting to League Client...`);
        } else {
          // We have live game data, so LCU connection is not critical
          wsLCUConnecting.set(false);
          setStableError(null);
        }
      } else if (status.includes('not connected')) {
        // League not running or other connection issues
        wsLCUConnecting.set(false);
        setStableError(`LCU not connected: ${status}`);
      }
    } else {
      wsLCUConnecting.set(false);
      setStableError(null);
      
      // Try to connect LCU WebSocket if not already connected
      connectLcuWebSocket().catch(err => {
        console.warn('[WebSocket] Failed to connect LCU WebSocket:', err);
      });
    }
  }
  
  if (message.LiveGameDataUpdate) {
    const data = message.LiveGameDataUpdate.data;
    console.log('[WebSocket] LiveGameDataUpdate received:', data);
    console.log('[WebSocket] Data structure check:', {
      hasActivePlayer: !!data.activePlayer,
      hasAllPlayers: !!data.allPlayers,
      hasGameData: !!data.gameData,
      hasEvents: !!data.events,
      gameTime: data.gameData?.gameTime,
      spectatorMode: data.isSpectatorMode
    });
    wsGameData.set(data);
    wsLiveGameData.set(data);
    
    // Clear LCU connection error if we have live game data
    if (data.allPlayers && data.allPlayers.length > 0) {
      wsLCUConnecting.set(false);
      setStableError(null);
    }
    
    // Extract game time from full data
    if (data.gameData?.gameTime !== undefined) {
      console.log('[WebSocket] Game time from LiveGameDataUpdate:', data.gameData.gameTime);
      syncGameTimeWithApi(data.gameData.gameTime);
      // Ensure ticker is running
      startGameTimeTicker();
    }
  } else if (message.LiveGameDataDelta) {
    const changes = message.LiveGameDataDelta.changes;
    console.log('[WebSocket] LiveGameDataDelta received:', changes);
    console.log('[WebSocket] LiveGameDataDelta gameTime:', changes.gameTime);
    // Apply delta changes to current live game data
    wsGameData.update(currentData => {
      if (!currentData) return changes;
      const updatedData = { ...currentData, ...changes };
      
      // Clear LCU connection error if we have live game data
      if (updatedData.allPlayers && updatedData.allPlayers.length > 0) {
        wsLCUConnecting.set(false);
        setStableError(null);
      }
      
      return updatedData;
    });
    wsLiveGameData.update(currentData => {
      if (!currentData) return changes;
      const updatedData = { ...currentData, ...changes };
      
      // Clear LCU connection error if we have live game data
      if (updatedData.allPlayers && updatedData.allPlayers.length > 0) {
        wsLCUConnecting.set(false);
        setStableError(null);
      }
      
      return updatedData;
    });
    
    // Extract game time and mode from delta changes
    if (changes.gameTime !== undefined) {
      console.log('[WebSocket] Game time from LiveGameDataDelta:', changes.gameTime);
      syncGameTimeWithApi(changes.gameTime);
      // Ensure ticker is running
      startGameTimeTicker();
    }
    
    if (changes.gameMode) {
      wsGameMode.set(changes.gameMode);
    }
    
    // Enhanced spectator mode detection
    let isSpectator = false;
    
    // Method 1: Check for activePlayer error
    if (changes.activePlayer?.error === "Spectator mode doesn't currently support this feature") {
      isSpectator = true;
      console.log('[WebSocket] Spectator mode detected via activePlayer error');
    }
    
    // Method 2: Check game state for spectator mode
    const currentState = get(wsGameState);
    if (currentState) {
      const updatedState = { ...currentState, ...changes };
      const isSpectatorFromState = updatedState.game_phase === 'in_game' && updatedState.live_client_available === false;
      if (isSpectatorFromState) {
        isSpectator = true;
        console.log('[WebSocket] Spectator mode detected via game state');
      }
    }
    
    // Method 3: Check if we have allPlayers but no activePlayer (spectator mode)
    if (changes.allPlayers && Array.isArray(changes.allPlayers) && changes.allPlayers.length > 0 && !changes.activePlayer) {
      isSpectator = true;
      console.log('[WebSocket] Spectator mode detected - has allPlayers but no activePlayer');
    }
    
    // Method 4: Check if we're in game but live client is not available
    if (changes.game_phase === 'in_game' && changes.live_client_available === false) {
      isSpectator = true;
      console.log('[WebSocket] Spectator mode detected - in game but live client unavailable');
    }
    
    // Update spectator mode state
    wsIsSpectatorMode.set(isSpectator);
    if (isSpectator) {
      setStableError('Spectator mode - The Live Client Data API is not available when spectating. This is a Riot Games API limitation.');
      console.log('[WebSocket] Spectator mode confirmed - Live Client Data API unavailable');
    } else if (changes.game_phase !== undefined || changes.live_client_available !== undefined) {
      // Clear error if we're no longer in spectator mode
      setStableError(null);
    }
    
    // Check if game has ended and process events
    if (changes.events?.Events) {
      console.log('[WebSocket] Processing events from LiveGameDataDelta:', changes.events.Events);
      
      // Process each event
      changes.events.Events.forEach((event: any) => {
        if (event.EventName) {
          const eventData = {
            type: event.EventName.toLowerCase(),
            timestamp: Date.now(),
            eventName: event.EventName,
            ...event
          };
          
          // Add to game events store
          wsGameEvents.update(events => {
            const newEvents = [eventData, ...events].slice(0, 50);
            console.log('[WebSocket] Added event from LiveGameDataDelta:', event.EventName);
            return newEvents;
          });
        }
      });
      
      const gameEndEvent = changes.events.Events.find((event: any) => event.EventName === 'GameEnd');
      if (gameEndEvent) {
        const gameResult = gameEndEvent.Result || 'Unknown';
        const aceEvent = changes.events.Events.find((event: any) => event.EventName === 'Ace');
        let winningTeam = '';
        if (aceEvent && aceEvent.AcingTeam) {
          winningTeam = aceEvent.AcingTeam === 'ORDER' ? 'Blue Team' : 'Red Team';
        }
        
        if (winningTeam && gameResult === 'Win') {
          wsGamePhase.set(`${winningTeam} Victory!`);
        } else {
          wsGamePhase.set(`Game Ended - ${gameResult}`);
        }
        stopGameTimeTicker();
      } else {
        wsGamePhase.set('In Game');
      }
    }
  }
  
  if (message.GameEvent) {
    console.log('[WebSocket] Game Event received:', message.GameEvent.event_type, message.GameEvent.data);
    
    const eventData = {
      type: message.GameEvent.event_type,
      timestamp: Date.now(),
      ...message.GameEvent.data
    };
    
    // Add to game events store
    wsGameEvents.update(events => {
      const newEvents = [eventData, ...events].slice(0, 50); // Keep last 50 events
      console.log('[WebSocket] Updated game events store, total events:', newEvents.length);
      return newEvents;
    });
    
    // Handle specific event types
    switch (message.GameEvent.event_type) {
      case 'kill':
      case 'death':
      case 'assist':
        console.log('[WebSocket] Combat event:', eventData);
        break;
      case 'dragon':
      case 'baron':
      case 'herald':
        console.log('[WebSocket] Objective event:', eventData);
        break;
      case 'tower':
      case 'inhibitor':
        console.log('[WebSocket] Structure event:', eventData);
        break;
      default:
        console.log('[WebSocket] Other event:', eventData);
    }
  }
  
  if (message.Error) {
    setStableError(message.Error.message);
  }
  
  if (message.Pong) {
    // Handle pong response
    console.log('[WebSocket] Received pong');
  }
  
  if (message.ChampionSelectUpdate) {
    const data = message.ChampionSelectUpdate.data;

    if (data == null) {
      clearChampionSelectStores();
      void refreshGameStateFromBackend().catch(() => {});
      return;
    }

    const currentPhase = get(wsGamePhase);
    const hasSession = data && (data.myTeam || data.timer?.phase);
    if (hasSession && !isPostChampSelectPhase(currentPhase)) {
      setChampionSelectStores(data);
      if (currentPhase === 'Unknown' || currentPhase === 'ChampSelect' || currentPhase === 'champion_select') {
        wsGamePhase.set('ChampSelect');
      }
    } else if (!hasSession || isPostChampSelectPhase(currentPhase)) {
      clearChampionSelectStores();
    }
  } else if (message.ChampionSelectDelta) {
    const changes = message.ChampionSelectDelta.changes;
    const currentPhase = get(wsGamePhase);
    if (isPostChampSelectPhase(currentPhase)) {
      clearChampionSelectStores();
    } else {
      wsChampionSelect.update(currentData => {
        const merged = !currentData ? changes : { ...currentData, ...changes };
        if (merged?.timer) {
          wsChampionSelectTimer.set(champSelectTimerSec(merged.timer));
        }
        return merged;
      });
      wsChampionSelectData.update(currentData => {
        const merged = !currentData ? changes : { ...currentData, ...changes };
        return merged;
      });

      wsIsChampionSelectActive.set(true);
      if (changes.timer?.phase) {
        wsChampionSelectPhase.set(changes.timer.phase);
      } else if (changes.phase !== undefined) {
        wsChampionSelectPhase.set(changes.phase);
        wsIsChampionSelectActive.set(changes.phase !== 'None');
      }
      if (changes.timer !== undefined) {
        wsChampionSelectTimer.set(champSelectTimerSec(changes.timer));
      }
      if (currentPhase === 'Unknown' || currentPhase === 'ChampSelect' || currentPhase === 'champion_select') {
        wsGamePhase.set('ChampSelect');
      }
    }
  }
  
  if (message.PlayerItemsUpdate) {
    const { player_id, items } = message.PlayerItemsUpdate;
    console.log('[WebSocket] PlayerItemsUpdate received for player:', player_id);
    wsAllPlayersItems.update(currentItems => ({
      ...currentItems,
      [player_id]: items
    }));
  }
  
  if (message.AllPlayersItemsUpdate) {
    const items = message.AllPlayersItemsUpdate.items;
    console.log('[WebSocket] AllPlayersItemsUpdate received:', items);
    wsAllPlayersItems.set(items);
    
    // Analyze builds for each player
    const analysis: Record<string, any> = {};
    Object.entries(items).forEach(([playerId, playerItems]) => {
      analysis[playerId] = analyzePlayerBuild(playerItems);
    });
    wsBuildAnalysis.set(analysis);
  }
  
  if (message.BuildRecommendation) {
    const { player_id, recommendations } = message.BuildRecommendation;
    console.log('[WebSocket] BuildRecommendation received for player:', player_id);
    wsItemRecommendations.update(currentRecs => ({
      ...currentRecs,
      [player_id]: recommendations
    }));
  }

  if (message.MatchHistoryUpdate) {
    const { player_id, analysis } = message.MatchHistoryUpdate;
    console.log('[WebSocket] MatchHistoryUpdate received for player:', player_id);
    wsMatchHistory.update(current => ({
      ...current,
      [player_id]: analysis
    }));
    wsMatchHistoryData.update(current => ({
      ...current,
      [player_id]: analysis
    }));
    
    // Update match history store (dynamic import to avoid circular dependency)
    import('./matchHistoryStore').then(({ matchHistoryActions }) => {
      matchHistoryActions.updateMatchHistory(player_id, analysis);
    });
  }

  if (message.PlayerStatsAnalysis) {
    const { player_id, stats } = message.PlayerStatsAnalysis;
    console.log('[WebSocket] PlayerStatsAnalysis received for player:', player_id);
    wsPlayerStatsAnalysis.update(current => ({
      ...current,
      [player_id]: stats
    }));
  }

  if (message.AllPlayersRunesUpdate) {
    const { runes } = message.AllPlayersRunesUpdate;
    console.log('[WebSocket] AllPlayersRunesUpdate received');
    wsAllPlayersRunes.set(runes);
    
    // Update rune analysis store
    import('./runeAnalysisStore').then(({ runeAnalysisActions }) => {
      runeAnalysisActions.updateAllPlayersRunes(runes);
    });
  }

  if (message.RuneAnalysisUpdate) {
    const { player_id, analysis } = message.RuneAnalysisUpdate;
    console.log('[WebSocket] RuneAnalysisUpdate received for player:', player_id);
    wsRuneAnalysisData.update(current => ({
      ...current,
      [player_id]: analysis
    }));
    
    // Update rune analysis store
    import('./runeAnalysisStore').then(({ runeAnalysisActions }) => {
      runeAnalysisActions.updateRuneAnalysis(player_id, analysis);
    });
  }

  if (message.RuneRecommendation) {
    const { player_id, recommendations } = message.RuneRecommendation;
    console.log('[WebSocket] RuneRecommendation received for player:', player_id);
    wsRuneRecommendations.update(current => ({
      ...current,
      [player_id]: recommendations
    }));
    
    // Update rune analysis store
    import('./runeAnalysisStore').then(({ runeAnalysisActions }) => {
      runeAnalysisActions.updateRuneRecommendations(player_id, recommendations);
    });
  }

  if (message.LobbyUpdate) {
    const { data } = message.LobbyUpdate;
    console.log('[WebSocket] LobbyUpdate received');
    wsLobbyData.set(data);
    wsLobbyStatus.set(data);
    
    // Update lobby store
    import('./lobbyStore').then(({ lobbyActions }) => {
      lobbyActions.updateLobbyData(data);
      lobbyActions.updateLobbyStatus(data);
    });
  }

  if (message.LobbyAnalysis) {
    const { analysis } = message.LobbyAnalysis;
    console.log('[WebSocket] LobbyAnalysis received');
    wsLobbyAnalysis.set(analysis);
    
    // Update lobby store
    import('./lobbyStore').then(({ lobbyActions }) => {
      lobbyActions.updateLobbyAnalysis(analysis);
    });
  }

  if (message.ReadyCheckUpdate) {
    const { data } = message.ReadyCheckUpdate;
    console.log('[WebSocket] ReadyCheckUpdate received');
    wsReadyCheckData.set(data);
    
    // Update lobby store
    import('./lobbyStore').then(({ lobbyActions }) => {
      lobbyActions.updateReadyCheckData(data);
    });
  }

  if (message.ReadyCheckAction) {
    const { action, result } = message.ReadyCheckAction;
    console.log('[WebSocket] ReadyCheckAction received:', action);
    // Handle ready check action result
    // This could update UI state or show notifications
  }

  if (message.LCUWebSocketEvent) {
    const { event } = message.LCUWebSocketEvent;
    console.log('[WebSocket] LCU WebSocket event received:', event);
    
    // Add to LCU WebSocket events store
    wsLCUWebSocketEvents.update(events => {
      const newEvent = {
        ...event,
        timestamp: Date.now(),
        id: Math.random().toString(36).substr(2, 9)
      };
      return [newEvent, ...events].slice(0, 100); // Keep last 100 events
    });
    
    // Update LCU WebSocket connection status
    wsLCUWebSocketConnected.set(true);
  }

  if (message.ConnectionHealthUpdate) {
    const { health } = message.ConnectionHealthUpdate;
    console.log('[WebSocket] Connection health update received:', health);
    
    // Update connection health store
    wsConnectionHealth.set(health);
    
    // Update LCU WebSocket connection status
    if (health.websocket_connected !== undefined) {
      wsLCUWebSocketConnected.set(health.websocket_connected);
    }
    
    // Update error state based on health
    if (!health.is_healthy && health.last_error) {
      setStableError(`Connection unhealthy: ${health.last_error}`);
    } else if (health.is_healthy) {
      setStableError(null);
    }
  }
  
  if (message.Ping) {
    console.log('[WebSocket] Ping received, sending pong');
    sendMessage({ Pong: {} });
  }
  
  if (message.Pong) {
    console.log('[WebSocket] Pong received - connection is active');
    // Ensure connection status is set to true when we receive pong
    wsConnected.set(true);
  }
}

// Helper function to analyze player build (moved from itemTrackingStore for WebSocket use)
function analyzePlayerBuild(playerItems: any) {
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
  const recommendations = generateRecommendations(completedItems);

  return {
    totalItems,
    completedItems: completedCount,
    goldEfficiency: Math.round(goldEfficiency),
    buildPath,
    recommendations,
    buildStage: completedCount < 2 ? 'early' : completedCount < 4 ? 'mid' : 'late'
  };
}

// Helper function to generate recommendations
function generateRecommendations(currentItems: any[]) {
  const recommendations = [];
  
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
}


// Store management
export function startWebSocketListening() {
  if (isListening) {
    console.log('[WebSocket] Already listening, skipping start');
    return;
  }
  
  isListening = true;
  console.log('[WebSocket] Starting WebSocket listening...');
  
  // Start WebSocket server first
  startWebSocketServer().then((serverStarted) => {
    if (serverStarted) {
      // Connect to WebSocket
      connectWebSocket().catch((error) => {
        console.error('[WebSocket] Failed to connect:', error);
        scheduleReconnect();
      });
    }
  });
}

export function stopWebSocketListening() {
  if (!isListening) return;
  
  isListening = false;
  console.log('[WebSocket] Stopping WebSocket listening...');
  
  // Disconnect WebSocket
  disconnectWebSocket();
  
  // Stop game time ticker
  stopGameTimeTicker();
  
  // Clear all data
  wsGameData.set(null);
  wsLiveGameStats.set(null);
  wsPlayerScores.set(null);
  wsGameState.set(null);
  wsError.set(null);
  wsGameTime.set('00:00');
  wsGamePhase.set('No Live Game');
  wsGameMode.set('Unknown');
  wsIsSpectatorMode.set(false);
  
  // Reset tracking variables
  wsGameStartTime = null;
  wsLastApiGameTime = null;
  wsLastApiSyncTime = null;
  
  // Stop WebSocket server
  stopWebSocketServer();
}

export function isWebSocketActive(): boolean {
  return isListening;
}

// Action functions
export async function refreshWebSocketData() {
  // Send ping to trigger server to send latest data
  sendMessage({ Ping: {} });
}

// Enhanced LCU API functions
export async function refreshGameStateFromBackend() {
  try {
    const state = (await invoke('get_comprehensive_game_state')) as ComprehensiveGameState;
    applyGameSnapshot(state);
    return state;
  } catch (error) {
    console.error('[WebSocket] Failed to refresh game state:', error);
    throw error;
  }
}

export async function detectLeagueClientWithRetry(maxAttempts: number = 5) {
  try {
    const result = await invoke('detect_league_client_with_retry', { maxAttempts });
    console.log('[WebSocket] League client detected with retry:', result);
    await refreshGameStateFromBackend().catch(() => {});
    return result;
  } catch (error) {
    console.error('[WebSocket] Failed to detect League client with retry:', error);
    throw error;
  }
}

export async function setConnectionWithWebSocket(connection: any) {
  try {
    await invoke('set_connection_with_websocket', { connection });
    console.log('[WebSocket] Connection set with WebSocket support');
  } catch (error) {
    console.error('[WebSocket] Failed to set connection with WebSocket:', error);
    throw error;
  }
}

export async function checkConnectionHealth() {
  try {
    const isHealthy = await invoke('check_connection_health');
    console.log('[WebSocket] Connection health check:', isHealthy);
    return isHealthy;
  } catch (error) {
    console.error('[WebSocket] Failed to check connection health:', error);
    throw error;
  }
}

export async function getConnectionHealthInfo() {
  try {
    const healthInfo = await invoke('get_connection_health_info');
    console.log('[WebSocket] Connection health info:', healthInfo);
    wsConnectionHealth.set(healthInfo);
    const h = healthInfo as Record<string, unknown>;
    if (h?.is_healthy === true || h?.healthy === true) {
      setStableError(null);
    }
    return healthInfo;
  } catch (error) {
    console.error('[WebSocket] Failed to get connection health info:', error);
    throw error;
  }
}

export async function getWebSocketStatus() {
  try {
    const status = await invoke('get_websocket_status');
    console.log('[WebSocket] WebSocket status:', status);
    return status;
  } catch (error) {
    console.error('[WebSocket] Failed to get WebSocket status:', error);
    throw error;
  }
}

export async function disconnectWebSocket() {
  try {
    await invoke('disconnect_websocket');
    console.log('[WebSocket] WebSocket disconnected');
    wsLCUWebSocketConnected.set(false);
  } catch (error) {
    console.error('[WebSocket] Failed to disconnect WebSocket:', error);
    throw error;
  }
}

// LCU WebSocket functions
export async function connectLcuWebSocket() {
  try {
    console.log('[LCU WebSocket] Connecting...');
    await invoke('connect_lcu_websocket');
    console.log('[LCU WebSocket] Connected successfully');
    wsLCUWebSocketConnected.set(true);
  } catch (error) {
    console.error('[LCU WebSocket] Failed to connect:', error);
    throw error;
  }
}

export async function disconnectLcuWebSocket() {
  try {
    await invoke('disconnect_lcu_websocket');
    console.log('[LCU WebSocket] Disconnected');
    wsLCUWebSocketConnected.set(false);
  } catch (error) {
    console.error('[LCU WebSocket] Failed to disconnect:', error);
    throw error;
  }
}

export async function isLcuWebSocketConnected(): Promise<boolean> {
  try {
    const connected = await invoke('is_lcu_websocket_connected') as boolean;
    wsLCUWebSocketConnected.set(connected);
    return connected;
  } catch (error) {
    console.error('[LCU WebSocket] Failed to check connection status:', error);
    return false;
  }
}

/** Full debug payload: WingBot WS client + Tauri `debug_lcu_connection` (never throws). */
export async function debugWebSocketConnection() {
  console.log('🔍 Building debug snapshot (WS + LCU)...');
  try {
    await getWebSocketServerStatus();
  } catch (e) {
    console.warn('[debug] getWebSocketServerStatus failed:', e);
  }

  const wingbot_websocket = {
    server_running: get(wsServerRunning),
    server_port: get(wsServerPort),
    connected: get(wsConnected),
    connecting: get(wsConnecting),
    reconnect_attempts: get(wsReconnectAttempts),
    error: get(wsError),
  };

  let lcu: unknown;
  try {
    lcu = await invoke('debug_lcu_connection');
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    console.error('🔍 debug_lcu_connection failed:', msg);
    lcu = { success: false, error: msg };
  }

  console.log('🔍 Debug snapshot ready');
  return { wingbot_websocket, lcu };
}


// Export derived stores for computed values
export const wsSyncStatus = derived([], () => getSyncStatus());

// Export action functions
export const wsActions = {
  refresh: refreshWebSocketData,
  debugConnection: debugWebSocketConnection,
  getSyncStatus,
  formatGameTime,
  startWebSocketListening,
  stopWebSocketListening,
  connectWebSocket,
  // Enhanced LCU API functions
  detectLeagueClientWithRetry,
  refreshGameStateFromBackend,
  setConnectionWithWebSocket,
  checkConnectionHealth,
  getConnectionHealthInfo,
  getWebSocketStatus,
  disconnectWebSocket,
  // LCU WebSocket functions
  connectLcuWebSocket,
  disconnectLcuWebSocket,
  isLcuWebSocketConnected
};
