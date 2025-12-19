<script lang="ts">
  import { tauriApi } from '$lib/api/tauri';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  let isSyncing = false;
  let isOnline = false;
  let lastSyncTime: Date | null = null;

  async function checkConnection() {
    isOnline = await tauriApi.checkConnection();
  }

  async function handleSync() {
    if (isSyncing) return;

    isSyncing = true;
    try {
      const result = await tauriApi.syncPull();
      lastSyncTime = new Date();
      dispatch('synced');
      alert(result);
    } catch (error) {
      console.error('Sync failed:', error);
      alert('Sync failed: ' + error);
    } finally {
      isSyncing = false;
    }
  }

  // Check connection on mount
  checkConnection();
  
  // Check connection every 30 seconds
  setInterval(checkConnection, 30000);
</script>

<div class="sync-controls">
  <div class="connection-status">
    <div class="status-dot" class:online={isOnline} class:offline={!isOnline}></div>
    <span>{isOnline ? 'Online' : 'Offline'}</span>
  </div>

  <button 
    class="sync-button"
    on:click={handleSync}
    disabled={isSyncing || !isOnline}
  >
    {#if isSyncing}
      <span class="spinner"></span>
      Syncing...
    {:else}
      🔄 Sync Now
    {/if}
  </button>

  {#if lastSyncTime}
    <div class="last-sync">
      Last synced: {lastSyncTime.toLocaleTimeString()}
    </div>
  {/if}
</div>

<style>
  .sync-controls {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem;
  }

  .connection-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
    color: #6b7280;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .status-dot.online {
    background: #10b981;
    box-shadow: 0 0 8px #10b981;
  }

  .status-dot.offline {
    background: #ef4444;
  }

  .sync-button {
    background: #3b82f6;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    cursor: pointer;
    font-size: 0.875rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    transition: background 0.2s;
  }

  .sync-button:hover:not(:disabled) {
    background: #2563eb;
  }

  .sync-button:disabled {
    background: #9ca3af;
    cursor: not-allowed;
  }

  .spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .last-sync {
    font-size: 0.75rem;
    color: #9ca3af;
  }
</style>
