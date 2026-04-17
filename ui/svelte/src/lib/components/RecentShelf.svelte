<script lang="ts">
  import type { RecentItem } from '$lib/types';
  import { fmtAgo, copyToClipboard } from '$lib/utils';

  let { items, onPlay }: { items: RecentItem[]; onPlay: (r: RecentItem) => void } = $props();

  let copiedId = $state<string | null>(null);

  async function handleCopy(r: RecentItem) {
    await copyToClipboard(r.file_url);
    copiedId = r.media_id;
    setTimeout(() => (copiedId = null), 2000);
  }
</script>

{#if items.length}
  <section>
    <h3 class="section-title">Recently Played</h3>
    <div class="shelf">
      {#each items as r (r.media_id)}
        <div class="card">
          <p class="title" title={r.title}>{r.title}</p>
          <p class="time">{fmtAgo(r.played_at)}</p>
          <div class="actions">
            <button class="btn-play" onclick={() => onPlay(r)}>▶ Play</button>
            <button
              class="btn-copy"
              class:copied={copiedId === r.media_id}
              onclick={() => handleCopy(r)}
            >{copiedId === r.media_id ? '✓' : '⎘'}</button>
          </div>
        </div>
      {/each}
    </div>
  </section>
{/if}

<style>
  section { margin-bottom: 2rem; }
  .section-title {
    font-size: 0.75rem; font-weight: 600; text-transform: uppercase;
    letter-spacing: 0.1em; color: #555; margin-bottom: 0.75rem;
  }
  .shelf {
    display: flex; gap: 0.75rem;
    overflow-x: auto; padding-bottom: 0.5rem;
  }
  .shelf::-webkit-scrollbar { height: 4px; }
  .shelf::-webkit-scrollbar-thumb { background: #333; border-radius: 2px; }
  .card {
    flex: 0 0 190px; background: #1a1a1a;
    border: 1px solid #2a2a2a; border-radius: 8px; padding: 0.75rem;
  }
  .title {
    font-size: 0.88rem; font-weight: 600; color: #e0e0e0;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    margin-bottom: 0.2rem;
  }
  .time { font-size: 0.72rem; color: #555; margin-bottom: 0.6rem; }
  .actions { display: flex; gap: 0.4rem; }
  .btn-play {
    flex: 1; padding: 0.3rem 0.5rem; border-radius: 5px;
    background: #f97316; color: #fff; border: none;
    font-size: 0.78rem; cursor: pointer;
  }
  .btn-play:hover { background: #ea6a0a; }
  .btn-copy {
    padding: 0.3rem 0.5rem; border-radius: 5px;
    background: #222; color: #aaa; border: none;
    font-size: 0.78rem; cursor: pointer;
  }
  .btn-copy:hover { background: #2a2a2a; color: #fff; }
  .btn-copy.copied { background: #14532d; color: #86efac; }
</style>
