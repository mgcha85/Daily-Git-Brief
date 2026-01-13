<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
</script>

<div class="app">
  <header>
    <div class="container">
      <nav>
        <a href="/" class="logo">
          <span class="logo-icon">📊</span>
          <span class="logo-text">Daily Git Brief</span>
        </a>
        <div class="nav-links">
          <a href="/" class:active={$page.url.pathname === '/'}>트렌딩</a>
          <a href="/languages" class:active={$page.url.pathname === '/languages'}>언어 트렌드</a>
        </div>
      </nav>
    </div>
  </header>

  <main>
    <slot />
  </main>

  <footer>
    <div class="container">
      <p>© 2026 Daily Git Brief. Data from <a href="https://ossinsight.io" target="_blank">OSS Insight</a></p>
    </div>
  </footer>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  header {
    position: sticky;
    top: 0;
    z-index: 100;
    background: rgba(13, 17, 23, 0.8);
    backdrop-filter: blur(12px);
    border-bottom: 1px solid var(--color-border);
  }

  nav {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4) 0;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-weight: 700;
    font-size: var(--font-size-xl);
    color: var(--color-text-primary);
  }

  .logo:hover {
    color: var(--color-text-primary);
  }

  .logo-icon {
    font-size: 1.5rem;
  }

  .nav-links {
    display: flex;
    gap: var(--space-6);
  }

  .nav-links a {
    color: var(--color-text-secondary);
    font-weight: 500;
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    transition: all var(--transition-fast);
  }

  .nav-links a:hover {
    color: var(--color-text-primary);
    background: var(--color-bg-tertiary);
  }

  .nav-links a.active {
    color: var(--color-accent-blue);
    background: rgba(88, 166, 255, 0.1);
  }

  main {
    flex: 1;
    padding: var(--space-8) 0;
  }

  footer {
    border-top: 1px solid var(--color-border);
    padding: var(--space-6) 0;
    text-align: center;
    color: var(--color-text-muted);
    font-size: var(--font-size-sm);
  }
</style>
