<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  
  let isMaximized = false;
  let isMinimized = false;
  let currentTab = 'dashboard';
  
  const tabs = [
    { id: 'dashboard', label: 'Dashboard', icon: '🏠' },
    { id: 'champion-select', label: 'Champion Select', icon: '⚔️' },
    { id: 'live-game', label: 'Live Game', icon: '🎮' },
    { id: 'match-history', label: 'Match History', icon: '📊' },
    { id: 'ai-chat', label: 'AI Chat', icon: '🤖' },
    { id: 'settings', label: 'Settings', icon: '⚙️' }
  ];
  
  async function minimizeWindow() {
    try {
      await invoke('minimize_window');
      isMinimized = true;
    } catch (error) {
      console.error('Failed to minimize window:', error);
    }
  }
  
  async function maximizeWindow() {
    try {
      if (isMaximized) {
        await invoke('unmaximize_window');
        isMaximized = false;
      } else {
        await invoke('maximize_window');
        isMaximized = true;
      }
    } catch (error) {
      console.error('Failed to maximize window:', error);
    }
  }
  
  async function closeWindow() {
    try {
      await invoke('close_window');
    } catch (error) {
      console.error('Failed to close window:', error);
    }
  }
  
  function setActiveTab(tabId: string) {
    currentTab = tabId;
  }
</script>

<div class="desktop-app">
  <!-- Title Bar -->
  <header class="title-bar">
    <div class="title-bar-content">
      <div class="app-info">
        <img src="/logo.png" class="app-logo" alt="WingBot" />
        <h1 class="app-title">WingBot</h1>
      </div>
      
      <nav class="app-nav">
        {#each tabs as tab}
          <button 
            class="nav-item {currentTab === tab.id ? 'active' : ''}"
            on:click={() => setActiveTab(tab.id)}
          >
            <span class="nav-icon">{tab.icon}</span>
            <span class="nav-label">{tab.label}</span>
          </button>
        {/each}
      </nav>
      
      <div class="window-controls">
        <button class="control-btn minimize" on:click={minimizeWindow} title="Minimize" aria-label="Minimize window">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <rect x="2" y="5" width="8" height="2" fill="currentColor"/>
          </svg>
        </button>
        <button class="control-btn maximize" on:click={maximizeWindow} title={isMaximized ? 'Restore' : 'Maximize'} aria-label={isMaximized ? 'Restore window' : 'Maximize window'}>
          {#if isMaximized}
            <svg width="12" height="12" viewBox="0 0 12 12">
              <rect x="2" y="2" width="6" height="6" fill="none" stroke="currentColor" stroke-width="1"/>
              <rect x="4" y="4" width="6" height="6" fill="none" stroke="currentColor" stroke-width="1"/>
            </svg>
          {:else}
            <svg width="12" height="12" viewBox="0 0 12 12">
              <rect x="1" y="1" width="10" height="10" fill="none" stroke="currentColor" stroke-width="1"/>
            </svg>
          {/if}
        </button>
        <button class="control-btn close" on:click={closeWindow} title="Close" aria-label="Close window">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <path d="M2 2l8 8M10 2l-8 8" stroke="currentColor" stroke-width="1.5" fill="none"/>
          </svg>
        </button>
      </div>
    </div>
  </header>
  
  <!-- Main Content Area -->
  <main class="app-content">
    <div class="content-wrapper">
      <!-- Sidebar -->
      <aside class="sidebar">
        <div class="sidebar-content">
          <div class="connection-status">
            <div class="status-indicator online"></div>
            <span class="status-text">Connected</span>
          </div>
          
          <div class="game-phase">
            <div class="phase-indicator">
              <div class="phase-dot active"></div>
              <span>In Game</span>
            </div>
          </div>
          
          <div class="quick-actions">
            <h3>Quick Actions</h3>
            <button class="quick-action-btn">
              <span class="action-icon">🎯</span>
              <span>AI Recommendations</span>
            </button>
            <button class="quick-action-btn">
              <span class="action-icon">📊</span>
              <span>Match Analysis</span>
            </button>
            <button class="quick-action-btn">
              <span class="action-icon">⚡</span>
              <span>Quick Settings</span>
            </button>
          </div>
        </div>
      </aside>
      
      <!-- Main Content -->
      <div class="main-content">
        <div class="content-header">
          <h2 class="page-title">
            {#each tabs as tab}
              {#if currentTab === tab.id}
                <span class="page-icon">{tab.icon}</span>
                {tab.label}
              {/if}
            {/each}
          </h2>
          <div class="content-actions">
            <button class="action-btn secondary">
              <span class="btn-icon">🔄</span>
              Refresh
            </button>
            <button class="action-btn primary">
              <span class="btn-icon">⚙️</span>
              Settings
            </button>
          </div>
        </div>
        
        <div class="content-body">
          <slot />
        </div>
      </div>
    </div>
  </main>
  
  <!-- Status Bar -->
  <footer class="status-bar">
    <div class="status-left">
      <div class="connection-info">
        <div class="status-indicator online"></div>
        <span>League Client Connected</span>
      </div>
      <div class="game-info">
        <span>Game Time: 15:32</span>
      </div>
    </div>
    
    <div class="status-center">
      <div class="progress-indicator">
        <div class="progress-bar">
          <div class="progress-fill" style="width: 75%"></div>
        </div>
        <span>Loading game data...</span>
      </div>
    </div>
    
    <div class="status-right">
      <div class="system-info">
        <span>AI Service: Online</span>
        <span>•</span>
        <span>WebSocket: Connected</span>
      </div>
    </div>
  </footer>
</div>

<style>
  .desktop-app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-primary, #0d1117);
    color: var(--text-primary, #f0f6fc);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    overflow: hidden;
  }
  
  /* Title Bar */
  .title-bar {
    background: var(--bg-secondary, #161b22);
    border-bottom: 1px solid var(--border-color, #30363d);
    -webkit-app-region: drag;
    user-select: none;
  }
  
  .title-bar-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 40px;
    padding: 0 1rem;
  }
  
  .app-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .app-logo {
    width: 24px;
    height: 24px;
  }
  
  .app-title {
    font-size: 1rem;
    font-weight: 600;
    margin: 0;
  }
  
  .app-nav {
    display: flex;
    gap: 0.25rem;
    -webkit-app-region: no-drag;
  }
  
  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: transparent;
    border: none;
    color: var(--text-secondary, #8b949e);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s ease;
    font-size: 0.875rem;
  }
  
  .nav-item:hover {
    background: var(--bg-hover, #21262d);
    color: var(--text-primary, #f0f6fc);
  }
  
  .nav-item.active {
    background: var(--accent-color, #4a90e2);
    color: white;
  }
  
  .nav-icon {
    font-size: 1rem;
  }
  
  .nav-label {
    font-weight: 500;
  }
  
  .window-controls {
    display: flex;
    gap: 0.25rem;
    -webkit-app-region: no-drag;
  }
  
  .control-btn {
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    color: var(--text-secondary, #8b949e);
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
  }
  
  .control-btn:hover {
    background: var(--bg-hover, #21262d);
    color: var(--text-primary, #f0f6fc);
  }
  
  .control-btn.close:hover {
    background: #e74c3c;
    color: white;
  }
  
  /* Main Content */
  .app-content {
    flex: 1;
    display: flex;
    overflow: hidden;
  }
  
  .content-wrapper {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
  
  .sidebar {
    width: 250px;
    background: var(--bg-secondary, #161b22);
    border-right: 1px solid var(--border-color, #30363d);
    display: flex;
    flex-direction: column;
  }
  
  .sidebar-content {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  
  .connection-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    background: var(--bg-primary, #0d1117);
    border-radius: 8px;
    border: 1px solid var(--border-color, #30363d);
  }
  
  .status-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #666;
  }
  
  .status-indicator.online {
    background: #27ae60;
    animation: pulse 2s infinite;
  }
  
  .status-text {
    font-size: 0.875rem;
    font-weight: 500;
  }
  
  .game-phase {
    padding: 0.75rem;
    background: var(--bg-primary, #0d1117);
    border-radius: 8px;
    border: 1px solid var(--border-color, #30363d);
  }
  
  .phase-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
  }
  
  .phase-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #666;
  }
  
  .phase-dot.active {
    background: var(--accent-color, #4a90e2);
    box-shadow: 0 0 8px var(--accent-color, #4a90e2);
  }
  
  .quick-actions h3 {
    font-size: 0.875rem;
    font-weight: 600;
    margin: 0 0 0.75rem 0;
    color: var(--text-secondary, #8b949e);
  }
  
  .quick-action-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.5rem;
    background: transparent;
    border: none;
    color: var(--text-primary, #f0f6fc);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s ease;
    font-size: 0.875rem;
    text-align: left;
  }
  
  .quick-action-btn:hover {
    background: var(--bg-hover, #21262d);
  }
  
  .action-icon {
    font-size: 1rem;
  }
  
  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  
  .content-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid var(--border-color, #30363d);
    background: var(--bg-secondary, #161b22);
  }
  
  .page-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1.25rem;
    font-weight: 600;
    margin: 0;
  }
  
  .page-icon {
    font-size: 1.5rem;
  }
  
  .content-actions {
    display: flex;
    gap: 0.5rem;
  }
  
  .action-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border: 1px solid var(--border-color, #30363d);
    border-radius: 6px;
    background: transparent;
    color: var(--text-primary, #f0f6fc);
    cursor: pointer;
    transition: all 0.2s ease;
    font-size: 0.875rem;
  }
  
  .action-btn:hover {
    background: var(--bg-hover, #21262d);
  }
  
  .action-btn.primary {
    background: var(--accent-color, #4a90e2);
    border-color: var(--accent-color, #4a90e2);
    color: white;
  }
  
  .action-btn.primary:hover {
    background: var(--accent-hover, #357abd);
  }
  
  .btn-icon {
    font-size: 1rem;
  }
  
  .content-body {
    flex: 1;
    padding: 1.5rem;
    overflow: auto;
  }
  
  /* Status Bar */
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 32px;
    padding: 0 1rem;
    background: var(--bg-secondary, #161b22);
    border-top: 1px solid var(--border-color, #30363d);
    font-size: 0.75rem;
    color: var(--text-secondary, #8b949e);
  }
  
  .status-left,
  .status-right {
    display: flex;
    align-items: center;
    gap: 1rem;
  }
  
  .connection-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .progress-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .progress-bar {
    width: 100px;
    height: 4px;
    background: var(--bg-primary, #0d1117);
    border-radius: 2px;
    overflow: hidden;
  }
  
  .progress-fill {
    height: 100%;
    background: var(--accent-color, #4a90e2);
    transition: width 0.3s ease;
  }
  
  .system-info {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  /* Responsive Design */
  @media (max-width: 1024px) {
    .sidebar {
      width: 200px;
    }
    
    .nav-label {
      display: none;
    }
    
    .nav-item {
      padding: 0.5rem;
    }
  }
  
  @media (max-width: 768px) {
    .sidebar {
      display: none;
    }
    
    .content-header {
      padding: 1rem;
    }
    
    .content-body {
      padding: 1rem;
    }
  }
</style>
