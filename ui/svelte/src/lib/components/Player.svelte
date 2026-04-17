<script lang="ts">
  import type { MediaItem } from '$lib/types';
  import { onMount, tick } from 'svelte';
  import {
    openPlayer, playNext, playPrev,
    cancelNext, pausePlayer, attachKeyboard,
    getNextItem, getPrevItem, hasPrev, hasNext,
  } from '$lib/player';

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
  let prevItem = $state<MediaItem | null>(null);
  let nextItem = $state<MediaItem | null>(null);
  let canPrev = $state(false);
  let canNext = $state(false);

  async function refreshNav() {
    // tick() lets Svelte flush any pending state before we read queue state
    await tick();
    prevItem = getPrevItem();
    nextItem = getNextItem();
    canPrev = hasPrev();
    canNext = hasNext();
  }

  function handleClose() {
    pausePlayer();
    cancelNext();
    upNextItem = null;
    onClose();
  }

  async function handleUpNext(next: MediaItem) {
    upNextItem = next;
    await refreshNav();
  }

  function handlePlayNext() {
    upNextItem = null;
    playNext();
  }

  function handleCancelNext() {
    cancelNext();
    upNextItem = null;
  }

  async function handlePrev() {
    upNextItem = null;
    playPrev();
    await refreshNav();
  }

  async function handleNext() {
    upNextItem = null;
    playNext();
    await refreshNav();
  }

  $effect(() => {
    if (item && allItems.length > 0) {
      upNextItem = null;
      openPlayer(item, allItems, {
        onUpNext: handleUpNext,
        onQueueChange: refreshNav,
      });
      // Use tick() so queue is fully built before reading nav state
      tick().then(() => refreshNav());
    }
  });

  onMount(() => {
    attachKeyboard(handleClose);
  });
</script>

<div id="player-modal" class:open={item !== null}>
  <div class="top-bar">
    <div class="nav-prev">
      {#if canPrev && prevItem}
        <button class="nav-btn" onclick={handlePrev} title="Previous: {prevItem.title}">
          ⏮ <span class="nav-label">{prevItem.title}</span>
        </button>
      {/if}
    </div>

    <p class="title">{item?.title ?? ''}</p>

    <div class="nav-next">
      {#if canNext && nextItem}
        <button class="nav-btn" onclick={handleNext} title="Next: {nextItem.title}">
          <span class="nav-label">{nextItem.title}</span> ⏭
        </button>
      {/if}
    </div>

    <button class="close-btn" onclick={handleClose}>✕</button>
  </div>

  <div class="player-wrap">
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
    background: rgba(0,0,0,0.95); z-index: 100;
    flex-direction: column; gap: 0;
  }
  #player-modal.open { display: flex; }

  .top-bar {
    display: flex; align-items: center;
    padding: 0.75rem 1rem; gap: 0.75rem;
    background: rgba(0,0,0,0.6);
    min-height: 3rem;
  }
  .title {
    flex: 1; color: #fff; font-size: 0.95rem;
    font-weight: 600; text-align: center;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .nav-prev, .nav-next {
    flex: 0 0 auto; max-width: 220px;
    overflow: hidden;
  }
  .nav-btn {
    display: flex; align-items: center; gap: 0.4rem;
    background: rgba(255,255,255,0.08); border: 1px solid rgba(255,255,255,0.12);
    border-radius: 6px; color: #ccc; font-size: 0.78rem;
    padding: 0.3rem 0.6rem; cursor: pointer; white-space: nowrap;
    max-width: 220px; overflow: hidden;
  }
  .nav-btn:hover { background: rgba(255,255,255,0.15); color: #fff; }
  .nav-label {
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
    max-width: 160px; display: inline-block;
  }
  .close-btn {
    flex: 0 0 auto;
    background: none; border: none; color: #888;
    font-size: 1.4rem; cursor: pointer; line-height: 1;
    padding: 0.2rem 0.4rem;
  }
  .close-btn:hover { color: #fff; }

  .player-wrap {
    flex: 1; display: flex; align-items: center; justify-content: center;
    position: relative; padding: 1rem;
  }
  .player-wrap :global(.plyr) { width: min(90vw, 1100px); }

  .up-next {
    position: absolute; bottom: 2rem; right: 1.5rem;
    background: rgba(0,0,0,0.88); border: 1px solid #333;
    border-radius: 8px; padding: 0.75rem 1rem; max-width: 260px;
  }
  .label { font-size: 0.7rem; color: #888; text-transform: uppercase; letter-spacing: 0.08em; margin-bottom: 0.3rem; }
  .next-title { font-size: 0.85rem; color: #e0e0e0; font-weight: 600; margin-bottom: 0.5rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .next-actions { display: flex; gap: 0.4rem; }
  .btn-play-next { padding: 0.3rem 0.7rem; border-radius: 4px; border: none; font-size: 0.8rem; cursor: pointer; background: #f97316; color: #fff; }
  .btn-cancel-next { padding: 0.3rem 0.7rem; border-radius: 4px; border: none; font-size: 0.8rem; cursor: pointer; background: #222; color: #aaa; }
</style>
