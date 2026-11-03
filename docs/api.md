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

## Vidyut (`/api/vidyut/*`)

Grammar tools on Ambuda's [vidyut](https://github.com/ambuda-org/vidyut) (core: `src/vidyut/`,
handlers: `src/routes/vidyut.rs`; data embedded from `src/vidyut/data/`). Pure computation, public.
**Devanagari only** in and out (Latin letters → 400 "Please enter the text in Devanagari").
Generated types: `frontend/src/lib/api/generated/types.gen.ts` (`frontend/src/lib/vidyut/api.ts`).

```ts
type LakaraId = 'lat'|'lit'|'lut'|'lrt'|'lot'|'lan'|'vidhi_lin'|'ashir_lin'|'lun'|'lrn';
type PrayogaId = 'kartari'|'karmani'|'bhave';      type PadaId = 'parasmaipada'|'atmanepada';
type PurushaId = 'prathama'|'madhyama'|'uttama';   type VacanaId = 'eka'|'dvi'|'bahu';
type LingaId = 'pum'|'stri'|'napumsaka';
type VibhaktiId = 'prathama'|'dvitiya'|'trtiya'|'caturthi'|'panchami'|'sasthi'|'saptami'|'sambodhana';
type SanadiId = 'nic'|'san'|'yan'|'yan_luk';
type SamasaKind = 'tatpurusha'|'karmadharaya'|'dvigu'|'bahuvrihi'|'dvandva'|'samahara_dvandva'|'avyayibhava';

interface RootInfo { code: string;            // vidyut Dhātupāṭha code 'GG.NNNN'
  gana: number; number: number; aupadeshika: string; roots: string[]; artha: string; }
interface DhatuArgs { code: string; sanadi?: SanadiId[];   // in order: ['nic','san'] = desid. of caus.
  upasargas?: string[]; }                                   // Devanagari, outermost first; आ = आङ्
type StemArgs =
  | { kind: 'basic'; stem: string }
  | { kind: 'krdanta'; dhatu: DhatuArgs; krt: string;      // vidyut SLP1 label: 'kta', 'Satf~'
      lakara?: LakaraId | null; prayoga?: PrayogaId | null }   // śatṛ / śānac only
  | { kind: 'taddhitanta'; stem: string; taddhita: string } // 'aR', 'tva'
  | { kind: 'samasa'; padas: SamasaPada[]; samasa_type: SamasaKind };
interface SamasaPada { stem: string; vibhakti?: VibhaktiId; linga?: LingaId }  // first of avyayībhāva = indeclinable
type WordSpec =                                             // what /derive explains
  | { kind: 'tinanta'; dhatu: DhatuArgs; prayoga; lakara; purusha; vacana; pada?: PadaId | null }
  | { kind: 'subanta'; stem: StemArgs; linga; vibhakti; vacana }
  | { kind: 'stem'; stem: StemArgs }  | { kind: 'stryanta'; stem: StemArgs }
  | { kind: 'dhatu'; dhatu: DhatuArgs } | { kind: 'vakya'; words: string[] };

interface Conjugation { root: RootInfo; stems: string[]; prayoga: PrayogaId;
  tables: { pada: PadaId; lakaras: { lakara: LakaraId; forms: string[][][] }[] }[] } // [purusha][vacana][variants]
interface Declension { stems: string[]; linga: LingaId; forms: string[][][] }      // [vibhakti][vacana][variants]
interface KrtGroup { key: string; label: string; items: { krt: string; label: string;
  lakara: LakaraId | null; prayoga: PrayogaId | null; gloss: string | null; forms: string[];
  lingas: LingaId[]; avyaya: boolean }[] }
interface TaddhitaForm { taddhita: string; label: string; gloss: string | null; forms: string[]; linga: LingaId }
interface SamasaResult { stems: string[]; words: { linga: LingaId; forms: string[] }[] }
interface Derivation { text: string;
  steps: { rule: RuleRef; terms: { text: string; changed: boolean }[] }[];
  optional_rules: { rule: RuleRef; applied: boolean }[] }
interface RuleRef { source: 'ashtadhyayi'|'varttika'|'dhatupatha'|'unadipatha'|'linganushasana'|
  'phit'|'kashika'|'kaumudi'|'other'; code: string; text: string | null;
  sutra_id: string | null;   // this site's sūtra id ('1.1.001') for Aṣṭādhyāyī / vārttika / Kāśikā
  url: string | null }
interface Scansion { lines: { syllables: { text: string; guru: boolean }[]; pattern: string;   // 'GGLG…'
    ganas: string[]; matras: number }[];
  best: MeterMatch | null; others: MeterMatch[] }
interface MeterMatch { name: string; kind: 'vrtta'|'jati'|'anushtubh';
  fit: 'full'|'pada'|'prefix'; note: string | null }                     // note: 'vipulā'
interface MeterInfo { name: string; kind; padas: { pattern: string;      // '.' = either
  ganas: string[]; yati: number[] }[]; matras: number[]; syllables: number[] }
```

| Method | Path | Body / query | Response |
|---|---|---|---|
| GET | `/api/vidyut/dhatus` | `q` (root, meaning word or code `1.1`), `gana` | `RootInfo[]` (exact roots first, max 60) |
| GET | `/api/vidyut/dhatus/{code}` | – | `{ root: RootInfo, dhatu_id: number \| null }` (`dhatu_id`: same root on `/dhatu/{id}`) |
| GET | `/api/vidyut/dhatus/site/{id}` | – | `RootInfo` for the site's root `id` (404 when vidyut has no safe match) |
| POST | `/api/vidyut/tinanta` | `{ dhatu: DhatuArgs, prayoga }` | `Conjugation` (one table set per pada the root takes) |
| POST | `/api/vidyut/krdanta` | `{ dhatu: DhatuArgs }` | `KrtGroup[]` (participles, avyaya, gerundives, nouns, other) |
| POST | `/api/vidyut/subanta` | `{ stem: StemArgs, linga }` | `Declension` |
| POST | `/api/vidyut/stryanta` | `{ stem: StemArgs }` | `string[]` feminine stems |
| POST | `/api/vidyut/taddhita` | `{ stem: string }` | `TaddhitaForm[]` (glossed common suffixes first) |
| POST | `/api/vidyut/samasa` | `{ padas: SamasaPada[] (2–6), samasa_type }` | `SamasaResult` |
| POST | `/api/vidyut/derive` | `WordSpec` | `Derivation[]` (one per result) |
| POST | `/api/vidyut/chandas` | `{ text }` (≤ 2,000 chars; a line per pāda / half-verse, daṇḍas end lines) | `Scansion` |
| GET | `/api/vidyut/meters` | – | `MeterInfo[]` (anuṣṭubh, vṛttas by length, jātis) |
| POST | `/api/vidyut/sandhi/join` | `{ words: string[] }` (2–12) | `string[]` joined phrase(s), spaced as printed (रामो गच्छति, तच्च) |
| POST | `/api/vidyut/sandhi/split` | `{ text }` | `{ chunk: string; splits: { left; right; left_known; right_known }[] }[]` (`*_known`: headword or inflected form in the dictionaries; both-known first) |
| GET | `/api/vidyut/sandhi/rules` | – | `{ first; second; result }[]` |

Notes: tables show one pause form where vidyut gives both (रामात् / रामाद्, 8.4.56); `/derive`
returns both. The site's Dhātupāṭha is another edition than vidyut's: roots are matched within
the gaṇa by upadeśa or plain root, preferring the same number (~95% of roots match).

