<script lang="ts">
  import { onMount } from 'svelte';

  let buildings: any[] = [];
  let isLoading = true;

  onMount(() => {
    // Placeholder - will implement in Phase 5
    isLoading = false;
  });
</script>

<div class="page">
  <div class="header">
    <h1>Buildings</h1>
    <button class="btn-primary">+ New Building</button>
  </div>

  <div class="empty">
    <p>Buildings functionality coming in Phase 5 - Sync Implementation</p>
    <p class="text-muted">This page will show all buildings across properties</p>
  </div>
</div>

<style>
  .page { max-width: 1400px; }
  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
  }
  .header h1 { margin: 0; font-size: 2rem; font-weight: 600; }
  .btn-primary {
    background: #3b82f6;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 0.5rem;
    cursor: pointer;
  }
  .empty {
    background: white;
    border-radius: 0.5rem;
    padding: 3rem;
    text-align: center;
  }
  .text-muted { color: #6b7280; font-size: 0.875rem; margin-top: 0.5rem; }
</style>
