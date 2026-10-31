//! Data importer: `dict_data/` (StarDict + TSV) → Postgres.
//!
//! Each work is replaced atomically in its own transaction (delete + COPY),
//! and skipped when the sha256 of its source files (plus [`IMPORTER_VERSION`])
//! is unchanged, unless `--force`.

pub mod markup;
pub mod paradigms;
pub mod sources;
pub mod stardict;

use crate::translit::{deva_key, loose_key, nfc};
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use sources::{KeyKind, Rec, Source};
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Instant;

/// Bump when parsing/normalization changes so every work is re-imported.
pub const IMPORTER_VERSION: &str = "9";

// ------------------------------------------------------------ COPY helpers

fn copy_field(out: &mut String, v: Option<&str>) {
  match v {
    None => out.push_str("\\N"),
    Some(s) => {
      for c in s.chars() {
        match c {
          '\\' => out.push_str("\\\\"),
          '\n' => out.push_str("\\n"),
          '\r' => out.push_str("\\r"),
          '\t' => out.push_str("\\t"),
          c => out.push(c),
        }
      }
    }
  }
}

fn pg_text_array(items: &[String]) -> String {
  let mut s = String::from("{");
  for (i, it) in items.iter().enumerate() {
    if i > 0 {
      s.push(',');
    }
    s.push('"');
    for c in it.chars() {
      match c {
        '\\' => s.push_str("\\\\"),
        '"' => s.push_str("\\\""),
        c => s.push(c),
      }
    }
    s.push('"');
  }
  s.push('}');
  s
}

struct Row(String);
impl Row {
  fn new() -> Self {
    Row(String::new())
  }
  fn f(&mut self, v: Option<&str>) -> &mut Self {
    if !self.0.is_empty() && !self.0.ends_with('\n') {
      self.0.push('\t');
    }
    copy_field(&mut self.0, v);
    self
  }
  fn end(&mut self) {
    self.0.push('\n');
  }
}

async fn copy_in(tx: &mut Transaction<'_, Postgres>, stmt: &str, data: String) -> Result<()> {
  let mut copy = tx
    .copy_in_raw(stmt)
    .await
    .with_context(|| stmt.to_string())?;
  // Send in chunks to keep individual messages reasonable.
  let bytes = data.into_bytes();
  for chunk in bytes.chunks(4 << 20) {
    copy.send(chunk).await?;
  }
  copy.finish().await?;
  Ok(())
}

fn hash_files(dir: &Path, files: &[&str]) -> Result<String> {
  let mut h = Sha256::new();
  h.update(IMPORTER_VERSION.as_bytes());
  for f in files {
    let p = dir.join(f);
    h.update(f.as_bytes());
    h.update(std::fs::read(&p).with_context(|| format!("reading {p:?}"))?);
  }
  Ok(format!("{:x}", h.finalize()))
}

// ------------------------------------------------------------ reference tables

pub async fn import_sutras(pool: &PgPool, dir: &Path) -> Result<HashMap<String, String>> {
  let sutras = sources::parse_sutras(dir)?;
  let mut tx = pool.begin().await?;
  sqlx::query("update entries set sutra_id = null where sutra_id is not null")
    .execute(&mut *tx)
    .await?;
  sqlx::query("delete from sutras").execute(&mut *tx).await?;
  let mut data = String::new();
  for (id, text) in &sutras {
    let parts: Vec<i32> = id.split('.').map(|p| p.parse().unwrap_or(0)).collect();
    let mut r = Row::new();
    r.f(Some(id))
      .f(Some(&parts[0].to_string()))
      .f(Some(&parts[1].to_string()))
      .f(Some(&parts[2].to_string()))
      .f(Some(text))
      .f(Some(&deva_key(text)));
    r.end();
    data.push_str(&r.0);
  }
  copy_in(
    &mut tx,
    "COPY sutras (id, adhyaya, pada, num, text, text_slp1) FROM STDIN",
    data,
  )
  .await?;
  tx.commit().await?;
  eprintln!("sutras: {}", sutras.len());
  Ok(sutras.into_iter().collect())
}

/// Returns dhatu ids in parse order.
pub async fn import_dhatus(pool: &PgPool, dir: &Path) -> Result<Vec<i32>> {
  let dhatus = sources::parse_dhatus(dir)?;
  let mut tx = pool.begin().await?;
  sqlx::query("update entries set dhatu_id = null where dhatu_id is not null")
    .execute(&mut *tx)
    .await?;
  sqlx::query("delete from dhatus").execute(&mut *tx).await?;
  let mut data = String::new();
  for (i, d) in dhatus.iter().enumerate() {
    let mut r = Row::new();
    r.f(Some(&d.number))
      .f(Some(&d.upadesha))
      .f(Some(&d.root))
      .f(Some(&d.root_slp1))
      .f(Some(&d.artha))
      .f(Some(&d.gana.to_string()))
      .f(d.pada.as_deref())
      .f(d.it.as_deref())
      .f(d.extra.as_deref())
      .f(Some(&i.to_string()));
    r.end();
    data.push_str(&r.0);
  }
  copy_in(
    &mut tx,
    "COPY dhatus (number, upadesha, root, root_slp1, artha, gana, pada, it, extra, seq) FROM STDIN",
    data,
  )
  .await?;
  let ids: Vec<(i32,)> = sqlx::query_as("select id from dhatus order by seq")
    .fetch_all(&mut *tx)
    .await?;
  tx.commit().await?;
  eprintln!("dhatus: {}", ids.len());
  Ok(ids.into_iter().map(|r| r.0).collect())
}

async fn copy_paradigms(
  tx: &mut Transaction<'_, Postgres>,
  source: &str,
  seq0: usize,
  items: &[paradigms::Paradigm],
  layouts: &mut HashMap<String, i32>,
) -> Result<usize> {
  // Declension / kṛdanta tables share a few layouts: store only their forms.
  let use_layouts = matches!(source, "heritage_decl" | "heritage_kridanta");
  let mut data = String::new();
  for (i, p) in items.iter().enumerate() {
    let root_key = p.root.as_deref().map(deva_key);
    let (tables, layout_id, forms) = match use_layouts
      .then(|| paradigms::split_layout(&p.tables))
      .flatten()
    {
      Some((layout, forms)) => {
        let text = layout.to_string();
        let key = format!("{:x}", Sha256::digest(text.as_bytes()))[..32].to_string();
        let id = match layouts.get(&key) {
          Some(id) => *id,
          None => {
            let (id,): (i32,) = sqlx::query_as(
              "insert into paradigm_layouts (key, layout) values ($1, $2::jsonb)
               on conflict (key) do update set key = excluded.key returning id",
            )
            .bind(&key)
            .bind(&text)
            .fetch_one(&mut **tx)
            .await?;
            layouts.insert(key, id);
            id
          }
        };
        (None, Some(id.to_string()), Some(forms))
      }
      None => (Some(p.tables.to_string()), None, None),
    };
    let mut r = Row::new();
    r.f(Some(source))
      .f(Some(&(seq0 + i).to_string()))
      .f(Some(&p.lemma))
      .f(Some(&deva_key(&p.lemma)))
      .f(p.root.as_deref())
      .f(root_key.as_deref())
      .f(p.label.as_deref())
      .f(tables.as_deref())
      .f(layout_id.as_deref())
      .f(forms.as_deref());
    r.end();
    data.push_str(&r.0);
  }
  copy_in(
    tx,
    "COPY paradigms (source, seq, lemma, lemma_slp1, root, root_slp1, label, tables, layout_id, forms) FROM STDIN",
    data,
  )
  .await?;
  let ids: Vec<(i32, i32)> = sqlx::query_as(
    "select seq, id from paradigms where source = $1 and seq >= $2 and seq < $3 order by seq",
  )
  .bind(source)
  .bind(seq0 as i32)
  .bind((seq0 + items.len()) as i32)
  .fetch_all(&mut **tx)
  .await?;
  let mut data = String::new();
  let mut n = 0usize;
  for (seq, id) in ids {
    let p = &items[seq as usize - seq0];
    for (form, cells) in &p.forms {
      let cells = format!(
        "{{{}}}",
        cells
          .iter()
          .map(|c| c.to_string())
          .collect::<Vec<_>>()
          .join(",")
      );
      let mut r = Row::new();
      r.f(Some(&id.to_string())).f(Some(form)).f(Some(&cells));
      r.end();
      data.push_str(&r.0);
      n += 1;
    }
  }
  copy_in(
    tx,
    "COPY paradigm_forms (paradigm_id, form_slp1, cells) FROM STDIN",
    data,
  )
  .await?;
  Ok(n)
}

/// Rebuild all inflection tables (Heritage declension / kṛdanta / conjugation
/// and JNU tiṅanta). One transaction per source.
pub async fn import_paradigms(pool: &PgPool, dir: &Path) -> Result<()> {
  let mut sources: Vec<(&str, Vec<&str>)> = Vec::new();
  for (src, base) in paradigms::HERITAGE_FILES {
    match sources.iter_mut().find(|(s, _)| s == src) {
      Some((_, v)) => v.push(base),
      None => sources.push((src, vec![base])),
    }
  }
  sources.push(("jnu_tinanta", vec![]));
  let mut layouts: HashMap<String, i32> = HashMap::new();
  for (source, files) in sources {
    let t0 = Instant::now();
    let mut tx = pool.begin().await?;
    sqlx::query("delete from paradigms where source = $1")
      .bind(source)
      .execute(&mut *tx)
      .await?;
    let (mut seq, mut nforms) = (0usize, 0usize);
    if source == "jnu_tinanta" {
      let items = paradigms::parse_jnu(dir)?;
      nforms += copy_paradigms(&mut tx, source, seq, &items, &mut layouts).await?;
      seq += items.len();
    } else {
      for base in files {
        let items = paradigms::parse_heritage_file(dir, base, source == "heritage_conj")?;
        nforms += copy_paradigms(&mut tx, source, seq, &items, &mut layouts).await?;
        seq += items.len();
      }
    }
    tx.commit().await?;
    eprintln!(
      "paradigms {source:<18} {seq:>7} tables {nforms:>8} forms in {:.1?}",
      t0.elapsed()
    );
  }
  Ok(())
}

// ------------------------------------------------------------ works

pub struct Ctx {
  pub sutra_text: HashMap<String, String>,
  pub dhatu_ids: Vec<i32>,
  /// root SLP1 → dhatu ids
  pub dhatu_by_root: HashMap<String, Vec<i32>>,
}

pub async fn load_ctx(pool: &PgPool) -> Result<Ctx> {
  let sutra_text: HashMap<String, String> =
    sqlx::query_as::<_, (String, String)>("select id, text from sutras")
      .fetch_all(pool)
      .await?
      .into_iter()
      .collect();
  let rows: Vec<(i32, String, String)> =
    sqlx::query_as("select id, root_slp1, upadesha from dhatus order by seq")
      .fetch_all(pool)
      .await?;
  let mut dhatu_by_root: HashMap<String, Vec<i32>> = HashMap::new();
  for (id, root, up) in &rows {
    for k in crate::translit::root_variants(root, &deva_key(up)) {
      let v = dhatu_by_root.entry(k).or_default();
      if !v.contains(id) {
        v.push(*id);
      }
    }
  }
  Ok(Ctx {
    sutra_text,
    dhatu_ids: rows.iter().map(|r| r.0).collect(),
    dhatu_by_root,
  })
}

fn rec_keys(rec: &Rec) -> Vec<(String, KeyKind)> {
  let mut out: Vec<(String, KeyKind)> = Vec::new();
  let mut seen = HashSet::new();
  if !rec.no_headword_key {
    let k = deva_key(&rec.headword);
    if seen.insert(k.clone()) && !k.is_empty() {
      out.push((k, KeyKind::Headword));
    }
  }
  for (k, kind) in &rec.keys {
    if !k.is_empty() && seen.insert(k.clone()) {
      out.push((k.clone(), *kind));
    }
  }
  out
}

pub async fn import_source(
  pool: &PgPool,
  dir: &Path,
  src: &Source,
  ctx: &Ctx,
  force: bool,
) -> Result<()> {
  let t0 = Instant::now();
  let hash = hash_files(dir, src.files)?;
  let existing: Option<(i32, Option<String>)> =
    sqlx::query_as("select id, source_hash from dictionaries where slug = $1")
      .bind(src.slug)
      .fetch_optional(pool)
      .await?;

  // Metadata is always refreshed (cheap) even when the data is unchanged.
  let id: (i32,) = sqlx::query_as(
    "insert into dictionaries (slug, name, short_name, kind, src_lang, tgt_lang, description, attribution, sort_order)
     values ($1, $2, $3, $4::dict_kind, $5::lang, $6::lang, $7, $8, $9)
     on conflict (slug) do update set name = excluded.name, short_name = excluded.short_name,
       kind = excluded.kind, src_lang = excluded.src_lang, tgt_lang = excluded.tgt_lang,
       description = excluded.description, attribution = excluded.attribution, sort_order = excluded.sort_order
     returning id",
  )
  .bind(src.slug)
  .bind(src.name)
  .bind(src.short_name)
  .bind(src.kind.as_str())
  .bind(src.src_lang)
  .bind(src.tgt_lang)
  .bind(src.description)
  .bind(src.attribution)
  .bind(src.sort_order)
  .fetch_one(pool)
  .await?;
  let dict_id = id.0;

  if !force && existing.as_ref().and_then(|e| e.1.as_deref()) == Some(hash.as_str()) {
    eprintln!("{:<22} unchanged, skipped", src.slug);
    return Ok(());
  }

  let mut recs = (src.parse)(dir).with_context(|| format!("parsing {}", src.slug))?;
  recs.retain(|r| !r.senses.is_empty() && !r.headword.trim().is_empty());

  let mut entry_data = String::new();
  let mut keys_per_seq: Vec<Vec<(String, KeyKind)>> = Vec::with_capacity(recs.len());
  let mut linked_dhatu = 0usize;
  for (seq, rec) in recs.iter_mut().enumerate() {
    if let Some(sid) = &rec.sutra_id {
      match ctx.sutra_text.get(sid) {
        // Commentaries are headed by the sūtra itself (Kāśikā reading).
        Some(t) if matches!(src.kind, sources::Kind::SutraCommentary) => rec.headword = t.clone(),
        Some(_) => {}
        None => rec.sutra_id = None,
      }
    }
    let dhatu_id = match (rec.dhatu_index, &rec.dhatu_root_key) {
      (Some(i), _) => ctx.dhatu_ids.get(i).copied(),
      (None, Some(k)) => match ctx.dhatu_by_root.get(k) {
        Some(ids) if ids.len() == 1 => Some(ids[0]),
        _ => None,
      },
      _ => None,
    };
    linked_dhatu += dhatu_id.is_some() as usize;
    rec.headword = nfc(rec.headword.trim());
    keys_per_seq.push(rec_keys(rec));

    let senses = pg_text_array(&rec.senses);
    let data = rec.data.as_ref().map(|d| d.to_string());
    let dhatu = dhatu_id.map(|d| d.to_string());
    let mut r = Row::new();
    r.f(Some(&dict_id.to_string()))
      .f(Some(&seq.to_string()))
      .f(Some(&rec.headword))
      .f(Some(&senses))
      .f(data.as_deref())
      .f(rec.sutra_id.as_deref())
      .f(dhatu.as_deref())
      .f(rec.gloss_en.as_deref().filter(|g| !g.is_empty()));
    r.end();
    entry_data.push_str(&r.0);
  }

  let mut tx = pool.begin().await?;
  sqlx::query("delete from entries where dictionary_id = $1")
    .bind(dict_id)
    .execute(&mut *tx)
    .await?;
  copy_in(
    &mut tx,
    "COPY entries (dictionary_id, seq, headword, senses, data, sutra_id, dhatu_id, gloss_en) FROM STDIN",
    entry_data,
  )
  .await?;
  let ids: Vec<(i32, i32)> =
    sqlx::query_as("select seq, id from entries where dictionary_id = $1 order by seq")
      .bind(dict_id)
      .fetch_all(&mut *tx)
      .await?;
  let mut key_data = String::new();
  let mut nkeys = 0usize;
  for (seq, id) in &ids {
    for (k, kind) in &keys_per_seq[*seq as usize] {
      let loose = loose_key(k);
      let mut r = Row::new();
      r.f(Some(&id.to_string()))
        .f(Some(&dict_id.to_string()))
        .f(Some(k))
        .f(Some(&loose))
        .f(Some(kind.as_str()));
      r.end();
      key_data.push_str(&r.0);
      nkeys += 1;
    }
  }
  copy_in(
    &mut tx,
    "COPY entry_keys (entry_id, dictionary_id, key_slp1, key_loose, kind) FROM STDIN",
    key_data,
  )
  .await?;
  sqlx::query(
    "update dictionaries set entry_count = $2, source_hash = $3, imported_at = now() where id = $1",
  )
  .bind(dict_id)
  .bind(ids.len() as i32)
  .bind(&hash)
  .execute(&mut *tx)
  .await?;
  tx.commit().await?;
  eprintln!(
    "{:<22} {:>7} entries {:>7} keys{} in {:.1?}",
    src.slug,
    ids.len(),
    nkeys,
    if linked_dhatu > 0 {
      format!(" ({linked_dhatu} linked to dhātus)")
    } else {
      String::new()
    },
    t0.elapsed()
  );
  Ok(())
}
