<script lang="ts">
  import { onMount } from 'svelte';
  import { tauriApi } from '$lib/api/tauri';
  import { clients, isLoading } from '$lib/stores/clients';
  import type { Client } from '$lib/types';

  let showForm = false;
  let formData = {
    name: '',
    email: '',
    phone: ''
  };

  onMount(async () => {
    await loadClients();
  });

  async function loadClients() {
    isLoading.set(true);
    try {
      const data = await tauriApi.listClients();
      clients.set(data);
    } catch (error) {
      console.error('Failed to load clients:', error);
      alert('Failed to load clients');
    } finally {
      isLoading.set(false);
    }
  }

  async function handleSubmit() {
    try {
      await tauriApi.createClient(
        formData.name,
        formData.email,
        formData.phone || undefined
      );
      
      // Reset form
      formData = { name: '', email: '', phone: '' };
      showForm = false;
      
      // Reload clients
      await loadClients();
    } catch (error) {
      console.error('Failed to create client:', error);
      alert('Failed to create client');
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Are you sure you want to delete this client?')) return;
    
    try {
      await tauriApi.deleteClient(id);
      await loadClients();
    } catch (error) {
      console.error('Failed to delete client:', error);
      alert('Failed to delete client');
    }
  }
</script>

<div class="page">
  <div class="header">
    <h1>Clients</h1>
    <button class="btn-primary" on:click={() => showForm = !showForm}>
      {showForm ? 'Cancel' : '+ New Client'}
    </button>
  </div>

  {#if showForm}
    <div class="card form-card">
      <h2>New Client</h2>
      <form on:submit|preventDefault={handleSubmit}>
        <div class="form-group">
          <label for="name">Name *</label>
          <input
            id="name"
            type="text"
            bind:value={formData.name}
            required
            placeholder="John Doe"
          />
        </div>

        <div class="form-group">
          <label for="email">Email *</label>
          <input
            id="email"
            type="email"
            bind:value={formData.email}
            required
            placeholder="john@example.com"
          />
        </div>

        <div class="form-group">
          <label for="phone">Phone</label>
          <input
            id="phone"
            type="tel"
            bind:value={formData.phone}
            placeholder="555-0100"
          />
        </div>

        <div class="form-actions">
          <button type="submit" class="btn-primary">Create Client</button>
        </div>
      </form>
    </div>
  {/if}

  {#if $isLoading}
    <div class="loading">Loading clients...</div>
  {:else if $clients.length === 0}
    <div class="empty">
      <p>No clients yet. Create your first client to get started!</p>
    </div>
  {:else}
    <div class="cards-grid">
      {#each $clients as client}
        <div class="card client-card">
          <div class="card-header">
            <h3>{client.name}</h3>
            {#if client.sync_status === 'pending'}
              <span class="badge badge-warning">Pending Sync</span>
            {:else}
              <span class="badge badge-success">Synced</span>
            {/if}
          </div>
          
          <div class="card-body">
            <p><strong>Email:</strong> {client.email}</p>
            {#if client.phone}
              <p><strong>Phone:</strong> {client.phone}</p>
            {/if}
            <p class="text-muted">
              Created: {new Date(client.created_at).toLocaleDateString()}
            </p>
          </div>

          <div class="card-actions">
            <button class="btn-secondary" on:click={() => alert('Edit coming in Phase 5')}>
              Edit
            </button>
            <button class="btn-danger" on:click={() => handleDelete(client.id)}>
              Delete
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .page {
    max-width: 1400px;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
  }

  .header h1 {
    margin: 0;
    font-size: 2rem;
    font-weight: 600;
  }

  .btn-primary {
    background: #3b82f6;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 0.5rem;
    font-size: 1rem;
    cursor: pointer;
    transition: background 0.2s;
  }

  .btn-primary:hover {
    background: #2563eb;
  }

  .btn-secondary {
    background: #6b7280;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    cursor: pointer;
    transition: background 0.2s;
  }

  .btn-secondary:hover {
    background: #4b5563;
  }

  .btn-danger {
    background: #ef4444;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    cursor: pointer;
    transition: background 0.2s;
  }

  .btn-danger:hover {
    background: #dc2626;
  }

  .card {
    background: white;
    border-radius: 0.5rem;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    padding: 1.5rem;
  }

  .form-card {
    margin-bottom: 2rem;
  }

  .form-card h2 {
    margin: 0 0 1.5rem 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .form-group {
    margin-bottom: 1rem;
  }

  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: #374151;
  }

  .form-group input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 1rem;
    box-sizing: border-box;
  }

  .form-group input:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }

  .form-actions {
    margin-top: 1.5rem;
  }

  .cards-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: 1.5rem;
  }

  .client-card .card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1rem;
  }

  .client-card h3 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .badge {
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .badge-success {
    background: #d1fae5;
    color: #065f46;
  }

  .badge-warning {
    background: #fef3c7;
    color: #92400e;
  }

  .card-body p {
    margin: 0.5rem 0;
    color: #374151;
  }

  .text-muted {
    color: #6b7280 !important;
    font-size: 0.875rem;
  }

  .card-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid #e5e7eb;
  }

  .loading,
  .empty {
    text-align: center;
    padding: 3rem;
    color: #6b7280;
  }

  .empty p {
    font-size: 1.125rem;
  }
</style>
