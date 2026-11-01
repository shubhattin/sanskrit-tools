# HTTP API contract (Rust axum, `/api/*`)

Dev: axum listens on `http://127.0.0.1:5780`; the Vite dev server proxies `/api` there.
All responses are JSON. Errors: `{ "error": string }` with a 4xx/5xx status.

Machine-readable spec: `frontend/openapi/schema.json` (utoipa, served from the
same `#[utoipa::path]` handlers — no drift by construction). Regenerate after
any route change with `bash scripts/sync-api.sh`; the frontend consumes only
the generated client in `frontend/src/lib/api/generated/`, with narrow
facades in `frontend/src/lib/api/types.ts`. `bun run check` fails if the
generated client is stale.

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

interface FormAnalysis {     // inflected form → stem / root (Heritage tables + JNU tiṅanta)
  form: string;              // Devanagari
  lemma: string;             // stem (declension) or root (conjugation)
  root: string | null;       // verbal root of a kṛdanta stem (गत → गम्)
  analysis: string;          // e.g. 'पुमान् · तृतीया · एकः; नपुंसकम् · तृतीया · एकः'
  source: ParadigmSource;
  paradigm_ids: number[];    // tables containing the form (first = primary)
  cells: number[][];         // cell numbers of the form, parallel to paradigm_ids
}

type ParadigmSource = 'heritage_decl' | 'heritage_kridanta' | 'heritage_conj' | 'jnu_tinanta';

interface ParadigmSummary { id: number; source: ParadigmSource; lemma: string; root: string | null;
  label: string | null; }   // label: gender (पुमान्/स्त्री/नपुंसकम्), 'गणः १', or JNU analysis

// Cells are numbered 0,1,2,… row-major across `tables` in order; each list item is one cell.
type ParadigmTable =
  | { group: string | null; title: string | null; label: string | null;
      cols: string[]; rows: { label: string; cells: string[][] }[] }   // a cell may hold 0..n forms
  | { group: string | null; title: string | null;
      list: { label: string; text: string; forms: string[] }[] };     // kṛdanta / avyaya lists
interface Paradigm extends ParadigmSummary { tables: ParadigmTable[]; }
// Conjugations: `group` = derivation (अप्रत्ययान्तधातु, णिच्, सन्, यङ्, कृदन्त, अव्यय …),
// `title` = lakāra (लट्, लङ् …), `label` = voice (परस्मैपदे / आत्मनेपदे / कर्मणि).

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
| GET | `/api/search` | `q` (required), `mode`=`sa`\|`en` (default `sa`), `dicts`=comma slugs (optional), `limit` per dict (default 20, max 100), `forms`=`0` to skip inflected-form analysis (default on; ending variants like ज्ञानम् ↔ ज्ञान still match) | `SearchResponse` |
| GET | `/api/suggest` | `q`, `limit` (default 12, max 50) | `Suggestion[]` (prefix autocomplete) |
| GET | `/api/entries/{id}` | – | `{ entry: Entry, dictionary: Dictionary }` |
| GET | `/api/dictionaries/{slug}/browse` | `seq` (center, default 0), `before` (default 15), `after` (default 30); or `q` to jump to a headword | `{ dictionary: Dictionary, items: BrowseItem[], center: number }` (`center` = seq jumped to) |
| GET | `/api/sutras` | `q` optional (number prefix like `1.1` or text); `all=1` for the whole list (q ignored) | `Sutra[]` (max 200 unless `all=1`) |
| GET | `/api/sutras/{id}` | – | `SutraDetail` |
| GET | `/api/dhatus` | `q` optional, `gana` optional | `Dhatu[]` (whole list, ≤ 2,282) |
| GET | `/api/dhatus/{id}` | – | `DhatuDetail` |
| GET | `/api/paradigms` | `q` (stem or root, any script) | `ParadigmSummary[]` (stem's declensions; for a root also its conjugation and all kṛdanta stems; max 300) |
| GET | `/api/paradigms/{id}` | – | `Paradigm` |

Search input may be Devanagari, IAST, Harvard-Kyoto, ITRANS, SLP1 or lipilekhika "Normal";
the server normalizes it. `mode=en` runs English full-text over Sanskrit→English lexicons and
English headwords (computer terms, Apte / Monier-Williams / Borooah English→Sanskrit).
In `en` mode English→Sanskrit works come first and an exact English headword ranks highest.
