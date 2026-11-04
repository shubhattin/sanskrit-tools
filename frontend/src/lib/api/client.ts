import { queryOptions } from '@tanstack/react-query';
import type {
  BrowseResponse,
  Dhatu,
  DhatuDetail,
  Dictionary,
  EntryResponse,
  Paradigm,
  ParadigmSummary,
  SearchMode,
  SearchResponse,
  Suggestion,
  Sutra,
  SutraDetail
} from './types';

export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

export async function fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(path, init);
  if (!res.ok) {
    let message = `Request failed (${res.status})`;
    try {
      const body = (await res.json()) as { error?: string };
      if (body?.error) message = body.error;
    } catch {
      // Fall through to the generic message.
    }
    throw new ApiError(res.status, message);
  }
  return (await res.json()) as T;
}

const SEARCH_LIMIT = 20;

export interface SearchParams {
  q: string;
  mode: SearchMode;
  dicts?: string;
  /** Inflected-form analysis (Sanskrit mode). */
  forms: boolean;
}

export function dictionariesQuery() {
  return queryOptions({
    queryKey: ['dictionaries'],
    queryFn: () => fetchJson<Dictionary[]>('/api/dictionaries'),
    staleTime: 1000 * 60 * 60
  });
}

export function searchQuery(params: SearchParams) {
  const { q, mode, dicts, forms } = params;
  const withForms = mode === 'sa' && forms;
  return queryOptions({
    queryKey: ['search', q, mode, dicts ?? '', withForms],
    queryFn: () => {
      const sp = new URLSearchParams({ q, mode, limit: String(SEARCH_LIMIT) });
      if (dicts) sp.set('dicts', dicts);
      if (!withForms) sp.set('forms', '0');
      return fetchJson<SearchResponse>(`/api/search?${sp.toString()}`);
    },
    // Only run when there is an actual query; routes gate this with `enabled`.
    staleTime: 1000 * 60 * 5
  });
}

export function suggestQuery(q: string, limit = 12) {
  return queryOptions({
    queryKey: ['suggest', q, limit],
    queryFn: () => {
      const sp = new URLSearchParams({ q, limit: String(limit) });
      return fetchJson<Suggestion[]>(`/api/suggest?${sp.toString()}`);
    },
    staleTime: 1000 * 60 * 5
  });
}

export function entryQuery(id: number) {
  return queryOptions({
    queryKey: ['entry', id],
    queryFn: () => fetchJson<EntryResponse>(`/api/entries/${id}`)
  });
}

export interface BrowseParams {
  slug: string;
  seq?: number;
  before?: number;
  after?: number;
  q?: string;
}

export function browseQuery({ slug, seq, before, after, q }: BrowseParams) {
  return queryOptions({
    queryKey: ['browse', slug, seq ?? null, before ?? null, after ?? null, q ?? null],
    queryFn: () => {
      const sp = new URLSearchParams();
      if (q) sp.set('q', q);
      else sp.set('seq', String(seq ?? 0));
      sp.set('before', String(before ?? 15));
      sp.set('after', String(after ?? 30));
      return fetchJson<BrowseResponse>(
        `/api/dictionaries/${encodeURIComponent(slug)}/browse?${sp.toString()}`
      );
    }
  });
}

export function sutrasQuery(q?: string) {
  return queryOptions({
    queryKey: ['sutras', q ?? ''],
    queryFn: () => {
      const sp = new URLSearchParams();
      if (q) sp.set('q', q);
      const qs = sp.toString();
      return fetchJson<Sutra[]>(`/api/sutras${qs ? `?${qs}` : ''}`);
    },
    staleTime: 1000 * 60 * 30
  });
}

/** Every sūtra (≈4k) for on-device filtering. */
export function allSutrasQuery() {
  return queryOptions({
    queryKey: ['sutras', 'all'],
    queryFn: () => fetchJson<Sutra[]>('/api/sutras?all=1'),
    staleTime: Infinity,
    gcTime: 1000 * 60 * 60
  });
}

export function sutraQuery(id: string) {
  return queryOptions({
    queryKey: ['sutra', id],
    queryFn: () => fetchJson<SutraDetail>(`/api/sutras/${encodeURIComponent(id)}`)
  });
}

export function dhatusQuery(opts: { q?: string; gana?: number }) {
  return queryOptions({
    queryKey: ['dhatus', opts.q ?? '', opts.gana ?? 0],
    queryFn: () => {
      const sp = new URLSearchParams();
      if (opts.q) sp.set('q', opts.q);
      if (opts.gana) sp.set('gana', String(opts.gana));
      const qs = sp.toString();
      return fetchJson<Dhatu[]>(`/api/dhatus${qs ? `?${qs}` : ''}`);
    },
    staleTime: 1000 * 60 * 30
  });
}

export function dhatuQuery(id: number) {
  return queryOptions({
    queryKey: ['dhatu', id],
    queryFn: () => fetchJson<DhatuDetail>(`/api/dhatus/${id}`)
  });
}

export function paradigmsQuery(q: string) {
  return queryOptions({
    queryKey: ['paradigms', q],
    queryFn: () => {
      const sp = new URLSearchParams({ q });
      return fetchJson<ParadigmSummary[]>(`/api/paradigms?${sp.toString()}`);
    },
    staleTime: 1000 * 60 * 30
  });
}

export function paradigmQuery(id: number) {
  return queryOptions({
    queryKey: ['paradigm', id],
    queryFn: () => fetchJson<Paradigm>(`/api/paradigms/${id}`),
    staleTime: 1000 * 60 * 30
  });
}
