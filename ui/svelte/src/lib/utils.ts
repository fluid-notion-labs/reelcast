export function fmtDur(secs: number | null): string {
  if (!secs) return '';
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  return h > 0 ? `${h}h ${m}m` : `${m}m`;
}

export function fmtSize(bytes: number): string {
  return bytes > 1e9
    ? (bytes / 1e9).toFixed(1) + ' GB'
    : (bytes / 1e6).toFixed(0) + ' MB';
}

export function fmtAgo(ts: number): string {
  const diff = Math.floor(Date.now() / 1000) - ts;
  if (diff < 60) return 'just now';
  if (diff < 3600) return Math.floor(diff / 60) + 'm ago';
  if (diff < 86400) return Math.floor(diff / 3600) + 'h ago';
  return Math.floor(diff / 86400) + 'd ago';
}

export function naturalSort(a: string, b: string): number {
  return a.localeCompare(b, undefined, { numeric: true, sensitivity: 'base' });
}

export function copyToClipboard(text: string): Promise<void> {
  if (navigator.clipboard && window.isSecureContext) {
    return navigator.clipboard.writeText(text);
  }
  const ta = document.createElement('textarea');
  ta.value = text;
  ta.style.cssText = 'position:fixed;opacity:0';
  document.body.appendChild(ta);
  ta.focus(); ta.select();
  document.execCommand('copy');
  document.body.removeChild(ta);
  return Promise.resolve();
}

const SEARCH_KEY = 'reelcast_searches';

export function getRecentSearches(): string[] {
  try { return JSON.parse(localStorage.getItem(SEARCH_KEY) || '[]'); } catch { return []; }
}

export function saveSearch(q: string) {
  if (!q) return;
  let searches = getRecentSearches().filter(s => s !== q);
  searches.unshift(q);
  localStorage.setItem(SEARCH_KEY, JSON.stringify(searches.slice(0, 10)));
}
