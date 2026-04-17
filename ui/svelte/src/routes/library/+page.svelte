<script lang="ts">
  import { onMount } from 'svelte';
  import type { MediaItem } from '$lib/types';
  import { fmtDur, fmtSize } from '$lib/utils';

  type SeriesGroup = {
    key: string;
    items: MediaItem[];
    dirs: string[];
  };

  let allItems = $state<MediaItem[]>([]);
  let seriesGroups = $state<SeriesGroup[]>([]);
  let ungrouped = $state<MediaItem[]>([]);
  let loading = $state(true);
  let expandedKeys = $state<Set<string>>(new Set());

  function toggle(key: string) {
    const next = new Set(expandedKeys);
    next.has(key) ? next.delete(key) : next.add(key);
    expandedKeys = next;
  }

  function epKey(filename: string): string | null {
    const m = filename.match(/[Ss](\d{1,2})[Ee](\d{1,2})/);
    return m ? `${m[1].padStart(2,'0')}${m[2].padStart(2,'0')}` : null;
  }

  // Extract meaningful tokens from a dir path (last 2 components, normalized)
  function dirTokens(dirs: string[]): Set<string> {
    const tokens = new Set<string>();
    const skipWords = new Set(['the','and','of','in','to','a','an','for','with','from']);
    for (const d of dirs) {
      const parts = d.replace(/\/+$/, '').split('/').slice(-2);
      for (const part of parts) {
        const normalized = part.replace(/[.\-_]/g, ' ').toLowerCase()
          .replace(/(s\d{1,2}|complete|720p|1080p|480p|webrip|dsnp|galaxytv|tgx|season\s*\d+|2160p|hevc|x264|x265)/g, '');
        for (const tok of normalized.split(/\s+/)) {
          if (tok.length > 3 && !skipWords.has(tok)) tokens.add(tok);
        }
      }
    }
    return tokens;
  }

  function shouldMerge(keyA: string, dirsA: string[], keyB: string, dirsB: string[]): boolean {
    // One key must be a substring of the other
    if (!keyA.includes(keyB) && !keyB.includes(keyA)) return false;
    // Dirs must share a common ancestor
    const ancestorsA = new Set(dirsA.map(d => d.split('/').slice(0,-1).join('/')));
    const ancestorsB = new Set(dirsB.map(d => d.split('/').slice(0,-1).join('/')));
    for (const a of ancestorsA) if (ancestorsB.has(a)) return true;
    // Or dir tokens overlap
    const tokA = dirTokens(dirsA);
    const tokB = dirTokens(dirsB);
    for (const t of tokA) if (tokB.has(t)) return true;
    return false;
  }

  function mergeGroups(groups: SeriesGroup[]): SeriesGroup[] {
    // Union-find style merge
    const merged = [...groups];
    let changed = true;
    while (changed) {
      changed = false;
      outer: for (let i = 0; i < merged.length; i++) {
        for (let j = i + 1; j < merged.length; j++) {
          if (shouldMerge(merged[i].key, merged[i].dirs, merged[j].key, merged[j].dirs)) {
            // Merge j into i — use the shorter key as canonical
            const canonical = merged[i].key.length <= merged[j].key.length
              ? merged[i].key : merged[j].key;
            merged[i] = {
              key: canonical,
              items: [...merged[i].items, ...merged[j].items],
              dirs: [...new Set([...merged[i].dirs, ...merged[j].dirs])],
            };
            merged.splice(j, 1);
            changed = true;
            break outer;
          }
        }
      }
    }
    return merged;
  }

  function buildStructure(items: MediaItem[]) {
    const groups = new Map<string, MediaItem[]>();
    const lone: MediaItem[] = [];
    for (const item of items) {
      if (item.series_key) {
        const g = groups.get(item.series_key) ?? [];
        g.push(item);
        groups.set(item.series_key, g);
      } else {
        lone.push(item);
      }
    }
    seriesGroups = [...groups.entries()]
      .map(([key, groupItems]) => ({
        key,
        dirs: [...new Set(groupItems.map(i => i.dir))].sort(),
        items: [...groupItems].sort((a, b) => {
          const ea = epKey(a.filename), eb = epKey(b.filename);
          if (ea && eb) return ea.localeCompare(eb, undefined, { numeric: true });
          return a.filename.localeCompare(b.filename, undefined, { numeric: true });
        }),
      }))
      .sort((a, b) => a.key.localeCompare(b.key));
    seriesGroups = mergeGroups(seriesGroups)
      .map(g => ({
        ...g,
        items: [...g.items].sort((a, b) => {
          const ea = epKey(a.filename), eb = epKey(b.filename);
          if (ea && eb) return ea.localeCompare(eb, undefined, { numeric: true });
          return a.filename.localeCompare(b.filename, undefined, { numeric: true });
        }),
      }))
      .sort((a, b) => a.key.localeCompare(b.key));
    ungrouped = lone.sort((a, b) => a.title.localeCompare(b.title));
  }

  function totalDur(items: MediaItem[]): string {
    const s = items.reduce((acc, i) => acc + (i.duration_secs ?? 0), 0);
    if (!s) return '';
    const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60);
    return h > 0 ? `${h}h ${m}m` : `${m}m`;
  }
  function totalSz(items: MediaItem[]): string {
    return fmtSize(items.reduce((acc, i) => acc + i.size_bytes, 0));
  }

  let copied = $state(false);

  function buildJson() {
    return JSON.stringify({
      series: seriesGroups.map(g => ({
        series_key: g.key,
        episode_count: g.items.length,
        dirs: g.dirs,
        episodes: g.items.map(i => ({
          title: i.title,
          filename: i.filename,
          dir: i.dir,
          duration_secs: i.duration_secs,
        })),
      })),
      movies: ungrouped.map(i => ({
        title: i.title,
        filename: i.filename,
        year: i.year,
        duration_secs: i.duration_secs,
      })),
      summary: {
        series_count: seriesGroups.length,
        movie_count: ungrouped.length,
        total_files: allItems.length,
      }
    }, null, 2);
  }

  async function copyJson() {
    const json = buildJson();
    try {
      await navigator.clipboard.writeText(json);
    } catch {
      const ta = document.createElement('textarea');
      ta.value = json; ta.style.cssText = 'position:fixed;opacity:0';
      document.body.appendChild(ta); ta.focus(); ta.select();
      document.execCommand('copy'); document.body.removeChild(ta);
    }
    copied = true;
    setTimeout(() => copied = false, 2000);
  }

  onMount(async () => {
    const res = await fetch('/media');
    allItems = await res.json();
    buildStructure(allItems);
    loading = false;
  });
</script>

<svelte:head><title>Library — Reelcast</title></svelte:head>

<div class="page">
  <header>
    <a href="/" class="back">← Back</a>
    <h1>Library Structure</h1>
    {#if !loading}
      <div class="summary-row">
        <p class="summary">{seriesGroups.length} series · {ungrouped.length} movies · {allItems.length} total</p>
        <button class="copy-btn" class:copied={copied} onclick={copyJson}>
          {copied ? '✓ Copied' : '⎘ Copy as JSON'}
        </button>
      </div>
    {/if}
  </header>

  {#if loading}
    <p class="muted">Loading…</p>
  {:else}
    <section>
      <h2 class="section-title">Series ({seriesGroups.length})</h2>
      {#each seriesGroups as g (g.key)}
        <div class="group">
          <button class="group-header" onclick={() => toggle(g.key)}>
            <span class="arrow">{expandedKeys.has(g.key) ? '▾' : '▸'}</span>
            <span class="group-name">{g.key}</span>
            <span class="pills">
              <span class="pill">{g.items.length} ep</span>
              {#if totalDur(g.items)}<span class="pill muted">{totalDur(g.items)}</span>{/if}
              <span class="pill muted">{totalSz(g.items)}</span>
              {#if g.dirs.length > 1}<span class="pill warn">{g.dirs.length} dirs</span>{/if}
            </span>
          </button>

          {#if expandedKeys.has(g.key)}
            <div class="group-body">
              {#if g.dirs.length > 1}
                <div class="dir-list">
                  {#each g.dirs as dir}<p class="dir">📁 {dir}</p>{/each}
                </div>
              {/if}
              <table>
                <thead><tr><th>#</th><th>Title</th><th>Dur</th><th>Filename</th></tr></thead>
                <tbody>
                  {#each g.items as item, i (item.id)}
                    <tr>
                      <td class="n">{i + 1}</td>
                      <td>{item.title}</td>
                      <td class="dur">{fmtDur(item.duration_secs)}</td>
                      <td class="file">{item.filename}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {/if}
        </div>
      {/each}
    </section>

    {#if ungrouped.length}
      <section>
        <h2 class="section-title">Movies & Ungrouped ({ungrouped.length})</h2>
        <div class="group">
          <button class="group-header" onclick={() => toggle('__movies__')}>
            <span class="arrow">{expandedKeys.has('__movies__') ? '▾' : '▸'}</span>
            <span class="group-name">All</span>
            <span class="pills">
              <span class="pill">{ungrouped.length} files</span>
              <span class="pill muted">{totalSz(ungrouped)}</span>
            </span>
          </button>
          {#if expandedKeys.has('__movies__')}
            <div class="group-body">
              <table>
                <thead><tr><th>Title</th><th>Year</th><th>Dur</th><th>Filename</th></tr></thead>
                <tbody>
                  {#each ungrouped as item (item.id)}
                    <tr>
                      <td>{item.title}</td>
                      <td class="n">{item.year ?? '—'}</td>
                      <td class="dur">{fmtDur(item.duration_secs)}</td>
                      <td class="file">{item.filename}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {/if}
        </div>
      </section>
    {/if}
  {/if}
</div>

<style>
  :global(body) { font-family: system-ui, sans-serif; background: #0f0f0f; color: #e0e0e0; margin: 0; }
  .page { max-width: 1100px; margin: 0 auto; padding: 2rem; }
  header { margin-bottom: 2rem; }
  .back { color: #f97316; text-decoration: none; font-size: 0.88rem; }
  .back:hover { text-decoration: underline; }
  h1 { font-size: 1.4rem; font-weight: 600; color: #fff; margin: 0.4rem 0 0.2rem; }
  .summary-row { display: flex; align-items: center; gap: 1rem; margin-top: 0.2rem; }
  .summary, .muted { color: #555; font-size: 0.85rem; }
  .copy-btn { padding: 0.3rem 0.8rem; border-radius: 6px; border: 1px solid #2a2a2a; background: #1a1a1a; color: #aaa; font-size: 0.8rem; cursor: pointer; }
  .copy-btn:hover { border-color: #f97316; color: #f97316; }
  .copy-btn.copied { background: #14532d; border-color: #14532d; color: #86efac; }
  .section-title { font-size: 0.72rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; color: #555; margin: 1.5rem 0 0.5rem; }

  .group { border: 1px solid #1e1e1e; border-radius: 8px; margin-bottom: 0.4rem; overflow: hidden; }
  .group-header { width: 100%; display: flex; align-items: center; gap: 0.75rem; padding: 0.6rem 1rem; background: #151515; border: none; color: #e0e0e0; cursor: pointer; text-align: left; }
  .group-header:hover { background: #1c1c1c; }
  .arrow { color: #555; font-size: 0.8rem; }
  .group-name { flex: 1; font-size: 0.92rem; font-weight: 600; text-transform: capitalize; }
  .pills { display: flex; gap: 0.35rem; }
  .pill { padding: 0.1rem 0.45rem; border-radius: 999px; background: #222; border: 1px solid #2a2a2a; font-size: 0.7rem; color: #aaa; white-space: nowrap; }
  .pill.muted { color: #555; }
  .pill.warn { background: #2a1a00; border-color: #78350f; color: #f97316; }

  .group-body { padding: 0.75rem 1rem 1rem; background: #0d0d0d; }
  .dir-list { margin-bottom: 0.6rem; }
  .dir { font-size: 0.72rem; color: #444; font-family: monospace; margin-bottom: 0.15rem; }

  table { width: 100%; border-collapse: collapse; font-size: 0.8rem; }
  th { text-align: left; color: #444; font-weight: 500; padding: 0.25rem 0.5rem; border-bottom: 1px solid #1a1a1a; }
  td { padding: 0.28rem 0.5rem; border-bottom: 1px solid #141414; vertical-align: top; }
  tr:last-child td { border-bottom: none; }
  tr:hover td { background: #141414; }
  .n { color: #444; width: 2.5rem; text-align: right; }
  .dur { color: #555; white-space: nowrap; width: 4.5rem; }
  .file { color: #3a3a3a; font-family: monospace; font-size: 0.7rem; word-break: break-all; }
</style>
