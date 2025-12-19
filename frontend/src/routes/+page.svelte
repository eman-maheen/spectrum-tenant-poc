<script lang="ts">
  import { onMount } from 'svelte';
  import { tauriApi } from '$lib/api/tauri';

  let stats = {
    clients: 0,
    properties: 0,
    pendingSync: 0
  };

  let isLoading = true;

  onMount(async () => {
    await loadStats();
  });

  async function loadStats() {
    try {
      const [clients, properties] = await Promise.all([
        tauriApi.listClients(),
        tauriApi.listProperties()
      ]);

      stats.clients = clients.length;
      stats.properties = properties.length;
      stats.pendingSync = clients.filter(c => c.sync_status === 'pending').length +
                         properties.filter(p => p.sync_status === 'pending').length;
    } catch (error) {
      console.error('Failed to load stats:', error);
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="dashboard">
  <h1>Dashboard</h1>
  
  {#if isLoading}
    <div class="loading">Loading...</div>
  {:else}
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon clients">
          <svg width="24" height="24" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"></path>
          </svg>
        </div>
        <div class="stat-content">
          <h3>Clients</h3>
          <p class="stat-number">{stats.clients}</p>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon properties">
          <svg width="24" height="24" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"></path>
          </svg>
        </div>
        <div class="stat-content">
          <h3>Properties</h3>
          <p class="stat-number">{stats.properties}</p>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon sync">
          <svg width="24" height="24" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
          </svg>
        </div>
        <div class="stat-content">
          <h3>Pending Sync</h3>
          <p class="stat-number">{stats.pendingSync}</p>
        </div>
      </div>
    </div>

    <div class="welcome-card">
      <h2>Welcome to Spectrum Tenant POC</h2>
      <p>This is an offline-first property management system with bidirectional sync.</p>
      
      <div class="features">
        <div class="feature">
          <div class="feature-icon">📱</div>
          <h3>Offline First</h3>
          <p>Work without internet, sync when connected</p>
        </div>
        
        <div class="feature">
          <div class="feature-icon">🔄</div>
          <h3>Auto Sync</h3>
          <p>Changes automatically sync with the server</p>
        </div>
        
        <div class="feature">
          <div class="feature-icon">🏢</div>
          <h3>Multi-Tenant</h3>
          <p>Isolated data for each tenant</p>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .dashboard {
    max-width: 1400px;
  }

  h1 {
    margin: 0 0 2rem 0;
    font-size: 2rem;
    font-weight: 600;
  }

  .loading {
    text-align: center;
    padding: 3rem;
    color: #6b7280;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1.5rem;
    margin-bottom: 2rem;
  }

  .stat-card {
    background: white;
    border-radius: 0.5rem;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    padding: 1.5rem;
    display: flex;
    gap: 1rem;
  }

  .stat-icon {
    width: 48px;
    height: 48px;
    border-radius: 0.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
  }

  .stat-icon.clients {
    background: #3b82f6;
  }

  .stat-icon.properties {
    background: #10b981;
  }

  .stat-icon.sync {
    background: #f59e0b;
  }

  .stat-content h3 {
    margin: 0;
    font-size: 0.875rem;
    color: #6b7280;
    font-weight: 500;
  }

  .stat-number {
    margin: 0.25rem 0 0 0;
    font-size: 2rem;
    font-weight: 700;
    color: #111827;
  }

  .welcome-card {
    background: white;
    border-radius: 0.5rem;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    padding: 2rem;
  }

  .welcome-card h2 {
    margin: 0 0 0.5rem 0;
    font-size: 1.5rem;
    font-weight: 600;
  }

  .welcome-card > p {
    margin: 0 0 2rem 0;
    color: #6b7280;
  }

  .features {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 2rem;
  }

  .feature {
    text-align: center;
  }

  .feature-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
  }

  .feature h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.125rem;
    font-weight: 600;
  }

  .feature p {
    margin: 0;
    color: #6b7280;
    font-size: 0.875rem;
  }
</style>
