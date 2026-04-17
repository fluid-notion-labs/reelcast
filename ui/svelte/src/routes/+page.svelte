<script lang="ts">
  import { onMount } from 'svelte';
  import type { MediaItem, RecentItem } from '$lib/types';
  import { saveSearch, getRecentSearches, naturalSort } from '$lib/utils';
  import MediaCard from '$lib/components/MediaCard.svelte';
  import RecentShelf from '$lib/components/RecentShelf.svelte';
  import Player from '$lib/components/Player.svelte';

  let query = $state('');
  let results = $state<MediaItem[]>([]);
  let recentPlayed = $state<RecentItem[]>([]);
  let recentSearches = $state<string[]>([]);
  let status = $state('');
  let activeItem = $state<MediaItem | null>(null);
  let showHttpsBanner = $state(false);

  function focusOnMount(node: HTMLElement) {
    node.focus();
  }

  async function doSearch() {
    const url = query.trim()
      ? `/search?q=${encodeURIComponent(query.trim())}`
      : '/search';
    status = 'Searching…';
    const res = await fetch(url);
    results = await res.json();
    if (query.trim()) {
      saveSearch(query.trim());
      recentSearches = getRecentSearches();
    }
    status = `${results.length} result${results.length !== 1 ? 's' : ''}`;
  }

  async function loadRecent() {
    const res = await fetch('/recent');
    recentPlayed = await res.json();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') doSearch();
  }

  function playItem(item: MediaItem) {
    activeItem = item;
    document.body.style.overflow = 'hidden';
  }

  function playRecentItem(r: RecentItem) {
    // Find full MediaItem from results, or synthesise one from recent data
    const found = results.find(m => m.id === r.media_id);
    if (found) { playItem(found); return; }
    // Minimal stand-in so player can open
    playItem({
      id: r.media_id,
      title: r.title,
      file_url: r.file_url,
      file_url_https: r.file_url_https,
      play_url: r.play_url,
      playlist_url: '',
      dir: '',
      filename: '',
      year: null,
      duration_secs: null,
      size_bytes: 0,
      container: null,
      resolution: null,
    });
  }

  function closePlayer() {
    activeItem = null;
    document.body.style.overflow = '';
  }

  onMount(() => {
    recentSearches = getRecentSearches();
    doSearch();
    loadRecent();
    showHttpsBanner =
      location.protocol === 'http:' &&
      location.hostname !== 'localhost' &&
      location.hostname !== '127.0.0.1';
  });
</script>

{#if showHttpsBanner}
  <div class="banner">
    ⚠️ Running on HTTP — Copy URL won't work.
    <a href="/setup">Set up HTTPS →</a>
  </div>
{/if}

<header>
  <h1>Reel<span>cast</span></h1>
</header>

<div class="search-bar">
  <input
    bind:value={query}
    onkeydown={handleKeydown}
    placeholder="Search movies…"
    use:focusOnMount
  />
  <button onclick={doSearch}>Search</button>
</div>

{#if recentSearches.length}
  <div class="chips">
    {#each recentSearches as s}
      <button class="chip" onclick={() => { query = s; doSearch(); }}>⌕ {s}</button>
    {/each}
  </div>
{/if}

<RecentShelf items={recentPlayed} onPlay={playRecentItem} />

<p class="status">{status}</p>

{#if results.length}
  <p class="section-title">Results</p>
  <div class="grid">
    {#each results as item (item.id)}
      <MediaCard {item} onPlay={playItem} />
    {/each}
  </div>
{/if}

<Player item={activeItem} allItems={results} onClose={closePlayer} />

<style>
  :global(*, *::before, *::after) { box-sizing: border-box; margin: 0; padding: 0; }
  :global(body) {
    font-family: system-ui, sans-serif;
    background: #0f0f0f; color: #e0e0e0;
    min-height: 100vh; padding: 2rem;
  }

  .banner {
    background: #1c1408; border: 1px solid #78350f;
    border-radius: 6px; padding: 0.6rem 1rem;
    margin-bottom: 1rem; font-size: 0.85rem; color: #fbbf24;
  }
  .banner a { color: #f97316; font-weight: 600; }

  h1 { font-size: 1.6rem; font-weight: 600; color: #fff; margin-bottom: 1.5rem; letter-spacing: -0.02em; }
  h1 span { color: #f97316; }

  .search-bar { display: flex; gap: 0.5rem; margin-bottom: 0.75rem; }
  .search-bar input {
    flex: 1; padding: 0.6rem 1rem; border-radius: 6px;
    border: 1px solid #333; background: #1a1a1a;
    color: #e0e0e0; font-size: 1rem; outline: none;
  }
  .search-bar input:focus { border-color: #f97316; }
  .search-bar button {
    padding: 0.6rem 1.2rem; border-radius: 6px; border: none;
    background: #f97316; color: #fff; font-size: 1rem; cursor: pointer;
  }
  .search-bar button:hover { background: #ea6a0a; }

  .chips { display: flex; flex-wrap: wrap; gap: 0.4rem; margin-bottom: 1.5rem; }
  .chip {
    padding: 0.25rem 0.75rem; border-radius: 999px;
    background: #222; border: 1px solid #333;
    color: #aaa; font-size: 0.8rem; cursor: pointer;
  }
  .chip:hover { background: #2a2a2a; color: #fff; border-color: #f97316; }

  .status { color: #555; font-size: 0.85rem; margin-bottom: 0.75rem; }
  .section-title {
    font-size: 0.75rem; font-weight: 600; text-transform: uppercase;
    letter-spacing: 0.1em; color: #555; margin-bottom: 0.75rem;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 1rem;
  }
</style>
