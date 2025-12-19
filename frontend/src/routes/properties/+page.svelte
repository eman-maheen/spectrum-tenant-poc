<script lang="ts">
  import { onMount } from 'svelte';
  import { tauriApi } from '$lib/api/tauri';
  import { properties } from '$lib/stores/properties';
  import { clients } from '$lib/stores/clients';
  import type { Property } from '$lib/types';

  let showForm = false;
  let isLoading = false;
  let formData = {
    clientId: '',
    name: '',
    address: '',
    city: '',
    state: '',
    zipCode: ''
  };

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    isLoading = true;
    try {
      const [propertiesData, clientsData] = await Promise.all([
        tauriApi.listProperties(),
        tauriApi.listClients()
      ]);
      properties.set(propertiesData);
      clients.set(clientsData);
    } catch (error) {
      console.error('Failed to load data:', error);
      alert('Failed to load data');
    } finally {
      isLoading = false;
    }
  }

  async function handleSubmit() {
    if (!formData.clientId) {
      alert('Please select a client');
      return;
    }

    try {
      await tauriApi.createProperty(
        formData.clientId,
        formData.name,
        formData.address,
        formData.city || undefined,
        formData.state || undefined,
        formData.zipCode || undefined
      );
      
      formData = {
        clientId: '',
        name: '',
        address: '',
        city: '',
        state: '',
        zipCode: ''
      };
      showForm = false;
      await loadData();
    } catch (error) {
      console.error('Failed to create property:', error);
      alert('Failed to create property');
    }
  }

  function getClientName(clientId: string): string {
    const client = $clients.find(c => c.id === clientId);
    return client ? client.name : 'Unknown Client';
  }
</script>

<div class="page">
  <div class="header">
    <h1>Properties</h1>
    <button class="btn-primary" on:click={() => showForm = !showForm}>
      {showForm ? 'Cancel' : '+ New Property'}
    </button>
  </div>

  {#if showForm}
    <div class="card form-card">
      <h2>New Property</h2>
      <form on:submit|preventDefault={handleSubmit}>
        <div class="form-group">
          <label for="client">Client *</label>
          <select id="client" bind:value={formData.clientId} required>
            <option value="">Select a client</option>
            {#each $clients as client}
              <option value={client.id}>{client.name}</option>
            {/each}
          </select>
        </div>

        <div class="form-group">
          <label for="name">Property Name *</label>
          <input
            id="name"
            type="text"
            bind:value={formData.name}
            required
            placeholder="Sunset Apartments"
          />
        </div>

        <div class="form-group">
          <label for="address">Address *</label>
          <input
            id="address"
            type="text"
            bind:value={formData.address}
            required
            placeholder="123 Main Street"
          />
        </div>

        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 1rem;">
          <div class="form-group">
            <label for="city">City</label>
            <input
              id="city"
              type="text"
              bind:value={formData.city}
              placeholder="Springfield"
            />
          </div>

          <div class="form-group">
            <label for="state">State</label>
            <input
              id="state"
              type="text"
              bind:value={formData.state}
              placeholder="IL"
            />
          </div>

          <div class="form-group">
            <label for="zipCode">Zip Code</label>
            <input
              id="zipCode"
              type="text"
              bind:value={formData.zipCode}
              placeholder="62701"
            />
          </div>
        </div>

        <div class="form-actions">
          <button type="submit" class="btn-primary">Create Property</button>
        </div>
      </form>
    </div>
  {/if}

  {#if isLoading}
    <div class="loading">Loading properties...</div>
  {:else if $properties.length === 0}
    <div class="empty">
      <p>No properties yet. Create your first property!</p>
    </div>
  {:else}
    <div class="cards-grid">
      {#each $properties as property}
        <div class="card">
          <div class="card-header">
            <h3>{property.name}</h3>
            {#if property.sync_status === 'pending'}
              <span class="badge badge-warning">Pending</span>
            {:else}
              <span class="badge badge-success">Synced</span>
            {/if}
          </div>
          
          <div class="card-body">
            <p><strong>Client:</strong> {getClientName(property.client_id)}</p>
            <p><strong>Address:</strong> {property.address}</p>
            {#if property.city || property.state || property.zip_code}
              <p>
                {property.city || ''}{property.city && property.state ? ', ' : ''}{property.state || ''} {property.zip_code || ''}
              </p>
            {/if}
            <p class="text-muted">
              Created: {new Date(property.created_at).toLocaleDateString()}
            </p>
          </div>

          <div class="card-actions">
            <button class="btn-secondary">View Details</button>
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

  select {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 1rem;
    box-sizing: border-box;
    background: white;
  }

  select:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
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
  }

  .btn-secondary:hover {
    background: #4b5563;
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

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1rem;
  }

  .card h3 {
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
</style>
