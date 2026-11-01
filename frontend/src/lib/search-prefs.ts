import { useSyncExternalStore } from 'react';
import { z } from 'zod';

/**
 * Search preferences persisted in localStorage: which dictionaries to search
 * and whether to analyse inflected forms. Parsed defensively so a stale or
 * hand-edited value never breaks the page (it falls back per field).
 */

const STORAGE_KEY = 'sans-dict-search-prefs';

/**
 * Monier-Williams and Apte, plus their English→Sanskrit volumes so that
 * English-mode search finds English headwords in the same two works.
 */
export const DEFAULT_DICTS: readonly string[] = ['mw', 'apte', 'mw-en', 'apte-en'];

export interface SearchPrefs {
  /** Selected dictionary slugs, or 'all'. Never an empty list. */
  dicts: string[] | 'all';
  /** Look up inflected forms (रामेण → राम, गतेन → गम्). */
  forms: boolean;
}

export const DEFAULT_PREFS: SearchPrefs = { dicts: [...DEFAULT_DICTS], forms: false };

const slugSchema = z.string().regex(/^[a-z0-9-]{1,64}$/);
const prefsSchema = z.object({
  v: z.literal(1),
  dicts: z
    .union([z.literal('all'), z.array(slugSchema).min(1).max(200)])
    .catch(DEFAULT_PREFS.dicts),
  forms: z.boolean().catch(DEFAULT_PREFS.forms)
});

function parse(raw: string | null): SearchPrefs {
  if (!raw) return DEFAULT_PREFS;
  try {
    const r = prefsSchema.safeParse(JSON.parse(raw));
    if (!r.success) return DEFAULT_PREFS;
    const dicts = r.data.dicts === 'all' ? 'all' : [...new Set(r.data.dicts)];
    return { dicts, forms: r.data.forms };
  } catch {
    return DEFAULT_PREFS;
  }
}

let cachedRaw: string | null | undefined;
let cached: SearchPrefs = DEFAULT_PREFS;
/** Used only when localStorage cannot be written. */
let memoryOnly: SearchPrefs | null = null;
const listeners = new Set<() => void>();

function readRaw(): string | null {
  try {
    return localStorage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
}

function getSnapshot(): SearchPrefs {
  if (memoryOnly) return memoryOnly;
  const raw = readRaw();
  if (raw !== cachedRaw) {
    cachedRaw = raw;
    cached = parse(raw);
  }
  return cached;
}

function subscribe(cb: () => void): () => void {
  listeners.add(cb);
  // Keep tabs in sync.
  const onStorage = (e: StorageEvent) => {
    if (e.key === STORAGE_KEY) cb();
  };
  window.addEventListener('storage', onStorage);
  return () => {
    listeners.delete(cb);
    window.removeEventListener('storage', onStorage);
  };
}

export function setSearchPrefs(patch: Partial<SearchPrefs>): void {
  const next = { ...getSnapshot(), ...patch };
  if (Array.isArray(next.dicts) && next.dicts.length === 0) next.dicts = [...DEFAULT_DICTS];
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ v: 1, ...next }));
  } catch {
    // Storage unavailable (private mode / quota): keep it for this session.
    memoryOnly = next;
  }
  for (const l of listeners) l();
}

export function useSearchPrefs(): SearchPrefs {
  return useSyncExternalStore(subscribe, getSnapshot, () => DEFAULT_PREFS);
}

/**
 * Resolve stored slugs against the live dictionary list: unknown slugs are
 * dropped; if nothing survives, the defaults (or every dictionary) are used.
 */
export function resolveSelectedSlugs(
  prefs: SearchPrefs,
  available: readonly { slug: string }[]
): string[] {
  const all = available.map((d) => d.slug);
  if (prefs.dicts === 'all') return all;
  const known = new Set(all);
  const picked = prefs.dicts.filter((s) => known.has(s));
  if (picked.length > 0) return picked;
  const defaults = DEFAULT_DICTS.filter((s) => known.has(s));
  return defaults.length > 0 ? defaults : all;
}
