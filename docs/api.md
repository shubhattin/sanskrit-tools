# HTTP API contract (Rust axum, `/api/*`)

Dev: axum listens on `http://127.0.0.1:5780`; the Vite dev server proxies `/api` there.
All responses are JSON. Errors: `{ "error": string }` with a 4xx/5xx status.

## Types

```ts
type DictKind = 'lexicon' | 'thesaurus' | 'dhatu_work' | 'sutra_commentary' | 'concordance';
type Lang = 'sa' | 'en' | 'hi';

interface Dictionary {
  id: number;
  slug: string;            // 'mw', 'apte', 'vacaspatyam', 'kashika', ...
  name: string;            // 'Monier-Williams Sanskrit-English Dictionary'
  short_name: string;      // 'MW'
  kind: DictKind;
  src_lang: Lang;
  tgt_lang: Lang;
  description: string | null;
  attribution: string | null;
  entry_count: number;
  sort_order: number;
}

interface Entry {
  id: number;
  dictionary_id: number;
  seq: number;              // book order inside its dictionary (0-based)
  headword: string;         // Devanagari (Latin for a few English/BHS headwords)
  // Each sense is a sanitized HTML fragment. Allowed tags: b, i, br, small, sup,
  // span.page (page ref), span.ref (citation), span.sa (Sanskrit run),
  // a[data-q] (cross reference: navigate to search with q=<data-q>, value is Devanagari).
  senses: string[];
  data: Record<string, unknown> | null;  // structured extras (Amara relations, RV location, ...)
  sutra_id: string | null;  // '1.1.001'
  dhatu_id: number | null;
}

interface EntryHit extends Entry {
  match: 'exact' | 'form' | 'prefix' | 'fuzzy' | 'fulltext';
  snippet: string | null;   // fulltext mode: HTML with <mark>…</mark>
}

interface DictionaryHits {
  dictionary: Dictionary;
  entries: EntryHit[];
  total: number;            // hits in this dictionary (entries may be truncated)
}

interface FormAnalysis {     // inflected-form → lemma (from JNU tiṅanta)
  form: string;              // Devanagari
  lemma: string;             // Devanagari root/stem
  analysis: string;          // e.g. 'भ्वादिगण, परस्मै, लट्'
}

interface SearchResponse {
  query: string;
  slp1: string[];            // SLP1 candidate keys the query was interpreted as
  mode: 'sa' | 'en';
  groups: DictionaryHits[];  // ordered by dictionary sort_order
  forms: FormAnalysis[];
  did_you_mean: string[];    // Devanagari headwords when nothing exact was found
}

interface Suggestion { headword: string; slp1: string; count: number; }

interface BrowseItem { id: number; seq: number; headword: string; }

interface Sutra { id: string; adhyaya: number; pada: number; num: number; text: string; }
interface SutraDetail { sutra: Sutra; prev: string | null; next: string | null;
  commentaries: { dictionary: Dictionary; entries: Entry[] }[]; }

interface Dhatu {
  id: number; number: string;        // '1.1147' (gaṇa.index)
  upadesha: string; root: string; artha: string;
  gana: number; pada: string | null; it: string | null; extra: string | null;
}
interface DhatuDetail { dhatu: Dhatu; works: { dictionary: Dictionary; entries: Entry[] }[]; }
```

## Endpoints

| Method | Path | Query | Response |
|---|---|---|---|
| GET | `/api/dictionaries` | – | `Dictionary[]` |
| GET | `/api/search` | `q` (required), `mode`=`sa`\|`en` (default `sa`), `dicts`=comma slugs (optional), `limit` per dict (default 20, max 100) | `SearchResponse` |
| GET | `/api/suggest` | `q`, `limit` (default 12, max 50) | `Suggestion[]` (prefix autocomplete) |
| GET | `/api/entries/{id}` | – | `{ entry: Entry, dictionary: Dictionary }` |
| GET | `/api/dictionaries/{slug}/browse` | `seq` (center, default 0), `before` (default 15), `after` (default 30); or `q` to jump to a headword | `{ dictionary: Dictionary, items: BrowseItem[], center: number }` (`center` = seq jumped to) |
| GET | `/api/sutras` | `q` optional (number prefix like `1.1` or text) | `Sutra[]` (max 200) |
| GET | `/api/sutras/{id}` | – | `SutraDetail` |
| GET | `/api/dhatus` | `q` optional, `gana` optional | `Dhatu[]` (max 300) |
| GET | `/api/dhatus/{id}` | – | `DhatuDetail` |

Search input may be Devanagari, IAST, Harvard-Kyoto, ITRANS, SLP1 or lipilekhika "Normal";
the server normalizes it. `mode=en` runs English full-text over Sanskrit→English lexicons and
English headwords (computer terms).
