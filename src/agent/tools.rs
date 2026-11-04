//! The agent's dictionary tools (aisdk [`Tool`]s). Each one calls the same handler as the public
//! API, so the agent sees exactly what the site shows, then trims it for the model: HTML → text,
//! long texts capped, ids kept so the UI can link to the pages. Shapes: `docs/agent.md`.

use crate::agent::text::{senses_text, truncate};
use crate::agent::vidyut_tools as vt;
use crate::agent::web_search::{WebSearch, WebSearchInput, WebSearchOutput};
use crate::routes::{self, ApiError};
use crate::state::AppState;
use aisdk::core::tools::{Tool, ToolExecute};
use axum::extract::{Path, Query, State};
use schemars::{JsonSchema, schema_for};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::future::Future;
use std::sync::Arc;

/// Characters of entry text per search hit / per full entry / per commentary.
const HIT_CHARS: usize = 1_200;
const ENTRY_CHARS: usize = 12_000;
const COMMENTARY_CHARS: usize = 4_000;
const COMMENTARY_CHARS_REQUESTED: usize = 12_000;
const WORK_ENTRY_CHARS: usize = 2_500;
const LIST_MAX: usize = 40;

/// The tools plus their Responses-API function specs (aisdk keeps its tool list private, and
/// the OpenRouter model needs the specs to send).
pub struct AgentTools {
  pub tools: Vec<Tool>,
  pub specs: Vec<Value>,
}

/// What every tool call can use: the database, the reader's preferred dictionaries, and
/// OpenAI web search (`None` omits the `web_search` tool — needs `OPENAI_API_KEY`).
#[derive(Clone)]
pub struct ToolCtx {
  pub state: AppState,
  /// Slugs searched when the model doesn't name dictionaries.
  pub preferred_dicts: Arc<[String]>,
  pub web_search: Option<WebSearch>,
}

pub fn agent_tools(ctx: &ToolCtx) -> AgentTools {
  let mut t = AgentTools {
    tools: Vec::new(),
    specs: Vec::new(),
  };
  t.add::<SearchInput, _, _>(
    ctx,
    "search_dictionaries",
    "Look up a Sanskrit word (any script: Devanagari, IAST, HK, ITRANS, SLP1) or, with \
     mode \"en\", search English text across the Sanskrit–English lexicons and English→Sanskrit \
     works. Matches headwords exactly, then stems/variant spellings, inflected forms (रामेण → राम) \
     and close spellings. Returns hits grouped by dictionary (entry text trimmed; use get_entry \
     for the full article) and an analysis of inflected forms.",
    search_dictionaries,
  );
  t.add::<EntryInput, _, _>(
    ctx,
    "get_entry",
    "Full text of one dictionary entry by id (from search_dictionaries or browse_dictionary).",
    get_entry,
  );
  t.add::<SuggestInput, _, _>(
    ctx,
    "suggest_headwords",
    "Headwords that start with a prefix, across all dictionaries, with how many dictionaries \
     have each. Useful to check spellings or find compounds.",
    suggest_headwords,
  );
  t.add::<BrowseInput, _, _>(
    ctx,
    "browse_dictionary",
    "Headwords around a word in one dictionary's own (alphabetical) order: neighbouring \
     entries, derivatives and compounds listed next to it.",
    browse_dictionary,
  );
  t.add::<ParadigmsInput, _, _>(
    ctx,
    "find_paradigms",
    "Find inflection tables. For a nominal stem (prātipadika, e.g. राम, नदी, मनस्): its \
     declension (śabdarūpa / subanta) per gender. For a verbal root (e.g. भू, गम्): its \
     conjugation (dhāturūpa / tiṅanta: every lakāra and voice, plus causative णिच्, desiderative \
     सन्, intensive यङ्) and every kṛdanta stem derived from it (participles, गत, गन्तव्य …). \
     Then call get_paradigm with an id.",
    find_paradigms,
  );
  t.add::<ParadigmInput, _, _>(
    ctx,
    "get_paradigm",
    "One inflection table by id (from find_paradigms, or paradigm_ids of a form analysis). \
     Conjugations are large: pass `group` (अप्रत्ययान्तधातु, णिच्, सन्, यङ्, कृदन्त, अव्यय) \
     and/or `title` (lakāra, e.g. लट्, लङ्, लोट्, विधिलिङ्, लिट्, लुट्, लृट्, लुङ्) to get only \
     those tables. `groups` in the result lists everything available.",
    get_paradigm,
  );
  t.add::<DhatusInput, _, _>(
    ctx,
    "search_dhatus",
    "Search the Pāṇinian Dhātupāṭha by root, upadeśa form, meaning (artha) or number \
     (\"1.1\"), optionally within one gaṇa (1–10). Returns number, upadeśa, root, artha, \
     gaṇa, pada and it markers.",
    search_dhatus,
  );
  t.add::<DhatuInput, _, _>(
    ctx,
    "get_dhatu",
    "One Dhātupāṭha root by id with what the dhātu works say about it (Mādhavīya Dhātuvṛtti, \
     Kṣīrataraṅgiṇī, Dhātupradīpa, Kṛdantarūpamālā …).",
    get_dhatu,
  );
  t.add::<SutrasInput, _, _>(
    ctx,
    "search_sutras",
    "Find Aṣṭādhyāyī sūtras by number prefix (\"1.1\", \"3.2.123\") or by text (any script).",
    search_sutras,
  );
  t.add::<SutraInput, _, _>(
    ctx,
    "get_sutra",
    "One sūtra by id (a.p.nnn, e.g. 1.1.001) with its commentaries: Kāśikā (kashika), Nyāsa \
     (nyasa), Laghusiddhāntakaumudī (laghu-kaumudi), Bālamanoramā (balamanorama), \
     Tattvabodhinī (tattvabodhini), and its gaṇa from the Gaṇapāṭha when it has one. Pass \
     `commentaries` to get only those, at greater length.",
    get_sutra,
  );
  // Grammar tools on vidyut (generated, not looked up: any root / stem, with derivations).
  t.add::<vt::ConjugateInput, _, _>(
    ctx,
    "vidyut_conjugate",
    "Generate a verb's conjugation with Pāṇini's grammar (vidyut): every lakāra (or one) of a \
     Dhātupāṭha root in a prayoga, per pada, also with sanādi (causative णिच्, desiderative सन्, \
     intensive यङ्) and upasargas. Use for any root/prefix/sanādi combination, or when \
     find_paradigms has no table. Rows: puruṣa × (eka | dvi | bahu).",
    vt::conjugate,
  );
  t.add::<vt::DeclineInput, _, _>(
    ctx,
    "vidyut_decline",
    "Generate the declension of any nominal stem in a gender with Pāṇini's grammar (vidyut). \
     Rows: vibhakti × (eka | dvi | bahu).",
    vt::decline,
  );
  t.add::<vt::KrdantasInput, _, _>(
    ctx,
    "vidyut_krdantas",
    "Generate a root's kṛdantas (vidyut): participles (क्त, क्तवतु, शतृ, शानच्…), gerund, \
     infinitive, gerundives, agent/action nouns and other kṛt suffixes.",
    vt::krdantas,
  );
  t.add::<vt::DerivationInput, _, _>(
    ctx,
    "vidyut_derivation",
    "Step-by-step Pāṇinian derivation (prakriyā) of one verb form (root + lakāra, puruṣa, \
     vacana), noun form (stem + liṅga, vibhakti, vacana), or any `form` as vidyut_analyze reads \
     it (+ `analysis` to choose one; covers kṛdantas): each step's sūtra (with sutra_id for \
     /sutra links), its text, and the result. Use to explain how a form arises.",
    vt::derivation,
  );
  t.add::<vt::AnalyzeInput, _, _>(
    ctx,
    "vidyut_analyze",
    "Parse Sanskrit text word by word with Pāṇini's grammar (vidyut's kosha of every generated \
     form + cheda segmenter): undoes sandhi, then gives each word's possible analyses (stem or \
     kṛdanta root + suffix, gender, case, number; verbs: root, lakāra, person, number), \
     likeliest first. Use for forms the dictionaries don't list (उद्भवनम्, गमयिष्यति) and to \
     parse sentences or verses.",
    vt::analyze,
  );
  t.add::<vt::SandhiInput, _, _>(
    ctx,
    "vidyut_sandhi",
    "Sandhi: `join` words with the grammar's rules (रामः + अस्ति → रामोऽस्ति), or `split` a \
     joined word or phrase: best whole readings (incl. cheda's) with each word's analysis, and \
     two-word phonetic candidates.",
    vt::sandhi,
  );
  t.add::<vt::MeterInput, _, _>(
    ctx,
    "vidyut_meter",
    "Identify the meter of a verse: laghu/guru scansion and gaṇas per line, the meter \
     (vṛtta, jāti or anuṣṭubh) and other compatible meters.",
    vt::meter,
  );
  if ctx.web_search.is_some() {
    t.add::<WebSearchInput, _, _>(
      ctx,
      "web_search",
      "Search the web for what the dictionaries can't answer: scholarship, texts and editions, \
       people, places, current matters. Returns a factual summary of what was found, with \
       source URLs. Ask a specific question.",
      web_search,
    );
  }
  t
}

impl AgentTools {
  fn add<I, O, Fut>(
    &mut self,
    ctx: &ToolCtx,
    name: &str,
    description: &str,
    run: fn(ToolCtx, I) -> Fut,
  ) where
    I: DeserializeOwned + JsonSchema + Send + 'static,
    O: Serialize,
    Fut: Future<Output = Result<O, String>> + Send + 'static,
  {
    let mut schema = serde_json::to_value(schema_for!(I)).unwrap_or_else(|_| json!({}));
    if let Value::Object(map) = &mut schema {
      map.remove("$schema");
      map.remove("title");
    }
    self.specs.push(json!({
      "type": "function",
      "name": name,
      "description": description,
      "parameters": schema.clone(),
    }));
    let ctx = ctx.clone();
    let execute = ToolExecute::from_async(move |_, input: Value| {
      let ctx = ctx.clone();
      async move {
        let input: I = serde_json::from_value(input)
          .map_err(|e| aisdk::Error::ToolCallError(format!("invalid input: {e}")))?;
        let output = run(ctx, input).await.map_err(aisdk::Error::ToolCallError)?;
        serde_json::to_string(&output).map_err(|e| aisdk::Error::ToolCallError(e.to_string()))
      }
    });
    self.tools.push(Tool {
      name: name.to_string(),
      description: description.to_string(),
      input_schema: serde_json::from_value(schema).unwrap_or_default(),
      execute,
    });
  }
}

fn api_error(e: ApiError) -> String {
  e.1
}

fn to_json<T: Serialize>(v: T) -> Result<Value, String> {
  serde_json::to_value(v).map_err(|e| e.to_string())
}

fn params<T: DeserializeOwned>(v: Value) -> Result<T, String> {
  serde_json::from_value(v).map_err(|e| e.to_string())
}

fn str_of(v: &Value) -> String {
  v.as_str().unwrap_or_default().to_string()
}

fn senses_of(entry: &Value) -> String {
  let senses: Vec<String> = serde_json::from_value(entry["senses"].clone()).unwrap_or_default();
  senses_text(&senses)
}

/// `{ dictionary, entries }` groups → `{ dict, dict_name, entries: [{ id, headword, text }] }`.
fn work_groups(groups: &Value, chars: usize) -> Vec<Value> {
  groups
    .as_array()
    .into_iter()
    .flatten()
    .map(|g| {
      let entries: Vec<Value> = g["entries"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|e| {
          json!({
            "id": e["id"],
            "headword": e["headword"],
            "text": truncate(senses_of(e), chars).0,
          })
        })
        .collect();
      json!({
        "dict": g["dictionary"]["slug"],
        "dict_name": g["dictionary"]["short_name"],
        "entries": entries,
      })
    })
    .collect()
}

// --- search_dictionaries -----------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
struct SearchInput {
  /// Word or phrase. Sanskrit in any script; English with mode "en".
  query: String,
  /// Dictionary slugs to search. Omit to search the reader's preferred dictionaries; use
  /// ["all"] to search every dictionary.
  dicts: Option<Vec<String>>,
  /// "sa" (default): Sanskrit headword lookup. "en": English full-text search.
  mode: Option<String>,
  /// Hits per dictionary (default 5, max 20).
  limit: Option<u32>,
}

#[derive(Serialize)]
struct SearchOutput {
  query: String,
  mode: String,
  slp1: Value,
  searched: Value,
  results: Vec<Value>,
  forms: Vec<Value>,
  did_you_mean: Value,
}

async fn search_dictionaries(ctx: ToolCtx, input: SearchInput) -> Result<SearchOutput, String> {
  let state = ctx.state;
  let all = routes::all_dictionaries(&state)
    .await
    .map_err(|e| e.to_string())?;
  let requested = input.dicts.unwrap_or_else(|| ctx.preferred_dicts.to_vec());
  let searched: Option<Vec<String>> =
    if requested.is_empty() || requested.iter().any(|d| d == "all") {
      None
    } else {
      let unknown: Vec<&String> = requested
        .iter()
        .filter(|s| !all.iter().any(|d| &d.slug == *s))
        .collect();
      if !unknown.is_empty() {
        return Err(format!(
          "unknown dictionary slug(s): {} (see the dictionary list in your instructions)",
          unknown
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ")
        ));
      }
      Some(requested)
    };
  let mode = match input.mode.as_deref() {
    Some("en") => "en",
    _ => "sa",
  };
  let limit = input.limit.unwrap_or(5).clamp(1, 20);
  let p = params(json!({
    "q": input.query,
    "mode": mode,
    "dicts": searched.as_ref().map(|d| d.join(",")),
    "limit": limit,
    "forms": "1",
  }))?;
  let resp = to_json(
    routes::search::search(State(state), Query(p))
      .await
      .map_err(api_error)?
      .0,
  )?;
  let results = resp["groups"]
    .as_array()
    .into_iter()
    .flatten()
    .map(|g| {
      let entries: Vec<Value> = g["entries"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|e| {
          json!({
            "id": e["id"],
            "headword": e["headword"],
            "match": e["match"],
            "text": truncate(senses_of(e), HIT_CHARS).0,
          })
        })
        .collect();
      json!({
        "dict": g["dictionary"]["slug"],
        "dict_name": g["dictionary"]["short_name"],
        "total": g["total"],
        "entries": entries,
      })
    })
    .collect();
  let forms = resp["forms"]
    .as_array()
    .into_iter()
    .flatten()
    .map(|f| {
      json!({
        "form": f["form"], "lemma": f["lemma"], "root": f["root"], "analysis": f["analysis"],
        "source": f["source"], "paradigm_ids": f["paradigm_ids"],
      })
    })
    .collect();
  Ok(SearchOutput {
    query: str_of(&resp["query"]),
    mode: mode.to_string(),
    slp1: resp["slp1"].clone(),
    searched: searched.map_or(json!("all"), |d| json!(d)),
    results,
    forms,
    did_you_mean: resp["did_you_mean"].clone(),
  })
}

// --- get_entry ---------------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
struct EntryInput {
  /// Entry id
  id: i32,
}

async fn get_entry(ctx: ToolCtx, input: EntryInput) -> Result<Value, String> {
  let resp = to_json(
    routes::browse::entry(State(ctx.state), Path(input.id))
      .await
      .map_err(api_error)?
      .0,
  )?;
  let e = &resp["entry"];
  let (text, truncated) = truncate(senses_of(e), ENTRY_CHARS);
  Ok(json!({
    "id": e["id"],
    "headword": e["headword"],
    "dict": resp["dictionary"]["slug"],
    "dict_name": resp["dictionary"]["short_name"],
    "text": text,
    "truncated": truncated,
    "sutra_id": e["sutra_id"],
    "dhatu_id": e["dhatu_id"],
  }))
}

// --- suggest_headwords -------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
struct SuggestInput {
  /// Beginning of a headword, any script.
  prefix: String,
  /// Max suggestions (default 15, max 50).
  limit: Option<u32>,
}

async fn suggest_headwords(ctx: ToolCtx, input: SuggestInput) -> Result<Value, String> {
  let p = params(json!({ "q": input.prefix, "limit": input.limit.unwrap_or(15).clamp(1, 50) }))?;
  let rows = to_json(
    routes::search::suggest(State(ctx.state), Query(p))
      .await
      .map_err(api_error)?
      .0,
  )?;
  let items: Vec<Value> = rows
    .as_array()
    .into_iter()
    .flatten()
    .map(|r| json!({ "headword": r["headword"], "count": r["count"] }))
    .collect();
  Ok(json!({ "items": items }))
}

// --- browse_dictionary -------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
struct BrowseInput {
  /// Dictionary slug
  dict: String,
  /// Headword to jump to (any script); the nearest one is used if absent.
  headword: String,
  /// Headwords before it (default 5, max 30).
  before: Option<u32>,
  /// Headwords after it (default 15, max 50).
  after: Option<u32>,
}

async fn browse_dictionary(ctx: ToolCtx, input: BrowseInput) -> Result<Value, String> {
  let p = params(json!({
    "q": input.headword,
    "before": input.before.unwrap_or(5).min(30),
    "after": input.after.unwrap_or(15).min(50),
  }))?;
  let resp = to_json(
    routes::browse::browse(State(ctx.state), Path(input.dict), Query(p))
      .await
      .map_err(api_error)?
      .0,
  )?;
  Ok(json!({
    "dict": resp["dictionary"]["slug"],
    "dict_name": resp["dictionary"]["short_name"],
    "center": resp["center"],
    "items": resp["items"],
  }))
}

// --- find_paradigms / get_paradigm -------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
struct ParadigmsInput {
  /// Nominal stem or verbal root, any script.
  query: String,
}

async fn find_paradigms(ctx: ToolCtx, input: ParadigmsInput) -> Result<Value, String> {
  let p = params(json!({ "q": input.query }))?;
  let rows = to_json(
    routes::paradigms::list(State(ctx.state), Query(p))
      .await
      .map_err(api_error)?
      .0,
  )?;
  let all = rows.as_array().cloned().unwrap_or_default();
  Ok(json!({
    "query": input.query,
    "total": all.len(),
    "paradigms": all.into_iter().take(80).collect::<Vec<_>>(),
  }))
}

#[derive(Deserialize, JsonSchema)]
struct ParadigmInput {
  /// Paradigm id
  id: i32,
  /// Only tables of this derivation group (conjugations), e.g. अप्रत्ययान्तधातु, णिच्, सन्, यङ्,
  /// कृदन्त, अव्यय.
  group: Option<String>,
  /// Only tables with this title (conjugations: the lakāra, e.g. लट्).
  title: Option<String>,
}

async fn get_paradigm(ctx: ToolCtx, input: ParadigmInput) -> Result<Value, String> {
  let resp = to_json(
    routes::paradigms::detail(State(ctx.state), Path(input.id))
      .await
      .map_err(api_error)?
      .0,
  )?;
  let tables = resp["tables"].as_array().cloned().unwrap_or_default();

  let mut groups: Vec<(Value, Vec<Value>)> = Vec::new();
  for t in &tables {
    match groups.iter_mut().find(|(g, _)| *g == t["group"]) {
      Some((_, titles)) => {
        if !titles.contains(&t["title"]) {
          titles.push(t["title"].clone());
        }
      }
      None => groups.push((t["group"].clone(), vec![t["title"].clone()])),
    }
  }
  let matches = |field: &Value, want: &Option<String>| {
    want
      .as_deref()
      .is_none_or(|w| field.as_str().is_some_and(|f| f.trim() == w.trim()))
  };
  let filtered = input.group.is_some() || input.title.is_some();
  let selected: Vec<Value> = tables
    .into_iter()
    .filter(|t| matches(&t["group"], &input.group) && matches(&t["title"], &input.title))
    .collect();
  if filtered && selected.is_empty() {
    return Err(format!(
      "no table matches group={:?} title={:?}; available: {}",
      input.group,
      input.title,
      serde_json::to_string(&groups).unwrap_or_default()
    ));
  }
  Ok(json!({
    "id": resp["id"],
    "source": resp["source"],
    "lemma": resp["lemma"],
    "root": resp["root"],
    "label": resp["label"],
    "tables": selected,
    "groups": groups
      .into_iter()
      .map(|(group, titles)| json!({ "group": group, "titles": titles }))
      .collect::<Vec<_>>(),
    "filtered": filtered,
  }))
}

// --- dhātus ------------------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
struct DhatusInput {
  /// Root, upadeśa, meaning or number (any script). Omit to list a whole gaṇa.
  query: Option<String>,
  /// Gaṇa 1–10.
  gana: Option<i16>,
}

async fn search_dhatus(ctx: ToolCtx, input: DhatusInput) -> Result<Value, String> {
  if input.query.as_deref().is_none_or(|q| q.trim().is_empty()) && input.gana.is_none() {
    return Err("give a query or a gana".into());
  }
  let p = params(json!({ "q": input.query, "gana": input.gana }))?;
  let rows = to_json(
    routes::dhatus::list(State(ctx.state), Query(p))
      .await
      .map_err(api_error)?
      .0,
  )?;
  let all = rows.as_array().cloned().unwrap_or_default();
  Ok(json!({
    "total": all.len(),
    "dhatus": all.into_iter().take(LIST_MAX).collect::<Vec<_>>(),
  }))
}

#[derive(Deserialize, JsonSchema)]
struct DhatuInput {
  /// Dhātu id (from search_dhatus or an entry's dhatu_id).
  id: i32,
}

async fn get_dhatu(ctx: ToolCtx, input: DhatuInput) -> Result<Value, String> {
  let resp = to_json(
    routes::dhatus::detail(State(ctx.state), Path(input.id))
      .await
      .map_err(api_error)?
      .0,
  )?;
  Ok(json!({
    "dhatu": resp["dhatu"],
    "works": work_groups(&resp["works"], WORK_ENTRY_CHARS),
  }))
}

// --- web_search --------------------------------------------------------------------------------------

async fn web_search(ctx: ToolCtx, input: WebSearchInput) -> Result<WebSearchOutput, String> {
  let search = ctx
    .web_search
    .clone()
    .ok_or("web search is not configured (OPENAI_API_KEY)")?;
  search.search(&input.query).await
}

// --- sūtras ------------------------------------------------------------------------------------

#[derive(Deserialize, JsonSchema)]
struct SutrasInput {
  /// Number prefix ("1.1", "3.2.123") or words of the sūtra, any script.
  query: String,
}

async fn search_sutras(ctx: ToolCtx, input: SutrasInput) -> Result<Value, String> {
  let p = params(json!({ "q": input.query }))?;
  let rows = to_json(
    routes::sutras::list(State(ctx.state), Query(p))
      .await
      .map_err(api_error)?
      .0,
  )?;
  let all = rows.as_array().cloned().unwrap_or_default();
  let sutras: Vec<Value> = all
    .iter()
    .take(LIST_MAX)
    .map(|s| json!({ "id": s["id"], "text": s["text"] }))
    .collect();
  Ok(json!({ "total": all.len(), "sutras": sutras }))
}

#[derive(Deserialize, JsonSchema)]
struct SutraInput {
  /// Sūtra id a.p.nnn, e.g. "1.1.001" ("1.1.1" also works).
  id: String,
  /// Commentary slugs to include (default: all, shorter).
  commentaries: Option<Vec<String>>,
}

/// `1.1.1` → `1.1.001`.
fn normalize_sutra_id(id: &str) -> String {
  let parts: Vec<&str> = id.trim().split('.').collect();
  match parts.as_slice() {
    [a, p, n] => match n.parse::<u32>() {
      Ok(n) => format!("{a}.{p}.{n:03}"),
      Err(_) => id.trim().to_string(),
    },
    _ => id.trim().to_string(),
  }
}

async fn get_sutra(ctx: ToolCtx, input: SutraInput) -> Result<Value, String> {
  let resp = to_json(
    routes::sutras::detail(State(ctx.state), Path(normalize_sutra_id(&input.id)))
      .await
      .map_err(api_error)?
      .0,
  )?;
  let chars = if input.commentaries.is_some() {
    COMMENTARY_CHARS_REQUESTED
  } else {
    COMMENTARY_CHARS
  };
  let commentaries: Vec<Value> = resp["commentaries"]
    .as_array()
    .into_iter()
    .flatten()
    .filter(|g| {
      input.commentaries.as_ref().is_none_or(|want| {
        g["dictionary"]["slug"]
          .as_str()
          .is_some_and(|s| want.iter().any(|w| w == s))
      })
    })
    .map(|g| {
      let text = g["entries"]
        .as_array()
        .into_iter()
        .flatten()
        .map(senses_of)
        .collect::<Vec<_>>()
        .join("\n\n");
      let (text, truncated) = truncate(text, chars);
      json!({
        "dict": g["dictionary"]["slug"],
        "dict_name": g["dictionary"]["short_name"],
        "text": text,
        "truncated": truncated,
      })
    })
    .collect();
  Ok(json!({
    "sutra": { "id": resp["sutra"]["id"], "text": resp["sutra"]["text"] },
    "prev": resp["prev"],
    "next": resp["next"],
    "commentaries": commentaries,
  }))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn sutra_ids_are_zero_padded() {
    assert_eq!(normalize_sutra_id("1.1.1"), "1.1.001");
    assert_eq!(normalize_sutra_id(" 3.2.123 "), "3.2.123");
    assert_eq!(normalize_sutra_id("x"), "x");
  }
}
