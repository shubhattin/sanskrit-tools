// Typed mirror of docs/api.md. Keep in sync with the Rust axum contract.

export type DictKind = 'lexicon' | 'thesaurus' | 'dhatu_work' | 'sutra_commentary' | 'concordance';
export type Lang = 'sa' | 'en' | 'hi';
export type SearchMode = 'sa' | 'en';
export type MatchKind = 'exact' | 'form' | 'prefix' | 'fuzzy' | 'fulltext';

export interface Dictionary {
  id: number;
  slug: string;
  name: string;
  short_name: string;
  kind: DictKind;
  src_lang: Lang;
  tgt_lang: Lang;
  description: string | null;
  attribution: string | null;
  entry_count: number;
  sort_order: number;
}

export interface Entry {
  id: number;
  dictionary_id: number;
  seq: number;
  headword: string;
  senses: string[];
  data: Record<string, unknown> | null;
  sutra_id: string | null;
  dhatu_id: number | null;
}

export interface EntryHit extends Entry {
  match: MatchKind;
  snippet: string | null;
}

export interface DictionaryHits {
  dictionary: Dictionary;
  entries: EntryHit[];
  total: number;
}

export interface FormAnalysis {
  form: string;
  lemma: string;
  root: string | null;
  analysis: string;
  source: ParadigmSource;
  paradigm_ids: number[];
  cells: number[][];
}

export type ParadigmSource =
  | 'heritage_decl'
  | 'heritage_kridanta'
  | 'heritage_conj'
  | 'jnu_tinanta';

export interface ParadigmSummary {
  id: number;
  source: ParadigmSource;
  lemma: string;
  root: string | null;
  label: string | null;
}

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

export interface Paradigm extends ParadigmSummary {
  tables: ParadigmTable[];
}

export const PARADIGM_SOURCE_LABELS: Record<ParadigmSource, string> = {
  heritage_decl: 'Declension',
  heritage_kridanta: 'Kṛdanta',
  heritage_conj: 'Conjugation',
  jnu_tinanta: 'Tiṅanta (JNU)'
};

export interface SearchResponse {
  query: string;
  slp1: string[];
  mode: SearchMode;
  groups: DictionaryHits[];
  forms: FormAnalysis[];
  did_you_mean: string[];
}

export interface Suggestion {
  headword: string;
  slp1: string;
  count: number;
}

export interface BrowseItem {
  id: number;
  seq: number;
  headword: string;
}

export interface BrowseResponse {
  dictionary: Dictionary;
  items: BrowseItem[];
}

export interface Sutra {
  id: string;
  adhyaya: number;
  pada: number;
  num: number;
  text: string;
}

export interface SutraDetail {
  sutra: Sutra;
  prev: string | null;
  next: string | null;
  commentaries: { dictionary: Dictionary; entries: Entry[] }[];
}

export interface Dhatu {
  id: number;
  number: string;
  upadesha: string;
  root: string;
  artha: string;
  gana: number;
  pada: string | null;
  it: string | null;
  extra: string | null;
}

export interface DhatuDetail {
  dhatu: Dhatu;
  works: { dictionary: Dictionary; entries: Entry[] }[];
}

export interface EntryResponse {
  entry: Entry;
  dictionary: Dictionary;
}

export const DICT_KIND_LABELS: Record<DictKind, string> = {
  lexicon: 'Lexicons',
  thesaurus: 'Thesauri',
  dhatu_work: 'Dhātu works',
  sutra_commentary: 'Sūtra commentaries',
  concordance: 'Concordances'
};

export const DICT_KIND_ORDER: DictKind[] = [
  'lexicon',
  'thesaurus',
  'dhatu_work',
  'sutra_commentary',
  'concordance'
];
