import type { MediaItem } from './types';
import { naturalSort } from './utils';

// Plyr is loaded via CDN in app.html
declare const Plyr: any;

let plyr: any = null;
let queue: MediaItem[] = [];
let queueIdx = -1;
let nextTimer: ReturnType<typeof setTimeout> | null = null;

export function initPlyr() {
  if (plyr) return;
  plyr = new Plyr('#player', {
    controls: ['play-large','play','progress','current-time','duration','mute','volume','fullscreen'],
    keyboard: { focused: false, global: false },
  });
  plyr.on('ended', showUpNext);
}

export function getPlyr() { return plyr; }

export function buildQueue(item: MediaItem, items: MediaItem[]) {
  queue = [...items]
    .filter(m => m.dir === item.dir)
    .sort((a, b) => naturalSort(a.filename, b.filename));
  queueIdx = queue.findIndex(m => m.id === item.id);
}

export function openPlayer(item: MediaItem, allItems: MediaItem[], onUpNext: (next: MediaItem) => void, onClose?: () => void) {
  initPlyr();
  buildQueue(item, allItems);
  plyr.source = { type: 'video', sources: [{ src: item.file_url_https }] };
  plyr.play();

  plyr.off('ended');
  plyr.on('ended', () => showUpNext(onUpNext));
}

export function playNext(onUpNext: (next: MediaItem) => void) {
  cancelNext();
  if (queueIdx < queue.length - 1) {
    queueIdx++;
    const next = queue[queueIdx];
    plyr.source = { type: 'video', sources: [{ src: next.file_url_https }] };
    plyr.play();
    plyr.off('ended');
    plyr.on('ended', () => showUpNext(onUpNext));
  }
}

export function getNextItem(): MediaItem | null {
  return queueIdx >= 0 && queueIdx < queue.length - 1 ? queue[queueIdx + 1] : null;
}

function showUpNext(onUpNext: (next: MediaItem) => void) {
  cancelNext();
  const next = getNextItem();
  if (!next) return;
  onUpNext(next);
  nextTimer = setTimeout(() => playNext(onUpNext), 8000);
}

export function cancelNext() {
  if (nextTimer) { clearTimeout(nextTimer); nextTimer = null; }
}

export function pausePlayer() {
  plyr?.pause();
}

// VLC-style keyboard handler — attach once globally
export function attachKeyboard(onClose: () => void, onUpNext: (next: MediaItem) => void) {
  window.addEventListener('keydown', (e) => {
    if (!plyr) return;
    const modal = document.getElementById('player-modal');
    if (!modal?.classList.contains('open')) return;
    if ((e.target as HTMLElement).tagName === 'INPUT') return;

    const cur = plyr.currentTime;
    switch (true) {
      case e.key === ' ':               e.preventDefault(); plyr.togglePlay(); break;
      case e.key === 'f' || e.key==='F': plyr.fullscreen.toggle(); break;
      case e.key === 'm' || e.key==='M': plyr.muted = !plyr.muted; break;
      case e.shiftKey && e.key==='ArrowLeft':  e.preventDefault(); plyr.currentTime = Math.max(0,cur-3); break;
      case e.shiftKey && e.key==='ArrowRight': e.preventDefault(); plyr.currentTime = cur+3; break;
      case e.ctrlKey  && e.key==='ArrowLeft':  e.preventDefault(); plyr.currentTime = Math.max(0,cur-60); break;
      case e.ctrlKey  && e.key==='ArrowRight': e.preventDefault(); plyr.currentTime = cur+60; break;
      case !e.shiftKey&&!e.altKey&&!e.ctrlKey&&!e.metaKey&&e.key==='ArrowLeft':  e.preventDefault(); plyr.currentTime = Math.max(0,cur-10); break;
      case !e.shiftKey&&!e.altKey&&!e.ctrlKey&&!e.metaKey&&e.key==='ArrowRight': e.preventDefault(); plyr.currentTime = cur+10; break;
      case e.key==='ArrowUp':   e.preventDefault(); plyr.volume = Math.min(1,plyr.volume+0.1); break;
      case e.key==='ArrowDown': e.preventDefault(); plyr.volume = Math.max(0,plyr.volume-0.1); break;
      case e.key==='Escape': onClose(); break;
    }
  });
}
