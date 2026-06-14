<script lang="ts">
  import ChampionSelectOverlay from '$lib/components/champion-select/ChampionSelectOverlay.svelte';
  import { onMount } from 'svelte';
  import { 
    wsConnected,
    wsLCUConnecting,
    wsChampionSelectData,
    wsActions,
    startWebSocketListening,
    stopWebSocketListening
  } from '$lib/stores/websocketStore';
  
  let isConnected = false;
  let isLCUConnecting = false;
  let connectionStatus = 'Disconnected';
  let championSelectData: any = null;
  
  onMount(() => {
    // Subscribe to WebSocket data
    const unsub1 = wsConnected.subscribe((connected: boolean) => {
      isConnected = connected;
      connectionStatus = connected ? 'Connected' : 'Disconnected';
    });
    
    const unsub2 = wsLCUConnecting.subscribe((connecting: boolean) => {
      isLCUConnecting = connecting;
      if (connecting) {
        connectionStatus = 'Connecting...';
      }
    });
    
    const unsub3 = wsChampionSelectData.subscribe((data: any) => {
      championSelectData = data;
      if (data) {
        isConnected = true;
        connectionStatus = 'In Champion Select';
      }
    });
    
    // Start WebSocket listening
    startWebSocketListening();
    
    return () => {
      unsub1();
      unsub2();
      unsub3();
      stopWebSocketListening();
    };
  });
  
  function handleRefresh() {
    wsActions.refresh();
  }
</script>

<svelte:head>
  <title>Champion Select - WingBot</title>
</svelte:head>

<div class="champion-select-page">
  <div class="page-header">
    <h1 class="page-title">
      <span class="page-icon">⚔️</span>
      Champion Select
    </h1>
    <div class="connection-status">
      <div class="status-indicator {isConnected ? 'online' : (isLCUConnecting ? 'connecting' : 'offline')}"></div>
      <span class="status-text">{connectionStatus}</span>
      <button class="refresh-btn" onclick={handleRefresh} title="Refresh Connection">
        🔄
      </button>
    </div>
  </div>
  
  <div class="content-area">
    {#if isConnected}
      <div class="overlay-container animate-fade-in">
        <ChampionSelectOverlay />
      </div>
    {:else}
      <div class="loading-state animate-fade-in">
        <div class="loading-spinner"></div>
        <h2>Connecting to League Client...</h2>
        <p>Please make sure League of Legends is running and you're in champion select.</p>
      </div>
    {/if}
  </div>
  
  <div class="page-footer">
    <div class="info-cards">
      <div class="info-card">
        <div class="card-icon">🎯</div>
        <div class="card-content">
          <h3>AI Recommendations</h3>
          <p>Get intelligent champion suggestions based on team composition and enemy picks.</p>
        </div>
      </div>
      
      <div class="info-card">
        <div class="card-icon">📊</div>
        <div class="card-content">
          <h3>Real-time Analysis</h3>
          <p>Live updates on pick/ban phases with strategic insights and counter-picks.</p>
        </div>
      </div>
      
      <div class="info-card">
        <div class="card-icon">⚡</div>
        <div class="card-content">
          <h3>Instant Feedback</h3>
          <p>Immediate analysis of team compositions and synergy recommendations.</p>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .champion-select-page {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    background: var(--bg-primary);
    color: var(--text-primary);
  }
  
  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 2rem;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }
  
  .page-title {
    display: flex;
    align-items: center;
    gap: 1rem;
    font-size: 2rem;
    font-weight: var(--font-weight-bold);
    margin: 0;
    color: var(--accent-color);
  }
  
  .page-icon {
    font-size: 2.5rem;
  }
  
  .connection-status {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1.5rem;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
  }
  
  .refresh-btn {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: 0.5rem;
    cursor: pointer;
    font-size: 1rem;
    transition: var(--transition-normal);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .refresh-btn:hover {
    background: var(--bg-hover);
    border-color: var(--border-hover);
    transform: rotate(180deg);
  }
  
  .status-indicator {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #666;
  }
  
  .status-indicator.online {
    background: var(--success-color);
    animation: pulse 2s infinite;
  }
  
  .status-indicator.connecting {
    background: var(--warning-color);
    animation: pulse 1s infinite;
  }
  
  .status-indicator.offline {
    background: var(--error-color);
  }
  
  .status-text {
    font-weight: var(--font-weight-medium);
    color: var(--text-secondary);
  }
  
  .content-area {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
  }
  
  .overlay-container {
    width: 100%;
    max-width: 1200px;
  }
  
  .loading-state {
    text-align: center;
    padding: 4rem 2rem;
  }
  
  .loading-spinner {
    width: 60px;
    height: 60px;
    border: 4px solid var(--border-color);
    border-top: 4px solid var(--accent-color);
    border-radius: 50%;
    animation: rotate 1s linear infinite;
    margin: 0 auto 2rem;
  }
  
  .loading-state h2 {
    font-size: 1.5rem;
    font-weight: var(--font-weight-semibold);
    margin-bottom: 1rem;
    color: var(--text-primary);
  }
  
  .loading-state p {
    color: var(--text-secondary);
    font-size: var(--font-size-lg);
  }
  
  .page-footer {
    padding: 2rem;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
  }
  
  .info-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1.5rem;
    max-width: 1200px;
    margin: 0 auto;
  }
  
  .info-card {
    display: flex;
    align-items: flex-start;
    gap: 1rem;
    padding: 1.5rem;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    transition: var(--transition-normal);
  }
  
  .info-card:hover {
    border-color: var(--border-hover);
    box-shadow: var(--shadow-lg);
    transform: translateY(-2px);
  }
  
  .card-icon {
    font-size: 2rem;
    flex-shrink: 0;
  }
  
  .card-content h3 {
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-semibold);
    margin: 0 0 0.5rem 0;
    color: var(--text-primary);
  }
  
  .card-content p {
    color: var(--text-secondary);
    line-height: var(--line-height-relaxed);
    margin: 0;
  }
  
  /* Responsive Design */
  @media (max-width: 768px) {
    .page-header {
      flex-direction: column;
      gap: 1rem;
      text-align: center;
    }
    
    .page-title {
      font-size: 1.5rem;
    }
    
    .content-area {
      padding: 1rem;
    }
    
    .info-cards {
      grid-template-columns: 1fr;
    }
    
    .info-card {
      flex-direction: column;
      text-align: center;
    }
  }
</style>