<script lang="ts">
  import type { MediaItem } from '$lib/types';
  import { onMount } from 'svelte';
  import { openPlayer, playNext, cancelNext, pausePlayer, attachKeyboard, getNextItem } from '$lib/player';

  let {
    item,
    allItems,
    onClose,
  }: {
    item: MediaItem | null;
    allItems: MediaItem[];
    onClose: () => void;
  } = $props();

  let upNextItem = $state<MediaItem | null>(null);

  function handleClose() {
    pausePlayer();
    cancelNext();
    upNextItem = null;
    onClose();
  }

  function handleUpNext(next: MediaItem) {
    upNextItem = next;
  }

  function handlePlayNext() {
    upNextItem = null;
    playNext(handleUpNext);
  }

  function handleCancelNext() {
    cancelNext();
    upNextItem = null;
  }

  $effect(() => {
    if (item) {
      upNextItem = null;
      openPlayer(item, allItems, handleUpNext, handleClose);
    }
  });

  onMount(() => {
    attachKeyboard(handleClose, handleUpNext);
  });
</script>

<div id="player-modal" class:open={item !== null}>
  <button class="close-btn" onclick={handleClose}>✕</button>
  <p class="title">{item?.title ?? ''}</p>
  <div class="player-wrap">
    <!-- Plyr targets this element -->
    <video id="player" playsinline controls></video>

    {#if upNextItem}
      <div class="up-next">
        <p class="label">Up Next</p>
        <p class="next-title">{upNextItem.title}</p>
        <div class="next-actions">
          <button class="btn-play-next" onclick={handlePlayNext}>▶ Play Now</button>
          <button class="btn-cancel-next" onclick={handleCancelNext}>✕</button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  #player-modal {
    display: none; position: fixed; inset: 0;
    background: rgba(0,0,0,0.92); z-index: 100;
    align-items: center; justify-content: center;
    flex-direction: column; gap: 1rem;
  }
  #player-modal.open { display: flex; }
  .close-btn {
    position: fixed; top: 1.2rem; right: 1.4rem;
    background: none; border: none; color: #888;
    font-size: 1.6rem; cursor: pointer; line-height: 1;
  }
  .close-btn:hover { color: #fff; }
  .title { color: #fff; font-size: 1rem; font-weight: 600; }
  .player-wrap { width: min(90vw, 1100px); position: relative; }
  .player-wrap :global(video) { width: 100%; border-radius: 8px; }

  .up-next {
    position: absolute; bottom: 4rem; right: 1rem;
    background: rgba(0,0,0,0.88); border: 1px solid #333;
    border-radius: 8px; padding: 0.75rem 1rem; max-width: 260px;
  }
  .label { font-size: 0.7rem; color: #888; text-transform: uppercase; letter-spacing: 0.08em; margin-bottom: 0.3rem; }
  .next-title { font-size: 0.85rem; color: #e0e0e0; font-weight: 600; margin-bottom: 0.5rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .next-actions { display: flex; gap: 0.4rem; }
  .btn-play-next { padding: 0.3rem 0.7rem; border-radius: 4px; border: none; font-size: 0.8rem; cursor: pointer; background: #f97316; color: #fff; }
  .btn-cancel-next { padding: 0.3rem 0.7rem; border-radius: 4px; border: none; font-size: 0.8rem; cursor: pointer; background: #222; color: #aaa; }
</style>
