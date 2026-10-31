//! `/api/search` and `/api/suggest`.
//!
//! Sanskrit mode cascade: exact SLP1 key → inflected form (Heritage / JNU
//! paradigms) and ending variants (ज्ञानम् ↔ ज्ञान) → loose key equality →
//! prefix; when nothing matched, trigram "did you mean" over loose keys.
//! English mode: full-text search over `entries.search_en`.

use super::{ApiError, ApiResult, Dictionary, Entry, int_array, stmt};
use crate::entities::{dictionaries, entries, entry_keys, enums::Lang};
use crate::state::AppState;
use crate::translit::{ending_variants, loose_key, query_keys, root_variants, slp1_to_deva};
use axum::Json;
use axum::extract::{Query, State};
use sea_orm::{
  ColumnTrait, DbErr, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct SearchParams {
  q: Option<String>,
  mode: Option<String>,
  dicts: Option<String>,
  limit: Option<i64>,
}

#[derive(Serialize)]
pub struct EntryHit {
  #[serde(flatten)]
  entry: Entry,
  #[serde(rename = "match")]
  match_kind: &'static str,
  snippet: Option<String>,
}

#[derive(Serialize)]
pub struct DictionaryHits {
  dictionary: Dictionary,
  entries: Vec<EntryHit>,
  total: i64,
}

#[derive(Serialize)]
pub struct FormAnalysis {
  /// The analysed form (Devanagari)
  form: String,
  lemma: String,
  /// Verbal root for kṛdanta stems
  root: Option<String>,
  /// e.g. `पुमान् · तृतीया · एकः; नपुंसकम् · तृतीया · एकः`
  analysis: String,
  /// `heritage_decl` | `heritage_kridanta` | `heritage_conj` | `jnu_tinanta`
  source: String,
  /// Paradigms containing the form (first is the primary one)
  paradigm_ids: Vec<i32>,
  /// Cell numbers of the form in each paradigm (parallel to `paradigm_ids`)
  cells: Vec<Vec<i16>>,
  #[serde(skip)]
  lemma_slp1: String,
  #[serde(skip)]
  root_slp1: Option<String>,
}

#[derive(FromQueryResult)]
struct FormRow {
  id: i32,
  source: String,
  lemma: String,
  lemma_slp1: String,
  root: Option<String>,
  root_slp1: Option<String>,
  label: Option<String>,
  form_slp1: String,
  cells: Vec<i16>,
  tables: serde_json::Value,
}

const SOURCE_RANK: [&str; 4] = [
  "heritage_kridanta",
  "heritage_conj",
  "jnu_tinanta",
  "heritage_decl",
];

/// Look the query keys up in `paradigm_forms`, merging the hits per
/// (lemma, root, source). A stem that is also a kṛdanta of some root is shown
/// once, with the root. A form identical to its own stem is skipped (that is
/// the headword itself, already matched exactly).
async fn analyze_forms(state: &AppState, keys: &[String]) -> Result<Vec<FormAnalysis>, ApiError> {
  if keys.is_empty() {
    return Ok(vec![]);
  }
  // Cell descriptions only need labels: a layout has the same structure.
  let rows = FormRow::find_by_statement(stmt(
    "select p.id, p.source, p.lemma, p.lemma_slp1, p.root, p.root_slp1, p.label,
            f.form_slp1, f.cells, coalesce(p.tables, l.layout) as tables
     from paradigm_forms f join paradigms p on p.id = f.paradigm_id
     left join paradigm_layouts l on l.id = p.layout_id
     where f.form_slp1 = any($1) and p.lemma_slp1 <> f.form_slp1
     order by p.id limit 80",
    vec![keys.to_vec().into()],
  ))
  .all(&state.db)
  .await?;
  let kridanta_lemmas: std::collections::HashSet<String> = rows
    .iter()
    .filter(|r| r.source == "heritage_kridanta")
    .map(|r| r.lemma_slp1.clone())
    .collect();
  let mut out: Vec<FormAnalysis> = Vec::new();
  for r in rows {
    if r.source == "heritage_decl" && kridanta_lemmas.contains(&r.lemma_slp1) {
      continue;
    }
    let mut analysis = crate::import::paradigms::describe_cells(&r.tables, &r.cells);
    if let Some(l) = r
      .label
      .as_deref()
      .filter(|_| r.source == "heritage_decl" || r.source == "heritage_kridanta")
    {
      // The gender is already the table label; keep it only once.
      if !analysis.starts_with(l) {
        analysis = format!("{l} · {analysis}");
      }
    }
    if r.source == "heritage_conj"
      && let Some(l) = &r.label
    {
      analysis = format!("{l} · {analysis}");
    }
    match out
      .iter_mut()
      .find(|f| f.lemma_slp1 == r.lemma_slp1 && f.root_slp1 == r.root_slp1 && f.source == r.source)
    {
      Some(f) => {
        if !f.analysis.split("; ").any(|a| a == analysis) {
          f.analysis = format!("{}; {}", f.analysis, analysis);
        }
        f.paradigm_ids.push(r.id);
        f.cells.push(r.cells.clone());
      }
      None => out.push(FormAnalysis {
        form: slp1_to_deva(&r.form_slp1),
        lemma: r.lemma,
        root: r.root,
        analysis,
        source: r.source,
        paradigm_ids: vec![r.id],
        cells: vec![r.cells.clone()],
        lemma_slp1: r.lemma_slp1,
        root_slp1: r.root_slp1,
      }),
    }
  }
  out.sort_by_key(|f| SOURCE_RANK.iter().position(|s| *s == f.source).unwrap_or(9));
  Ok(out)
}

#[derive(Serialize)]
pub struct SearchResponse {
  query: String,
  slp1: Vec<String>,
  mode: &'static str,
  groups: Vec<DictionaryHits>,
  forms: Vec<FormAnalysis>,
  did_you_mean: Vec<String>,
}

const MATCH_RANK: [&str; 5] = ["exact", "form", "fuzzy", "prefix", "fulltext"];

async fn dict_filter(state: &AppState, dicts: &Option<String>) -> Result<Option<Vec<i32>>, DbErr> {
  let Some(d) = dicts.as_deref().filter(|d| !d.trim().is_empty()) else {
    return Ok(None);
  };
  let slugs: Vec<String> = d
    .split(',')
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .collect();
  let ids: Vec<i32> = dictionaries::Entity::find()
    .select_only()
    .column(dictionaries::Column::Id)
    .filter(dictionaries::Column::Slug.is_in(slugs))
    .into_tuple()
    .all(&state.db)
    .await?;
  Ok(Some(ids))
}

#[derive(Clone, Copy)]
enum KeyColumn {
  Slp1,
  Loose,
}

/// (entry_id, dictionary_id) matched by keys.
async fn match_keys(
  state: &AppState,
  column: KeyColumn,
  keys: &[String],
  dicts: &Option<Vec<i32>>,
) -> Result<Vec<(i32, i32)>, DbErr> {
  if keys.is_empty() {
    return Ok(vec![]);
  }
  let col = match column {
    KeyColumn::Slp1 => entry_keys::Column::KeySlp1,
    KeyColumn::Loose => entry_keys::Column::KeyLoose,
  };
  let mut q = entry_keys::Entity::find()
    .select_only()
    .column(entry_keys::Column::EntryId)
    .column(entry_keys::Column::DictionaryId)
    .distinct()
    .filter(col.is_in(keys.to_vec()))
    .limit(5000);
  if let Some(d) = dicts {
    q = q.filter(entry_keys::Column::DictionaryId.is_in(d.clone()));
  }
  q.into_tuple().all(&state.db).await
}

#[derive(FromQueryResult)]
struct KeyRow {
  key_slp1: String,
}

pub async fn search(
  State(state): State<AppState>,
  Query(p): Query<SearchParams>,
) -> ApiResult<SearchResponse> {
  let q = p.q.unwrap_or_default();
  let q = q.trim();
  if q.is_empty() {
    return Err(ApiError::bad_request("q is required"));
  }
  if q.chars().count() > 200 {
    return Err(ApiError::bad_request("q is too long"));
  }
  let limit = p.limit.unwrap_or(20).clamp(1, 100);
  let dicts = dict_filter(&state, &p.dicts).await?;
  let all_dicts = super::all_dictionaries(&state).await?;

  if p.mode.as_deref() == Some("en") {
    return english(&state, q, limit, &dicts, all_dicts).await;
  }

  let keys = query_keys(q);
  let mut hits: HashMap<i32, (i32, &'static str)> = HashMap::new(); // entry → (dict, match)
  let add =
    |rows: Vec<(i32, i32)>, kind: &'static str, hits: &mut HashMap<i32, (i32, &'static str)>| {
      for (e, d) in rows {
        hits.entry(e).or_insert((d, kind));
      }
    };

  // 1. exact
  add(
    match_keys(&state, KeyColumn::Slp1, &keys, &dicts).await?,
    "exact",
    &mut hits,
  );

  // 2. inflected forms → lemma (and root for kṛdanta / conjugation); ending variants
  let forms = analyze_forms(&state, &keys).await?;
  let mut form_keys: Vec<String> = Vec::new();
  for f in &forms {
    let mut ks = root_variants(&f.lemma_slp1, &f.lemma_slp1);
    if let Some(r) = &f.root_slp1 {
      ks.extend(root_variants(r, r));
    }
    for k in ks {
      if !form_keys.contains(&k) && !keys.contains(&k) {
        form_keys.push(k);
      }
    }
  }
  for k in &keys {
    for v in ending_variants(k) {
      if !form_keys.contains(&v) && !keys.contains(&v) {
        form_keys.push(v);
      }
    }
  }
  add(
    match_keys(&state, KeyColumn::Slp1, &form_keys, &dicts).await?,
    "form",
    &mut hits,
  );

  // 3. loose equality (vowel length, jñ typed as jn, sibilants, …)
  let loose: Vec<String> = {
    let mut v: Vec<String> = keys
      .iter()
      .map(|k| loose_key(k))
      .filter(|k| k.len() >= 2)
      .collect();
    v.dedup();
    v
  };
  if hits.len() < 5 {
    add(
      match_keys(&state, KeyColumn::Loose, &loose, &dicts).await?,
      "fuzzy",
      &mut hits,
    );
  }

  // 4. prefix (C-collated b-tree range scan)
  if hits.is_empty()
    && let Some(k) = keys.first().filter(|k| k.len() >= 3)
  {
    let mut pq = entry_keys::Entity::find()
      .select_only()
      .column(entry_keys::Column::EntryId)
      .column(entry_keys::Column::DictionaryId)
      .filter(entry_keys::Column::KeySlp1.starts_with(k))
      .order_by_asc(entry_keys::Column::KeySlp1)
      .limit(200);
    if let Some(d) = &dicts {
      pq = pq.filter(entry_keys::Column::DictionaryId.is_in(d.clone()));
    }
    add(pq.into_tuple().all(&state.db).await?, "prefix", &mut hits);
  }

  // 5. did you mean (trigram similarity on loose keys)
  let mut did_you_mean = Vec::new();
  if hits.is_empty()
    && let Some(l) = loose.first()
  {
    let rows = KeyRow::find_by_statement(stmt(
      "select key_slp1 from (
         select key_slp1, similarity(key_loose, $1) as sim from entry_keys
         where key_loose % $1 order by sim desc limit 60) t
       group by key_slp1 order by max(sim) desc limit 8",
      vec![l.clone().into()],
    ))
    .all(&state.db)
    .await?;
    did_you_mean = rows
      .into_iter()
      .map(|r| slp1_to_deva(&r.key_slp1))
      .collect();
  }

  let groups = load_groups(&state, hits, limit, all_dicts).await?;
  Ok(Json(SearchResponse {
    query: q.to_string(),
    slp1: keys,
    mode: "sa",
    groups,
    forms,
    did_you_mean,
  }))
}

async fn entries_by_ids(state: &AppState, ids: Vec<i32>) -> Result<Vec<Entry>, DbErr> {
  Ok(
    entries::Entity::find()
      .filter(entries::Column::Id.is_in(ids))
      .all(&state.db)
      .await?
      .into_iter()
      .map(Entry::from)
      .collect(),
  )
}

async fn load_groups(
  state: &AppState,
  hits: HashMap<i32, (i32, &'static str)>,
  limit: i64,
  all_dicts: Vec<Dictionary>,
) -> Result<Vec<DictionaryHits>, ApiError> {
  // Per dictionary: order by match rank then book order, keep `limit`.
  let mut per_dict: HashMap<i32, Vec<(usize, i32)>> = HashMap::new();
  for (e, (d, kind)) in &hits {
    let rank = MATCH_RANK.iter().position(|k| k == kind).unwrap_or(9);
    per_dict.entry(*d).or_default().push((rank, *e));
  }
  let mut wanted: Vec<i32> = Vec::new();
  let mut totals: HashMap<i32, i64> = HashMap::new();
  for (d, v) in per_dict.iter_mut() {
    totals.insert(*d, v.len() as i64);
    v.sort();
    wanted.extend(v.iter().take(limit as usize).map(|x| x.1));
  }
  let mut by_id: HashMap<i32, Entry> = entries_by_ids(state, wanted)
    .await?
    .into_iter()
    .map(|e| (e.id, e))
    .collect();

  let mut groups = Vec::new();
  for d in all_dicts {
    let Some(v) = per_dict.get(&d.id) else {
      continue;
    };
    let mut items: Vec<(usize, i32, EntryHit)> = Vec::new();
    for (rank, id) in v.iter().take(limit as usize) {
      if let Some(entry) = by_id.remove(id) {
        let seq = entry.seq;
        items.push((
          *rank,
          seq,
          EntryHit {
            entry,
            match_kind: MATCH_RANK[*rank],
            snippet: None,
          },
        ));
      }
    }
    items.sort_by_key(|x| (x.0, x.1));
    groups.push(DictionaryHits {
      total: totals.get(&d.id).copied().unwrap_or(0),
      dictionary: d,
      entries: items.into_iter().map(|x| x.2).collect(),
    });
  }
  Ok(groups)
}

fn escape_mark(s: &str) -> String {
  crate::import::markup::escape(s)
    .replace('\u{1}', "<mark>")
    .replace('\u{2}', "</mark>")
}

#[derive(FromQueryResult)]
struct FtsRow {
  id: i32,
  headline: String,
}

#[derive(FromQueryResult)]
struct FtsTotal {
  dictionary_id: i32,
  total: i64,
}

async fn english(
  state: &AppState,
  q: &str,
  limit: i64,
  dicts: &Option<Vec<i32>>,
  all_dicts: Vec<Dictionary>,
) -> ApiResult<SearchResponse> {
  // Rank: exact English headword ×50 (English→Sanskrit works), cover density
  // (log-length normalized), ×5 when the term occurs in the first ~40 chars of
  // the gloss (अग्नि: "fire, sacrificial fire"), ties → shorter gloss.
  let rows = FtsRow::find_by_statement(stmt(
    "with q as (select websearch_to_tsquery('english', $1) as q),
     top as (
       select e.id, e.dictionary_id, e.gloss_en,
              row_number() over (partition by e.dictionary_id order by
                (case when lower(e.headword) = lower($1) then 50
                      when lower(e.headword) like lower($1) || '%' then 3 else 1 end)
                * ts_rank_cd(e.search_en, q.q, 1)
                * (case when to_tsvector('english', left(e.gloss_en, 40)) @@ q.q then 5 else 1 end) desc,
                length(e.gloss_en), e.seq) as rn
       from entries e, q
       where e.gloss_en is not null and e.search_en @@ q.q
         and ($2::int[] is null or e.dictionary_id = any($2))
     )
     select top.id,
            ts_headline('english', top.gloss_en, q.q,
              'StartSel=\u{1}, StopSel=\u{2}, MaxFragments=2, MaxWords=24, MinWords=8, FragmentDelimiter= … ') as headline
     from top, q where rn <= $3 order by top.dictionary_id, rn",
    vec![q.into(), int_array(dicts.as_ref()), limit.into()],
  ))
  .all(&state.db)
  .await?;

  let totals: HashMap<i32, i64> = FtsTotal::find_by_statement(stmt(
    "select dictionary_id, count(*) as total from entries
     where gloss_en is not null and search_en @@ websearch_to_tsquery('english', $1)
       and ($2::int[] is null or dictionary_id = any($2))
     group by dictionary_id",
    vec![q.into(), int_array(dicts.as_ref())],
  ))
  .all(&state.db)
  .await?
  .into_iter()
  .map(|t| (t.dictionary_id, t.total))
  .collect();

  let order: HashMap<i32, usize> = rows.iter().enumerate().map(|(i, r)| (r.id, i)).collect();
  let snippets: HashMap<i32, String> = rows
    .iter()
    .map(|r| (r.id, escape_mark(&r.headline)))
    .collect();
  let mut entries = entries_by_ids(state, rows.iter().map(|r| r.id).collect()).await?;
  entries.sort_by_key(|e| order.get(&e.id).copied().unwrap_or(usize::MAX));

  // English→Sanskrit works first, then Sanskrit→English by their usual order.
  let mut all_dicts = all_dicts;
  all_dicts.sort_by_key(|d| (d.src_lang != Lang::En, d.sort_order));
  let mut groups = Vec::new();
  for d in all_dicts {
    let es: Vec<EntryHit> = entries
      .iter()
      .filter(|e| e.dictionary_id == d.id)
      .map(|e| EntryHit {
        entry: e.clone(),
        match_kind: "fulltext",
        snippet: snippets.get(&e.id).cloned(),
      })
      .collect();
    if !es.is_empty() {
      groups.push(DictionaryHits {
        total: totals.get(&d.id).copied().unwrap_or(es.len() as i64),
        dictionary: d,
        entries: es,
      });
    }
  }
  Ok(Json(SearchResponse {
    query: q.to_string(),
    slp1: vec![],
    mode: "en",
    groups,
    forms: vec![],
    did_you_mean: vec![],
  }))
}

#[derive(Deserialize)]
pub struct SuggestParams {
  q: Option<String>,
  limit: Option<i64>,
}

#[derive(Serialize, FromQueryResult)]
struct SuggestRow {
  key_slp1: String,
  count: i64,
}

#[derive(Serialize)]
pub struct Suggestion {
  headword: String,
  slp1: String,
  count: i64,
}

pub async fn suggest(
  State(state): State<AppState>,
  Query(p): Query<SuggestParams>,
) -> ApiResult<Vec<Suggestion>> {
  let limit = p.limit.unwrap_or(12).clamp(1, 50);
  let keys = query_keys(p.q.as_deref().unwrap_or(""));
  let Some(k) = keys.first().filter(|k| !k.is_empty()) else {
    return Ok(Json(vec![]));
  };
  // Bounded prefix scan on the C-collated b-tree, then shortest keys first.
  // Concordances (pāda phrases) are excluded: they are not headwords.
  let rows = SuggestRow::find_by_statement(stmt(
    "with c as (
       select key_slp1 from entry_keys
       where key_slp1 like $1 || '%'
         and dictionary_id not in (select id from dictionaries where kind = 'concordance')
       order by key_slp1 limit 600)
     select key_slp1, count(*) as count from c group by key_slp1
     order by length(key_slp1), key_slp1 limit $2",
    vec![k.clone().into(), limit.into()],
  ))
  .all(&state.db)
  .await?;
  Ok(Json(
    rows
      .into_iter()
      .map(|r| Suggestion {
        headword: slp1_to_deva(&r.key_slp1),
        slp1: r.key_slp1,
        count: r.count,
      })
      .collect(),
  ))
}
