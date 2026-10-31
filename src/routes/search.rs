//! `/api/search` and `/api/suggest`.
//!
//! Sanskrit mode cascade: exact SLP1 key → inflected form (JNU tiṅanta) /
//! ending variants (ज्ञानम् ↔ ज्ञान) → loose key equality → prefix; when
//! nothing matched, trigram "did you mean" over loose keys.
//! English mode: full-text search over `entries.search_en`.

use super::{ApiError, ApiResult, Dictionary, ENTRY_COLS, Entry};
use crate::state::AppState;
use crate::translit::{ending_variants, loose_key, query_keys, root_variants, slp1_to_deva};
use axum::Json;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
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

#[derive(Serialize, FromRow)]
pub struct FormAnalysis {
  form: String,
  lemma: String,
  analysis: String,
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

async fn dict_filter(
  state: &AppState,
  dicts: &Option<String>,
) -> Result<Option<Vec<i32>>, ApiError> {
  let Some(d) = dicts.as_deref().filter(|d| !d.trim().is_empty()) else {
    return Ok(None);
  };
  let slugs: Vec<String> = d
    .split(',')
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .collect();
  let ids: Vec<(i32,)> = sqlx::query_as("select id from dictionaries where slug = any($1)")
    .bind(&slugs)
    .fetch_all(&state.db)
    .await?;
  Ok(Some(ids.into_iter().map(|r| r.0).collect()))
}

/// (entry_id, dictionary_id) matched by keys.
async fn match_keys(
  state: &AppState,
  column: &str,
  keys: &[String],
  dicts: &Option<Vec<i32>>,
) -> Result<Vec<(i32, i32)>, sqlx::Error> {
  if keys.is_empty() {
    return Ok(vec![]);
  }
  sqlx::query_as(&format!(
    "select distinct entry_id, dictionary_id from entry_keys
     where {column} = any($1) and ($2::int[] is null or dictionary_id = any($2)) limit 5000"
  ))
  .bind(keys)
  .bind(dicts)
  .fetch_all(&state.db)
  .await
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
    match_keys(&state, "key_slp1", &keys, &dicts).await?,
    "exact",
    &mut hits,
  );

  // 2. inflected forms → lemma; ending variants
  let forms: Vec<(String, String, String, String)> = sqlx::query_as(
    "select distinct form, lemma, analysis, lemma_slp1 from word_forms where form_slp1 = any($1) limit 40",
  )
  .bind(&keys)
  .fetch_all(&state.db)
  .await?;
  let mut form_keys: Vec<String> = Vec::new();
  for (_, _, _, lemma) in &forms {
    for k in root_variants(lemma, lemma) {
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
    match_keys(&state, "key_slp1", &form_keys, &dicts).await?,
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
      match_keys(&state, "key_loose", &loose, &dicts).await?,
      "fuzzy",
      &mut hits,
    );
  }

  // 4. prefix
  if hits.is_empty() && keys.first().is_some_and(|k| k.len() >= 3) {
    let rows: Vec<(i32, i32)> = sqlx::query_as(
      "select entry_id, dictionary_id from entry_keys
       where key_slp1 like $1 || '%' and ($2::int[] is null or dictionary_id = any($2))
       order by key_slp1 limit 200",
    )
    .bind(&keys[0])
    .bind(&dicts)
    .fetch_all(&state.db)
    .await?;
    add(rows, "prefix", &mut hits);
  }

  // 5. did you mean
  let mut did_you_mean = Vec::new();
  if hits.is_empty()
    && let Some(l) = loose.first()
  {
    let rows: Vec<(String,)> = sqlx::query_as(
      "select key_slp1 from (
           select key_slp1, similarity(key_loose, $1) as sim from entry_keys
           where key_loose % $1 order by sim desc limit 60) t
         group by key_slp1 order by max(sim) desc limit 8",
    )
    .bind(l)
    .fetch_all(&state.db)
    .await?;
    did_you_mean = rows.into_iter().map(|r| slp1_to_deva(&r.0)).collect();
  }

  let groups = load_groups(&state, hits, limit, all_dicts, HashMap::new()).await?;
  Ok(Json(SearchResponse {
    query: q.to_string(),
    slp1: keys,
    mode: "sa",
    groups,
    forms: forms
      .into_iter()
      .map(|(form, lemma, analysis, _)| FormAnalysis {
        form,
        lemma,
        analysis,
      })
      .collect(),
    did_you_mean,
  }))
}

async fn load_groups(
  state: &AppState,
  hits: HashMap<i32, (i32, &'static str)>,
  limit: i64,
  all_dicts: Vec<Dictionary>,
  snippets: HashMap<i32, String>,
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
    // rank then (for fulltext) snippet order is kept by caller via rank ties → id
    v.sort();
    wanted.extend(v.iter().take(limit as usize).map(|x| x.1));
  }
  let entries: Vec<Entry> = sqlx::query_as(&format!(
    "select {ENTRY_COLS} from entries e where e.id = any($1)"
  ))
  .bind(&wanted)
  .fetch_all(&state.db)
  .await?;
  let mut by_id: HashMap<i32, Entry> = entries.into_iter().map(|e| (e.id, e)).collect();

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
            snippet: snippets.get(id).cloned(),
          },
        ));
      }
    }
    if snippets.is_empty() {
      items.sort_by_key(|x| (x.0, x.1));
    }
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

async fn english(
  state: &AppState,
  q: &str,
  limit: i64,
  dicts: &Option<Vec<i32>>,
  all_dicts: Vec<Dictionary>,
) -> ApiResult<SearchResponse> {
  // Rank: cover density (log-length normalized), boosted ×5 when the term
  // occurs in the first ~40 chars of the gloss (अग्नि: "fire, sacrificial
  // fire"), ties → shorter gloss.
  let rows: Vec<(i32, i32, String)> = sqlx::query_as(
    "with q as (select websearch_to_tsquery('english', $1) as q),
     top as (
       select e.id, e.dictionary_id, e.gloss_en,
              row_number() over (partition by e.dictionary_id order by ts_rank_cd(e.search_en, q.q, 1)
                * (case when to_tsvector('english', left(e.gloss_en, 40)) @@ q.q then 5 else 1 end) desc,
                length(e.gloss_en), e.seq) as rn
       from entries e, q
       where e.gloss_en is not null and e.search_en @@ q.q
         and ($2::int[] is null or e.dictionary_id = any($2))
     )
     select top.id, top.dictionary_id,
            ts_headline('english', top.gloss_en, q.q,
              'StartSel=\u{1}, StopSel=\u{2}, MaxFragments=2, MaxWords=24, MinWords=8, FragmentDelimiter= … ')
     from top, q where rn <= $3 order by top.dictionary_id, rn",
  )
  .bind(q)
  .bind(dicts)
  .bind(limit)
  .fetch_all(&state.db)
  .await?;

  let totals: Vec<(i32, i64)> = sqlx::query_as(
    "select dictionary_id, count(*) from entries
     where gloss_en is not null and search_en @@ websearch_to_tsquery('english', $1)
       and ($2::int[] is null or dictionary_id = any($2))
     group by dictionary_id",
  )
  .bind(q)
  .bind(dicts)
  .fetch_all(&state.db)
  .await?;
  let totals: HashMap<i32, i64> = totals.into_iter().collect();

  let order: HashMap<i32, usize> = rows.iter().enumerate().map(|(i, r)| (r.0, i)).collect();
  let snippets: HashMap<i32, String> = rows.iter().map(|r| (r.0, escape_mark(&r.2))).collect();
  let ids: Vec<i32> = rows.iter().map(|r| r.0).collect();
  let mut entries: Vec<Entry> = sqlx::query_as(&format!(
    "select {ENTRY_COLS} from entries e where e.id = any($1)"
  ))
  .bind(&ids)
  .fetch_all(&state.db)
  .await?;
  entries.sort_by_key(|e| order.get(&e.id).copied().unwrap_or(usize::MAX));

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
  let rows: Vec<(String, i64)> = sqlx::query_as(
    "with c as (
       select key_slp1 from entry_keys
       where key_slp1 like $1 || '%'
         and dictionary_id not in (select id from dictionaries where kind = 'concordance')
       order by key_slp1 limit 600)
     select key_slp1, count(*) from c group by key_slp1
     order by length(key_slp1), key_slp1 limit $2",
  )
  .bind(k)
  .bind(limit)
  .fetch_all(&state.db)
  .await?;
  Ok(Json(
    rows
      .into_iter()
      .map(|(slp1, count)| Suggestion {
        headword: slp1_to_deva(&slp1),
        slp1,
        count,
      })
      .collect(),
  ))
}
