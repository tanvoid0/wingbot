<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  interface LeagueRegion {
    name: string;
    patchline: string;
    client_config_url: string;
    chat_host: string;
    chat_port: number;
  }

  interface LeagueInstallation {
    path: string;
    is_pbe: boolean;
    is_esports: boolean;
  }

  let regions: LeagueRegion[] = [];
  let installations: LeagueInstallation[] = [];
  let selectedRegion = '';
  let customArgs = '';
  let isLoading = false;
  let error = '';

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    try {
      isLoading = true;
      error = '';
      
      // Load regions and installations in parallel
      const [regionsResult, installationsResult] = await Promise.all([
        invoke<LeagueRegion[]>('get_available_regions'),
        invoke<LeagueInstallation[]>('detect_league_installations')
      ]);
      
      regions = regionsResult;
      installations = installationsResult;
      
      // Set default region if available
      if (regions.length > 0 && !selectedRegion) {
        selectedRegion = regions[0].name;
      }
      
    } catch (err) {
      error = `Failed to load data: ${err}`;
      console.error('Error loading League data:', err);
    } finally {
      isLoading = false;
    }
  }

  async function launchLeague() {
    if (!selectedRegion) {
      error = 'Please select a region';
      return;
    }

    try {
      isLoading = true;
      error = '';
      
      const args = customArgs.split(',').map(arg => arg.trim()).filter(arg => arg.length > 0);
      
      await invoke('launch_league_of_legends', {
        region: selectedRegion,
        customArgs: args.length > 0 ? args : null
      });
      
      console.log(`Launched League of Legends for region: ${selectedRegion}`);
      
    } catch (err) {
      error = `Failed to launch League: ${err}`;
      console.error('Error launching League:', err);
    } finally {
      isLoading = false;
    }
  }

  function getRegionDisplayName(region: LeagueRegion): string {
    const displayNames: Record<string, string> = {
      'NA': 'North America',
      'EUW': 'Europe West',
      'EUNE': 'Europe Nordic & East',
      'KR': 'Korea',
      'BR': 'Brazil',
      'LAN': 'Latin America North',
      'LAS': 'Latin America South',
      'OCE': 'Oceania',
      'RU': 'Russia',
      'TR': 'Turkey',
      'JP': 'Japan',
      'PBE': 'Public Beta Environment'
    };
    
    return displayNames[region.name] || region.name;
  }
</script>

<div class="league-launcher">
  <div class="header">
    <h2>League of Legends Launcher</h2>
    <p>Launch League of Legends with your preferred region and settings</p>
  </div>

  {#if error}
    <div class="error-message">
      <p>{error}</p>
    </div>
  {/if}

  <div class="content">
    <!-- Region Selection -->
    <div class="form-group">
      <label for="region-select">Region:</label>
      <select id="region-select" bind:value={selectedRegion} disabled={isLoading}>
        {#each regions as region}
          <option value={region.name}>
            {getRegionDisplayName(region)} ({region.name})
          </option>
        {/each}
      </select>
    </div>

    <!-- Custom Arguments -->
    <div class="form-group">
      <label for="custom-args">Custom Arguments (comma-separated):</label>
      <input 
        id="custom-args" 
        type="text" 
        bind:value={customArgs} 
        placeholder="e.g., allow-multiple-clients, disable-self-update"
        disabled={isLoading}
      />
      <small>Optional: Add custom launch arguments</small>
    </div>

    <!-- Installations Info -->
    {#if installations.length > 0}
      <div class="installations-info">
        <h3>Detected Installations:</h3>
        <ul>
          {#each installations as installation}
            <li>
              <strong>{installation.path}</strong>
              {#if installation.is_pbe}
                <span class="badge pbe">PBE</span>
              {/if}
              {#if installation.is_esports}
                <span class="badge esports">Esports</span>
              {/if}
            </li>
          {/each}
        </ul>
      </div>
    {/if}

    <!-- Launch Button -->
    <div class="actions">
      <button 
        class="launch-button" 
        on:click={launchLeague} 
        disabled={isLoading || !selectedRegion}
      >
        {#if isLoading}
          Launching...
        {:else}
          Launch League of Legends
        {/if}
      </button>
      
      <button class="refresh-button" on:click={loadData} disabled={isLoading}>
        Refresh
      </button>
    </div>
  </div>
</div>

<style>
  .league-launcher {
    max-width: 100%;
    margin: 0;
    padding: 0;
    font-family: inherit;
  }

  .header {
    text-align: left;
    margin-bottom: 12px;
  }

  .header h2 {
    margin: 0 0 4px 0;
    color: inherit;
    font-size: 1rem;
  }

  .header p {
    display: none;
  }

  .error-message {
    background-color: #e74c3c;
    color: white;
    padding: 12px;
    border-radius: 6px;
    margin-bottom: 20px;
  }

  .content {
    background: transparent;
    border-radius: 0;
    padding: 0;
    box-shadow: none;
  }

  .form-group {
    margin-bottom: 12px;
  }

  .form-group label {
    display: block;
    margin-bottom: 8px;
    font-weight: 600;
    color: #2c3e50;
  }

  .form-group select,
  .form-group input {
    width: 100%;
    padding: 12px;
    border: 2px solid #ecf0f1;
    border-radius: 6px;
    font-size: 16px;
    transition: border-color 0.2s;
  }

  .form-group select:focus,
  .form-group input:focus {
    outline: none;
    border-color: #3498db;
  }

  .form-group small {
    display: block;
    margin-top: 4px;
    color: #7f8c8d;
    font-size: 14px;
  }

  .installations-info {
    margin: 20px 0;
    padding: 16px;
    background-color: #f8f9fa;
    border-radius: 6px;
    border-left: 4px solid #3498db;
  }

  .installations-info h3 {
    margin: 0 0 12px 0;
    color: #2c3e50;
    font-size: 18px;
  }

  .installations-info ul {
    margin: 0;
    padding-left: 20px;
  }

  .installations-info li {
    margin-bottom: 8px;
    color: #34495e;
  }

  .badge {
    display: inline-block;
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 600;
    margin-left: 8px;
  }

  .badge.pbe {
    background-color: #e67e22;
    color: white;
  }

  .badge.esports {
    background-color: #9b59b6;
    color: white;
  }

  .actions {
    display: flex;
    gap: 12px;
    margin-top: 24px;
  }

  .launch-button {
    flex: 1;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    border: none;
    padding: 14px 24px;
    border-radius: 6px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .launch-button:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
  }

  .launch-button:disabled {
    background: #bdc3c7;
    cursor: not-allowed;
    transform: none;
    box-shadow: none;
  }

  .refresh-button {
    background: #95a5a6;
    color: white;
    border: none;
    padding: 14px 20px;
    border-radius: 6px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .refresh-button:hover:not(:disabled) {
    background: #7f8c8d;
  }

  .refresh-button:disabled {
    background: #bdc3c7;
    cursor: not-allowed;
  }
</style>
