<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { ScrollArea } from '$lib/components/ui/scroll-area';
  import { Badge } from '$lib/components/ui/badge';
  import { 
    Send, 
    Bot, 
    User, 
    Loader2, 
    MessageSquare,
    Brain,
    Sparkles,
    Gamepad2,
    Target,
    Zap,
    Shield,
    Sword,
    Eye,
    Clock,
    TrendingUp,
    AlertTriangle,
    CheckCircle,
    XCircle,
    Volume2,
    VolumeX,
    Play,
    Pause,
    Mic,
    MicOff
  } from '@lucide/svelte';
  
  // Import WebSocket stores for real-time game data
  import { 
    wsGameData, 
    wsLiveGameStats, 
    wsGameMode, 
    wsGameTime,
    wsGameEvents,
    wsConnected,
    wsActions
  } from '$lib/stores/websocketStore';

  let messages = $state<Array<{
    id: string;
    role: 'user' | 'assistant' | 'system';
    content: string;
    timestamp: Date;
    confidence?: number;
    context?: any;
    suggestions?: string[];
  }>>([]);
  
  let currentMessage = $state('');
  let isLoading = $state(false);
  let aiServiceStatus = $state<string>('Unknown');
  let isMinimized = $state(false);
  let gameContext = $state<any>(null);
  let smartSuggestions = $state<string[]>([]);
  
  // Voice/TTS functionality
  let isVoiceEnabled = $state(true);
  let isPlayingAudio = $state(false);
  let currentAudio: HTMLAudioElement | null = null;
  let isRecording = $state(false);
  let voiceMode = $state<'auto' | 'manual' | 'off'>('auto'); // auto = voice for in-game, manual = always ask, off = never

  // Reactive game data
  let gameData = $state($wsGameData);
  let gameStats = $state($wsLiveGameStats);
  let gameMode = $state($wsGameMode);
  let gameTime = $state($wsGameTime);
  let gameEvents = $state($wsGameEvents);
  let isConnected = $state($wsConnected);

  // Update reactive variables when stores change
  $effect(() => {
    gameData = $wsGameData;
    gameStats = $wsLiveGameStats;
    gameMode = $wsGameMode;
    gameTime = $wsGameTime;
    gameEvents = $wsGameEvents;
    isConnected = $wsConnected;
    updateGameContext();
  });

  onMount(async () => {
    try {
      interface SystemInfo {
        ai_service_status: string;
        league_running: boolean;
        os: string;
        arch: string;
      }
      const systemInfo = await invoke<SystemInfo>('get_system_info');
      aiServiceStatus = systemInfo.ai_service_status;
      
      // Add welcome message with context
      addSystemMessage("AI Assistant ready! I can help with champion picks, builds, strategy, and real-time game analysis.");
      
      // Start WebSocket connection if not already connected
      if (!isConnected) {
        await wsActions.connectWebSocket();
      }
    } catch (error) {
      console.error('Failed to get system info:', error);
    }
  });

  function updateGameContext() {
    if (gameData && gameStats) {
      gameContext = {
        gameMode: gameMode || 'Unknown',
        gameTime: gameTime || 0,
        playerCount: gameData.allPlayers?.length || 0,
        events: gameEvents || [],
        stats: gameStats,
        isInGame: gameData.allPlayers && gameData.allPlayers.length > 0
      };
      
      // Generate smart suggestions based on game context
      generateSmartSuggestions();
    }
  }

  function generateSmartSuggestions() {
    if (!gameContext) return;
    
    const suggestions = [];
    
    if (gameContext.isInGame) {
      if (gameContext.gameTime < 300) { // Early game
        suggestions.push("What should I build first?", "How should I play the early game?", "What runes should I take?");
      } else if (gameContext.gameTime < 900) { // Mid game
        suggestions.push("What items should I buy next?", "How should I teamfight?", "What objectives should we focus on?");
      } else { // Late game
        suggestions.push("How should I position in teamfights?", "What's the best strategy for late game?", "How do I close out the game?");
      }
      
      if (gameContext.events && gameContext.events.length > 0) {
        const recentEvent = gameContext.events[0];
        if (recentEvent.type === 'CHAMPION_KILL') {
          suggestions.push("How should I respond to this kill?", "What should I do after this death?");
        }
      }
    } else {
      suggestions.push("What champion should I pick?", "How do I counter this team comp?", "What's the best ban for this meta?");
    }
    
    smartSuggestions = suggestions.slice(0, 3);
  }

  function addSystemMessage(content: string) {
    const systemMessage = {
      id: Date.now().toString(),
      role: 'system' as const,
      content,
      timestamp: new Date()
    };
    messages = [...messages, systemMessage];
  }

  async function sendMessage() {
    if (!currentMessage.trim() || isLoading) return;

    const userMessage = {
      id: Date.now().toString(),
      role: 'user' as const,
      content: currentMessage.trim(),
      timestamp: new Date()
    };

    messages = [...messages, userMessage];
    const messageToSend = currentMessage.trim();
    currentMessage = '';
    isLoading = true;

    try {
      // Create context-aware request with game data
      const contextData = {
        gameMode: gameContext?.gameMode,
        gameTime: gameContext?.gameTime,
        isInGame: gameContext?.isInGame,
        playerCount: gameContext?.playerCount,
        recentEvents: gameContext?.events?.slice(0, 5), // Last 5 events
        gameStats: gameContext?.stats
      };

      interface AIResponse {
        response: string;
        confidence?: number;
        suggestions?: string[];
        tts_summary?: string;
      }
      const response = await invoke<AIResponse>('ai_chat', {
        message: messageToSend,
        sessionId: 'wingbot-session',
        context: contextData
      });

      const assistantMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant' as const,
        content: response.response || 'No response received',
        timestamp: new Date(),
        confidence: response.confidence,
        context: contextData,
        suggestions: response.suggestions || []
      };

      messages = [...messages, assistantMessage];
      
      // Play TTS if voice is enabled and appropriate
      if (shouldUseVoice() && response.tts_summary) {
        await playTTS(response.tts_summary);
      } else if (shouldUseVoice() && !response.tts_summary) {
        // Fallback to main response if no TTS summary
        await playTTS(response.response || 'No response received');
      }
    } catch (error) {
      console.error('Failed to send message:', error);
      const errorMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant' as const,
        content: 'Sorry, I encountered an error. Please try again.',
        timestamp: new Date()
      };
      messages = [...messages, errorMessage];
    } finally {
      isLoading = false;
    }
  }

  function sendSuggestion(suggestion: string) {
    currentMessage = suggestion;
    sendMessage();
  }

  async function playTTS(text: string) {
    if (!isVoiceEnabled || voiceMode === 'off') return;
    
    try {
      // Stop any currently playing audio
      if (currentAudio) {
        currentAudio.pause();
        currentAudio = null;
      }
      
      isPlayingAudio = true;
      
      // Generate TTS audio from backend
      const audioData = await invoke<number[]>('generate_tts_audio', { text });
      
      // Create audio blob and play
      const blob = new Blob([new Uint8Array(audioData)], { type: 'audio/wav' });
      const audioUrl = URL.createObjectURL(blob);
      
      currentAudio = new Audio(audioUrl);
      currentAudio.onended = () => {
        isPlayingAudio = false;
        URL.revokeObjectURL(audioUrl);
        currentAudio = null;
      };
      currentAudio.onerror = () => {
        isPlayingAudio = false;
        URL.revokeObjectURL(audioUrl);
        currentAudio = null;
      };
      
      await currentAudio.play();
    } catch (error) {
      console.error('Failed to play TTS:', error);
      isPlayingAudio = false;
    }
  }

  function stopAudio() {
    if (currentAudio) {
      currentAudio.pause();
      currentAudio = null;
    }
    isPlayingAudio = false;
  }

  function toggleVoice() {
    isVoiceEnabled = !isVoiceEnabled;
    if (!isVoiceEnabled) {
      stopAudio();
    }
  }

  function shouldUseVoice(): boolean {
    if (!isVoiceEnabled || voiceMode === 'off') return false;
    if (voiceMode === 'manual') return true;
    if (voiceMode === 'auto') {
      // Use voice for in-game scenarios, especially for critical decisions
      return gameContext?.isInGame || false;
    }
    return false;
  }

  function handleKeyPress(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      sendMessage();
    }
  }
</script>

<div class="flex flex-col h-full bg-background">
  <!-- Floating Header with Game Context -->
  <Card class="border-b border-border rounded-none sticky top-0 z-10 bg-background/95 backdrop-blur-sm">
    <CardHeader class="pb-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg bg-primary flex items-center justify-center">
            <Bot class="w-5 h-5 text-primary-foreground" />
          </div>
          <div>
            <CardTitle class="text-xl">AI Assistant</CardTitle>
            <p class="text-sm text-muted-foreground">
              {#if gameContext?.isInGame}
                Context-aware • {gameContext.gameMode} • {Math.floor(gameContext.gameTime / 60)}:{(gameContext.gameTime % 60).toString().padStart(2, '0')}
              {:else}
                Ready to help with League strategy
              {/if}
            </p>
          </div>
        </div>
        <div class="flex items-center gap-2">
          {#if gameContext?.isInGame}
            <Badge variant="default" class="bg-green-500/20 text-green-600 border-green-500/30">
              <CheckCircle class="w-3 h-3 mr-1" />
              In Game
            </Badge>
          {:else}
            <Badge variant="secondary">
              <XCircle class="w-3 h-3 mr-1" />
              Not in Game
            </Badge>
          {/if}
          <Badge variant={aiServiceStatus === 'Connected' ? 'default' : 'destructive'}>
            {aiServiceStatus}
          </Badge>
          
          <!-- Voice Controls -->
          <div class="flex items-center gap-1">
            <Button 
              variant="ghost" 
              size="sm" 
              onclick={toggleVoice}
              class="p-2"
              title={isVoiceEnabled ? 'Disable Voice' : 'Enable Voice'}
            >
              {#if isVoiceEnabled}
                <Volume2 class="w-4 h-4" />
              {:else}
                <VolumeX class="w-4 h-4" />
              {/if}
            </Button>
            
            {#if isPlayingAudio}
              <Button 
                variant="ghost" 
                size="sm" 
                onclick={stopAudio}
                class="p-2"
                title="Stop Audio"
              >
                <Pause class="w-4 h-4" />
              </Button>
            {/if}
            
            <select 
              bind:value={voiceMode}
              class="text-xs bg-background border border-border rounded px-2 py-1"
              title="Voice Mode"
            >
              <option value="auto">Auto</option>
              <option value="manual">Manual</option>
              <option value="off">Off</option>
            </select>
          </div>
          
          <Button 
            variant="ghost" 
            size="sm" 
            onclick={() => isMinimized = !isMinimized}
            class="p-2"
          >
            {#if isMinimized}
              <Eye class="w-4 h-4" />
            {:else}
              <Eye class="w-4 h-4" />
            {/if}
          </Button>
        </div>
      </div>
    </CardHeader>
  </Card>

  <!-- Smart Suggestions Bar -->
  {#if smartSuggestions.length > 0 && !isMinimized}
    <div class="border-b border-border bg-muted/30 p-3">
      <div class="flex items-center gap-2 mb-2">
        <Zap class="w-4 h-4 text-primary" />
        <span class="text-sm font-medium">Smart Suggestions</span>
        <Badge variant="outline" class="text-xs">
          {gameContext?.isInGame ? 'In Game' : 'Champion Select'}
        </Badge>
        {#if shouldUseVoice()}
          <Badge variant="outline" class="text-xs bg-green-500/20 text-green-600 border-green-500/30">
            <Volume2 class="w-3 h-3 mr-1" />
            Voice Enabled
          </Badge>
        {/if}
      </div>
      <div class="flex flex-wrap gap-2">
        {#each smartSuggestions as suggestion}
          <Button 
            variant="outline" 
            size="sm" 
            onclick={() => sendSuggestion(suggestion)}
            class="text-xs"
          >
            <Sparkles class="w-3 h-3 mr-1" />
            {suggestion}
          </Button>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Chat Messages -->
  {#if !isMinimized}
    <ScrollArea class="flex-1 p-4">
      <div class="space-y-4 max-w-4xl mx-auto">
        {#if messages.length === 0}
          <div class="text-center py-12">
            <div class="w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center mx-auto mb-4">
              <Brain class="w-8 h-8 text-primary" />
            </div>
            <h3 class="text-lg font-semibold mb-2">AI Assistant Ready</h3>
            <p class="text-muted-foreground mb-4">
              {#if gameContext?.isInGame}
                I can help with builds, strategy, and real-time game analysis!
              {:else}
                Ask me about champion picks, counters, builds, or strategy!
              {/if}
            </p>
            
            <!-- Context-aware quick actions -->
            <div class="flex flex-wrap gap-2 justify-center">
              {#if gameContext?.isInGame}
                <Button variant="outline" size="sm" onclick={() => sendSuggestion("What should I build next?")}>
                  <Sword class="w-4 h-4 mr-2" />
                  Build Advice
                </Button>
                <Button variant="outline" size="sm" onclick={() => sendSuggestion("How should I position in teamfights?")}>
                  <Target class="w-4 h-4 mr-2" />
                  Teamfight Tips
                </Button>
                <Button variant="outline" size="sm" onclick={() => sendSuggestion("What objectives should we focus on?")}>
                  <Shield class="w-4 h-4 mr-2" />
                  Objective Strategy
                </Button>
              {:else}
                <Button variant="outline" size="sm" onclick={() => sendSuggestion("What champion should I pick?")}>
                  <Gamepad2 class="w-4 h-4 mr-2" />
                  Champion Picks
                </Button>
                <Button variant="outline" size="sm" onclick={() => sendSuggestion("How do I counter this team comp?")}>
                  <Target class="w-4 h-4 mr-2" />
                  Counter Strategy
                </Button>
                <Button variant="outline" size="sm" onclick={() => sendSuggestion("What's the best ban for this meta?")}>
                  <AlertTriangle class="w-4 h-4 mr-2" />
                  Ban Recommendations
                </Button>
              {/if}
            </div>
          </div>
        {/if}

        {#each messages as message}
          <div class="flex {message.role === 'user' ? 'justify-end' : 'justify-start'}">
            <div class="flex gap-3 max-w-[80%] {message.role === 'user' ? 'flex-row-reverse' : 'flex-row'}">
              <div class="w-8 h-8 rounded-full flex items-center justify-center {message.role === 'user' ? 'bg-primary text-primary-foreground' : message.role === 'system' ? 'bg-blue-500 text-white' : 'bg-muted'}">
                {#if message.role === 'user'}
                  <User class="w-4 h-4" />
                {:else if message.role === 'system'}
                  <Brain class="w-4 h-4" />
                {:else}
                  <Bot class="w-4 h-4" />
                {/if}
              </div>
              <div class="flex flex-col gap-1">
                <Card class={message.role === 'user' ? 'bg-primary text-primary-foreground' : message.role === 'system' ? 'bg-blue-500/10 border-blue-500/20' : 'bg-muted'}>
                  <CardContent class="p-3">
                    <p class="text-sm whitespace-pre-wrap">{message.content}</p>
                    
                    <!-- Show context for assistant messages -->
                    {#if message.role === 'assistant' && message.context}
                      <div class="mt-2 pt-2 border-t border-border/50">
                        <div class="flex items-center gap-1 text-xs text-muted-foreground mb-1">
                          <Eye class="w-3 h-3" />
                          Context: {message.context.gameMode} • {Math.floor(message.context.gameTime / 60)}:{(message.context.gameTime % 60).toString().padStart(2, '0')}
                        </div>
                      </div>
                    {/if}
                    
                    <!-- Show suggestions for assistant messages -->
                    {#if message.role === 'assistant' && message.suggestions && message.suggestions.length > 0}
                      <div class="mt-2 pt-2 border-t border-border/50">
                        <div class="flex flex-wrap gap-1">
                          {#each message.suggestions as suggestion}
                            <Button 
                              variant="outline" 
                              size="sm" 
                              onclick={() => sendSuggestion(suggestion)}
                              class="text-xs h-6 px-2"
                            >
                              {suggestion}
                            </Button>
                          {/each}
                        </div>
                      </div>
                    {/if}
                    
                    <!-- Voice playback for assistant messages -->
                    {#if message.role === 'assistant' && isVoiceEnabled}
                      <div class="mt-2 pt-2 border-t border-border/50">
                        <div class="flex items-center gap-2">
                          <Button 
                            variant="ghost" 
                            size="sm" 
                            onclick={() => playTTS(message.content)}
                            class="text-xs h-6 px-2"
                            disabled={isPlayingAudio}
                          >
                            {#if isPlayingAudio}
                              <Pause class="w-3 h-3 mr-1" />
                              Playing...
                            {:else}
                              <Play class="w-3 h-3 mr-1" />
                              Play Voice
                            {/if}
                          </Button>
                          {#if shouldUseVoice()}
                            <Badge variant="outline" class="text-xs bg-blue-500/20 text-blue-600 border-blue-500/30">
                              <Volume2 class="w-3 h-3 mr-1" />
                              Auto-voice
                            </Badge>
                          {/if}
                        </div>
                      </div>
                    {/if}
                  </CardContent>
                </Card>
                <div class="flex items-center gap-2 text-xs text-muted-foreground {message.role === 'user' ? 'justify-end' : 'justify-start'}">
                  <span>{message.timestamp.toLocaleTimeString()}</span>
                  {#if message.confidence}
                    <Badge variant="outline" class="text-xs">
                      {Math.round(message.confidence * 100)}% confidence
                    </Badge>
                  {/if}
                  {#if message.role === 'system'}
                    <Badge variant="outline" class="text-xs bg-blue-500/20 text-blue-600 border-blue-500/30">
                      System
                    </Badge>
                  {/if}
                </div>
              </div>
            </div>
          </div>
        {/each}

        {#if isLoading}
          <div class="flex justify-start">
            <div class="flex gap-3">
              <div class="w-8 h-8 rounded-full bg-muted flex items-center justify-center">
                <Bot class="w-4 h-4" />
              </div>
              <Card class="bg-muted">
                <CardContent class="p-3">
                  <div class="flex items-center gap-2">
                    <Loader2 class="w-4 h-4 animate-spin" />
                    <span class="text-sm text-muted-foreground">AI is analyzing game context...</span>
                  </div>
                </CardContent>
              </Card>
            </div>
          </div>
        {/if}
      </div>
    </ScrollArea>
  {/if}

  <!-- Message Input -->
  {#if !isMinimized}
    <Card class="border-t border-border rounded-none sticky bottom-0 bg-background/95 backdrop-blur-sm">
      <CardContent class="p-4">
        <div class="flex gap-2 max-w-4xl mx-auto">
          <Input
            bind:value={currentMessage}
            placeholder={gameContext?.isInGame ? "Ask about builds, strategy, or teamfights..." : "Ask about champion picks, counters, or builds..."}
            onkeypress={handleKeyPress}
            disabled={isLoading}
            class="flex-1"
          />
          <Button 
            onclick={sendMessage} 
            disabled={!currentMessage.trim() || isLoading}
            size="sm"
          >
            {#if isLoading}
              <Loader2 class="w-4 h-4 animate-spin" />
            {:else}
              <Send class="w-4 h-4" />
            {/if}
          </Button>
        </div>
        
        <!-- Quick context info -->
        {#if gameContext}
          <div class="mt-2 flex items-center justify-center gap-4 text-xs text-muted-foreground">
            <div class="flex items-center gap-1">
              <Clock class="w-3 h-3" />
              {Math.floor(gameContext.gameTime / 60)}:{(gameContext.gameTime % 60).toString().padStart(2, '0')}
            </div>
            <div class="flex items-center gap-1">
              <Gamepad2 class="w-3 h-3" />
              {gameContext.gameMode}
            </div>
            <div class="flex items-center gap-1">
              <TrendingUp class="w-3 h-3" />
              {gameContext.playerCount} players
            </div>
          </div>
        {/if}
      </CardContent>
    </Card>
  {/if}
</div>
