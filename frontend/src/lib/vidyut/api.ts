// Queries for `/api/vidyut/*` (contract: docs/api.md § Vidyut). Every tool is a pure
// computation over vidyut, so POST results are cached by their body like GETs.
import { queryOptions } from '@tanstack/react-query';
import { fetchJson } from '@/lib/api/client';
import type {
  ChunkSplit,
  Conjugation,
  Declension,
  Derivation,
  DhatuArgs,
  KrtGroup,
  LingaId,
  MeterInfo,
  PrayogaId,
  RootDetail,
  RootInfo,
  SamasaKind,
  SamasaPada,
  SamasaResult,
  SandhiRule,
  Scansion,
  StemArgs,
  TaddhitaForm,
  WordSpec
} from '@/lib/api/generated/types.gen';

export type {
  ChunkSplit,
  Conjugation,
  Declension,
  Derivation,
  DhatuArgs,
  KrdantaForm,
  KrtGroup,
  LakaraId,
  LingaId,
  MeterInfo,
  MeterMatch,
  PadaId,
  PrayogaId,
  PurushaId,
  RootInfo,
  RuleRef,
  SamasaKind,
  SamasaPada,
  SamasaResult,
  SanadiId,
  SandhiRule,
  Scansion,
  StemArgs,
  Step,
  TaddhitaForm,
  VacanaId,
  VibhaktiId,
  WordSpec
} from '@/lib/api/generated/types.gen';

const BASE = '/api/vidyut';
const FOREVER = { staleTime: Infinity, gcTime: 1000 * 60 * 30 } as const;

function post<T>(path: string, body: unknown): Promise<T> {
  return fetchJson<T>(`${BASE}${path}`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body)
  });
}

export function vRootsQuery(q: string, gana?: number) {
  return queryOptions({
    queryKey: ['vidyut', 'roots', q, gana ?? 0],
    queryFn: () => {
      const sp = new URLSearchParams();
      if (q) sp.set('q', q);
      if (gana) sp.set('gana', String(gana));
      return fetchJson<RootInfo[]>(`${BASE}/dhatus?${sp.toString()}`);
    },
    ...FOREVER
  });
}

export function vRootQuery(code: string) {
  return queryOptions({
    queryKey: ['vidyut', 'root', code],
    queryFn: () => fetchJson<RootDetail>(`${BASE}/dhatus/${encodeURIComponent(code)}`),
    ...FOREVER
  });
}

/** vidyut's root for a root of the site's Dhātupāṭha (404 when there is no safe match). */
export function vSiteRootQuery(dhatuId: number) {
  return queryOptions({
    queryKey: ['vidyut', 'site-root', dhatuId],
    queryFn: () => fetchJson<RootInfo>(`${BASE}/dhatus/site/${dhatuId}`),
    retry: false,
    ...FOREVER
  });
}

export function tinantaQuery(dhatu: DhatuArgs, prayoga: PrayogaId) {
  return queryOptions({
    queryKey: ['vidyut', 'tinanta', dhatu, prayoga],
    queryFn: () => post<Conjugation>('/tinanta', { dhatu, prayoga }),
    ...FOREVER
  });
}

export function krdantaQuery(dhatu: DhatuArgs) {
  return queryOptions({
    queryKey: ['vidyut', 'krdanta', dhatu],
    queryFn: () => post<KrtGroup[]>('/krdanta', { dhatu }),
    ...FOREVER
  });
}

export function subantaQuery(stem: StemArgs, linga: LingaId) {
  return queryOptions({
    queryKey: ['vidyut', 'subanta', stem, linga],
    queryFn: () => post<Declension>('/subanta', { stem, linga }),
    ...FOREVER
  });
}

export function stryantaQuery(stem: StemArgs) {
  return queryOptions({
    queryKey: ['vidyut', 'stryanta', stem],
    queryFn: () => post<string[]>('/stryanta', { stem }),
    ...FOREVER
  });
}

export function taddhitaQuery(stem: string) {
  return queryOptions({
    queryKey: ['vidyut', 'taddhita', stem],
    queryFn: () => post<TaddhitaForm[]>('/taddhita', { stem }),
    ...FOREVER
  });
}

export function samasaQuery(padas: SamasaPada[], samasa_type: SamasaKind) {
  return queryOptions({
    queryKey: ['vidyut', 'samasa', padas, samasa_type],
    queryFn: () => post<SamasaResult>('/samasa', { padas, samasa_type }),
    ...FOREVER
  });
}

export function deriveQuery(spec: WordSpec) {
  return queryOptions({
    queryKey: ['vidyut', 'derive', spec],
    queryFn: () => post<Derivation[]>('/derive', spec),
    ...FOREVER
  });
}

export function chandasQuery(text: string) {
  return queryOptions({
    queryKey: ['vidyut', 'chandas', text],
    queryFn: () => post<Scansion>('/chandas', { text }),
    ...FOREVER
  });
}

export function metersQuery() {
  return queryOptions({
    queryKey: ['vidyut', 'meters'],
    queryFn: () => fetchJson<MeterInfo[]>(`${BASE}/meters`),
    ...FOREVER
  });
}

export function joinQuery(words: string[]) {
  return queryOptions({
    queryKey: ['vidyut', 'join', words],
    queryFn: () => post<string[]>('/sandhi/join', { words }),
    ...FOREVER
  });
}

export function splitQuery(text: string) {
  return queryOptions({
    queryKey: ['vidyut', 'split', text],
    queryFn: () => post<ChunkSplit[]>('/sandhi/split', { text }),
    ...FOREVER
  });
}

export function sandhiRulesQuery() {
  return queryOptions({
    queryKey: ['vidyut', 'sandhi-rules'],
    queryFn: () => fetchJson<SandhiRule[]>(`${BASE}/sandhi/rules`),
    ...FOREVER
  });
}
