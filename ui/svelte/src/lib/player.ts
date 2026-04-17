import type { MediaItem } from './types';
import { naturalSort } from './utils';

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
}

export function getPlyr() { return plyr; }
export function getQueue() { return queue; }
export function getQueueIdx() { return queueIdx; }
export function hasPrev() { return queueIdx > 0; }
export function hasNext() { return queueIdx >= 0 && queueIdx < queue.length - 1; }
export function getCurrentItem(): MediaItem | null { return queue[queueIdx] ?? null; }
export function getNextItem(): MediaItem | null { return hasNext() ? queue[queueIdx + 1] : null; }
export function getPrevItem(): MediaItem | null { return hasPrev() ? queue[queueIdx - 1] : null; }

function dirTokens(dirs: string[]): Set<string> {
  const tokens = new Set<string>();
  const skip = new Set(['the','and','of','in','to','a','an','for','with','from']);
  for (const d of dirs) {
    const parts = d.replace(/\/+$/, '').split('/').slice(-2);
    for (const part of parts) {
      const norm = part.replace(/[._-]/g, ' ').toLowerCase()
        .replace(/(s\d{1,2}|complete|720p|1080p|480p|webrip|dsnp|galaxytv|tgx|season\s*\d+)/g, '');
      for (const tok of norm.split(/\s+/))
        if (tok.length > 3 && !skip.has(tok)) tokens.add(tok);
    }
  }
  return tokens;
}

function keysRelated(keyA: string, dirA: string, keyB: string, dirB: string): boolean {
  if (!keyA.includes(keyB) && !keyB.includes(keyA)) return false;
  const ancA = dirA.split('/').slice(0,-1).join('/');
  const ancB = dirB.split('/').slice(0,-1).join('/');
  if (ancA === ancB) return true;
  const tokA = dirTokens([dirA]), tokB = dirTokens([dirB]);
  for (const t of tokA) if (tokB.has(t)) return true;
  return false;
}

export function buildQueue(item: MediaItem, items: MediaItem[]) {
  const grouped = item.series_key
    ? items.filter(m => m.series_key != null && (
        m.series_key === item.series_key ||
        keysRelated(item.series_key!, item.dir, m.series_key, m.dir)
      ))
    : items.filter(m => m.dir === item.dir);

  queue = [...grouped].sort((a, b) => {
    const epA = extractEpisodeKey(a.filename);
    const epB = extractEpisodeKey(b.filename);
    if (epA && epB) return epA.localeCompare(epB, undefined, { numeric: true, sensitivity: 'base' });
    return naturalSort(a.filename, b.filename);
  });

  queueIdx = queue.findIndex(m => m.id === item.id);
}

function extractEpisodeKey(filename: string): string | null {
  const m = filename.match(/[Ss](\d{1,2})[Ee](\d{1,2})/);
  if (m) return `${m[1].padStart(2,'0')} ${m[2].padStart(2,'0')}`;
  return null;
}

type QueueCallbacks = {
  onUpNext: (next: MediaItem) => void;
  onQueueChange: () => void; // called when idx changes so UI can re-render nav buttons
};

let callbacks: QueueCallbacks | null = null;

function playAtIdx(idx: number) {
  if (idx < 0 || idx >= queue.length) return;
  cancelNext();
  queueIdx = idx;
  const item = queue[queueIdx];
  plyr.source = { type: 'video', sources: [{ src: item.file_url_https }] };
  plyr.play();
  plyr.off('ended');
  plyr.on('ended', () => showUpNext());
  callbacks?.onQueueChange();
}

export function openPlayer(item: MediaItem, allItems: MediaItem[], cb: QueueCallbacks) {
  initPlyr();
  callbacks = cb;
  buildQueue(item, allItems);
  playAtIdx(queueIdx);
}

export function playNext() {
  if (hasNext()) playAtIdx(queueIdx + 1);
}

export function playPrev() {
  // If >3s in, restart current; otherwise go to previous
  if (plyr && plyr.currentTime > 3) {
    plyr.currentTime = 0;
    callbacks?.onQueueChange();
  } else if (hasPrev()) {
    playAtIdx(queueIdx - 1);
  }
}

function showUpNext() {
  cancelNext();
  const next = getNextItem();
  if (!next) return;
  callbacks?.onUpNext(next);
  nextTimer = setTimeout(() => playNext(), 8000);
}

export function cancelNext() {
  if (nextTimer) { clearTimeout(nextTimer); nextTimer = null; }
}

export function pausePlayer() {
  plyr?.pause();
}

export function attachKeyboard(onClose: () => void) {
  window.addEventListener('keydown', (e) => {
    if (!plyr) return;
    const modal = document.getElementById('player-modal');
    if (!modal?.classList.contains('open')) return;
    if ((e.target as HTMLElement).tagName === 'INPUT') return;

    const cur = plyr.currentTime;
    switch (true) {
      case e.key === ' ':                e.preventDefault(); plyr.togglePlay(); break;
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
      // Queue navigation
      case e.key==='n' || e.key==='N':   e.preventDefault(); playNext(); break;
      case e.key==='p' || e.key==='P':   e.preventDefault(); playPrev(); break;
      case e.key==='Escape': onClose(); break;
    }
  });
}
