<script lang="ts">
  import type { MediaItem } from '$lib/types';
  import { fmtDur, fmtSize, copyToClipboard } from '$lib/utils';

  let { item, onPlay }: { item: MediaItem; onPlay: (item: MediaItem) => void } = $props();

  let copied = $state(false);

  const meta = $derived(
    [item.year, fmtDur(item.duration_secs), item.resolution, fmtSize(item.size_bytes)]
      .filter(Boolean).join(' · ')
  );

  async function handleCopy() {
    await copyToClipboard(item.file_url);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }
</script>

<div class="card">
  <h2>{item.title}</h2>
  <p class="meta">{meta}</p>
  <div class="actions">
    <button class="btn-play" onclick={() => onPlay(item)}>▶ Play</button>
    <button class="btn-copy" class:copied onclick={handleCopy}>
      {copied ? '✓' : '⎘'}
    </button>
  </div>
</div>

<style>
  .card {
    background: #1a1a1a;
    border: 1px solid #2a2a2a;
    border-radius: 8px;
    padding: 1rem;
  }
  h2 { font-size: 1rem; font-weight: 600; margin-bottom: 0.3rem; color: #e0e0e0; }
  .meta { font-size: 0.78rem; color: #666; margin-bottom: 0.8rem; }
  .actions { display: flex; gap: 0.5rem; }
  .btn-play {
    flex: 1; padding: 0.4rem 0.6rem; border-radius: 5px;
    background: #f97316; color: #fff; border: none;
    font-size: 0.82rem; cursor: pointer;
  }
  .btn-play:hover { background: #ea6a0a; }
  .btn-copy {
    padding: 0.4rem 0.7rem; border-radius: 5px;
    background: #222; color: #aaa; border: none;
    font-size: 0.82rem; cursor: pointer;
  }
  .btn-copy:hover { background: #2a2a2a; color: #fff; }
  .btn-copy.copied { background: #14532d; color: #86efac; }
</style>
