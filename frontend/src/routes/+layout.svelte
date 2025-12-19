<script lang="ts">
  import { page } from '$app/stores';
  import SyncButton from '$lib/components/SyncButton.svelte';
  
  const navItems = [
    { href: '/', label: 'Dashboard' },
    { href: '/clients', label: 'Clients' },
    { href: '/properties', label: 'Properties' },
    { href: '/buildings', label: 'Buildings' },
    { href: '/units', label: 'Units' },
  ];

  function handleSynced() {
    // Reload current page data
    window.location.reload();
  }
</script>

<div class="app">
  <nav class="sidebar">
    <div class="logo">
      <h1>Spectrum</h1>
      <p>Tenant POC</p>
    </div>
    
    <ul class="nav-items">
      {#each navItems as item}
        <li>
          <a 
            href={item.href}
            class:active={$page.url.pathname === item.href}
          >
            {item.label}
          </a>
        </li>
      {/each}
    </ul>
    
    <div class="sidebar-footer">
      <SyncButton on:synced={handleSynced} />
    </div>
  </nav>
  
  <main class="content">
    <slot />
  </main>
</div>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
      Ubuntu, Cantarell, sans-serif;
    background: #f5f5f5;
  }

  .app {
    display: flex;
    height: 100vh;
  }

  .sidebar {
    width: 250px;
    background: #1a1a1a;
    color: white;
    display: flex;
    flex-direction: column;
  }

  .logo {
    padding: 2rem 1.5rem;
    border-top: 1px solid #333;
  }

  .logo h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
  }

  .logo p {
    margin: 0.25rem 0 0 0;
    font-size: 0.875rem;
    color: #888;
  }

  .nav-items {
    list-style: none;
    padding: 1rem 0;
    margin: 0;
    flex: 1;
  }

  .nav-items li a {
    display: block;
    padding: 0.75rem 1.5rem;
    color: #ccc;
    text-decoration: none;
    transition: all 0.2s;
  }

  .nav-items li a:hover {
    background: #2a2a2a;
    color: white;
  }

  .nav-items li a.active {
    background: #3b82f6;
    color: white;
  }

  .sidebar-footer {
    padding: 1rem;
    border-top: 1px solid #333;
  }

  .content {
    flex: 1;
    overflow: auto;
    padding: 2rem;
  }
</style>
