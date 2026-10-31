// Typed mirror of docs/api.md, derived from the backend OpenAPI spec.
// Source of truth: `frontend/openapi/schema.json` (regenerate with
// `bash scripts/sync-api.sh`; `bun run check` verifies freshness).
// DTOs come from `./generated/types.gen`; the few fields the schema leaves
// wide (string/unknown) are narrowed back to the unions the UI relies on.

import type {
  Entry as GenEntry,
  EntryResponse as GenEntryResponse,
  FormAnalysis as GenFormAnalysis,
  Paradigm as GenParadigm,
  ParadigmSummary as GenParadigmSummary,
  DictionaryHits as GenDictionaryHits,
  SearchResponse as GenSearchResponse
} from './generated/types.gen';

export type {
  DictKind,
  Lang,
  Dictionary,
  DictEntries,
  BrowseItem,
  BrowseResponse,
  Suggestion,
  Sutra,
  SutraDetail,
  Dhatu,
  DhatuDetail
} from './generated/types.gen';

export type SearchMode = 'sa' | 'en';
export type MatchKind = 'exact' | 'form' | 'prefix' | 'fuzzy' | 'fulltext';

export type ParadigmSource =
  | 'heritage_decl'
  | 'heritage_kridanta'
  | 'heritage_conj'
  | 'jnu_tinanta';

// Backend serializes `data` as an object or null; the schema leaves it
// `unknown`, so narrow it back for `Object.entries` / `DataList` consumers.
export type Entry = Omit<GenEntry, 'data'> & {
  data: Record<string, unknown> | null;
};

export type EntryHit = Entry & {
  match: MatchKind;
  snippet: string | null;
};

export type DictionaryHits = Omit<GenDictionaryHits, 'entries'> & {
  entries: EntryHit[];
};

export type EntryResponse = Omit<GenEntryResponse, 'entry'> & {
  entry: Entry;
};

export type FormAnalysis = Omit<GenFormAnalysis, 'source'> & {
  source: ParadigmSource;
};

export type ParadigmSummary = Omit<GenParadigmSummary, 'source'> & {
  source: ParadigmSource;
};

export type ParadigmTable =
  | {
      group: string | null;
      title: string | null;
      label: string | null;
      cols: string[];
      rows: { label: string; cells: string[][] }[];
    }
  | {
      group: string | null;
      title: string | null;
      list: { label: string; text: string; forms: string[] }[];
    };

// `tables` is `serde_json::Value` on the backend; the UI renders the
// `ParadigmTable[]` shape documented in docs/api.md.
export type Paradigm = Omit<GenParadigm, 'source' | 'tables'> & {
  source: ParadigmSource;
  tables: ParadigmTable[];
};

export type SearchResponse = Omit<GenSearchResponse, 'mode' | 'groups' | 'forms'> & {
  mode: SearchMode;
  groups: DictionaryHits[];
  forms: FormAnalysis[];
};

export const PARADIGM_SOURCE_LABELS: Record<ParadigmSource, string> = {
  heritage_decl: 'Declension',
  heritage_kridanta: 'Kṛdanta',
  heritage_conj: 'Conjugation',
  jnu_tinanta: 'Tiṅanta (JNU)'
};

export const DICT_KIND_LABELS: Record<import('./generated/types.gen').DictKind, string> = {
  lexicon: 'Lexicons',
  thesaurus: 'Thesauri',
  dhatu_work: 'Dhātu works',
  sutra_commentary: 'Sūtra commentaries',
  concordance: 'Concordances'
};

export const DICT_KIND_ORDER: import('./generated/types.gen').DictKind[] = [
  'lexicon',
  'thesaurus',
  'dhatu_work',
  'sutra_commentary',
  'concordance'
];
