<!-- frontend/src/lib/components/SyncButton.svelte -->

<script lang="ts">
  import { tauriApi } from '$lib/api/tauri';
  import { createEventDispatcher, onMount } from 'svelte';

  const dispatch = createEventDispatcher();

  let isSyncing = false;
  let isOnline = false;
  let lastSyncTime: Date | null = null;
  let pendingCount = 0;
  let syncMessage = '';
  let showMessage = false;

  async function checkConnection() {
    isOnline = await tauriApi.checkConnection();
  }

  async function updatePendingCount() {
    try {
      pendingCount = await tauriApi.getPendingCount();
    } catch (error) {
      console.error('Failed to get pending count:', error);
    }
  }

  async function handleSync() {
    if (isSyncing || !isOnline) return;

    isSyncing = true;
    showMessage = false;
    
    try {
      const result = await tauriApi.syncBidirectional();
      lastSyncTime = new Date();
      pendingCount = 0;
      syncMessage = result;
      showMessage = true;
      
      // Hide message after 5 seconds
      setTimeout(() => {
        showMessage = false;
      }, 5000);
      
      dispatch('synced');
    } catch (error) {
      console.error('Sync failed:', error);
      syncMessage = `❌ Sync failed: ${error}`;
      showMessage = true;
      
      setTimeout(() => {
        showMessage = false;
      }, 5000);
    } finally {
      isSyncing = false;
    }
  }

  onMount(() => {
    // Initial checks
    checkConnection();
    updatePendingCount();
    
    // Check connection every 30 seconds
    const connectionInterval = setInterval(checkConnection, 30000);
    
    // Update pending count every 10 seconds
    const pendingInterval = setInterval(updatePendingCount, 10000);
    
    return () => {
      clearInterval(connectionInterval);
      clearInterval(pendingInterval);
    };
  });
</script>

<div class="sync-controls">
  <div class="connection-status">
    <div class="status-dot" class:online={isOnline} class:offline={!isOnline}></div>
    <span class="status-text">
      {isOnline ? 'Online' : 'Offline'}
    </span>
    
    {#if pendingCount > 0}
      <span class="pending-badge" title="Pending changes to sync">
        {pendingCount}
      </span>
    {/if}
  </div>

  <button 
    class="sync-button"
    class:has-pending={pendingCount > 0}
    on:click={handleSync}
    disabled={isSyncing || !isOnline}
    title={isOnline ? 'Sync with server' : 'Server offline - cannot sync'}
  >
    {#if isSyncing}
      <span class="spinner"></span>
      Syncing...
    {:else}
      🔄 Sync Now
      {#if pendingCount > 0}
        <span class="badge">{pendingCount}</span>
      {/if}
    {/if}
  </button>

  {#if lastSyncTime}
    <div class="last-sync">
      Last: {lastSyncTime.toLocaleTimeString()}
    </div>
  {/if}
</div>

{#if showMessage}
  <div class="sync-message" class:success={!syncMessage.includes('failed')}>
    <pre>{syncMessage}</pre>
  </div>
{/if}

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
    color: #9ca3af;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    transition: all 0.3s ease;
  }

  .status-dot.online {
    background: #10b981;
    box-shadow: 0 0 8px #10b981;
    animation: pulse 2s infinite;
  }

  .status-dot.offline {
    background: #ef4444;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  .status-text {
    color: #d1d5db;
  }

  .pending-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 20px;
    height: 20px;
    padding: 0 6px;
    background: #f59e0b;
    color: white;
    border-radius: 10px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .sync-button {
    position: relative;
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
    transition: all 0.2s;
  }

  .sync-button.has-pending {
    background: #f59e0b;
    animation: attention 2s infinite;
  }

  @keyframes attention {
    0%, 100% {
      transform: scale(1);
    }
    50% {
      transform: scale(1.05);
    }
  }

  .sync-button:hover:not(:disabled) {
    background: #2563eb;
    transform: translateY(-1px);
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  }

  .sync-button.has-pending:hover:not(:disabled) {
    background: #d97706;
  }

  .sync-button:disabled {
    background: #4b5563;
    cursor: not-allowed;
    opacity: 0.6;
  }

  .sync-button .badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    background: rgba(255, 255, 255, 0.3);
    border-radius: 9px;
    font-size: 0.7rem;
    font-weight: 700;
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
    color: #6b7280;
  }

  .sync-message {
    margin-top: 0.5rem;
    padding: 0.75rem;
    border-radius: 0.375rem;
    font-size: 0.75rem;
    background: #1f2937;
    border-left: 3px solid #ef4444;
    animation: slideIn 0.3s ease-out;
  }

  .sync-message.success {
    border-left-color: #10b981;
  }

  .sync-message pre {
    margin: 0;
    color: #d1d5db;
    white-space: pre-wrap;
    font-family: 'Courier New', monospace;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>