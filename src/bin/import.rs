//! `cargo run --release --bin import -- [--data dict_data] [--force] [--refs] [--forms] [slug ...]`
//!
//! Without slugs every work is imported. Reference tables (sūtras, dhātus,
//! inflection paradigms; `--forms` rebuilds only the paradigms) are rebuilt when empty or with `--refs`; rebuilding them forces
//! the sūtra commentaries and dhātu works to be re-imported (their ids change).

use anyhow::{Result, bail};
use sans_dict::import::{self, sources};
use sqlx::postgres::PgPoolOptions;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
  let _ = dotenvy::from_filename(".env.local");
  let _ = dotenvy::dotenv();
  let mut data = PathBuf::from("dict_data");
  let mut force = false;
  let mut refs = false;
  let mut forms = false;
  let mut slugs = Vec::new();
  let mut args = std::env::args().skip(1);
  while let Some(a) = args.next() {
    match a.as_str() {
      "--data" => data = PathBuf::from(args.next().expect("--data <dir>")),
      "--force" => force = true,
      "--refs" => refs = true,
      "--forms" => forms = true,
      s if s.starts_with('-') => bail!("unknown flag {s}"),
      s => slugs.push(s.to_string()),
    }
  }
  let url = std::env::var("PG_DATABASE_URL")?;
  let pool = PgPoolOptions::new()
    .max_connections(4)
    .connect(&url)
    .await?;

  let (nsutra, ndhatu, nforms): (i64, i64, i64) = sqlx::query_as(
    "select (select count(*) from sutras), (select count(*) from dhatus), (select count(*) from paradigms)",
  )
  .fetch_one(&pool)
  .await?;
  let mut rebuilt = false;
  if refs || nsutra == 0 {
    import::import_sutras(&pool, &data).await?;
    rebuilt = true;
  }
  if refs || ndhatu == 0 {
    import::import_dhatus(&pool, &data).await?;
    rebuilt = true;
  }
  if refs || forms || nforms == 0 {
    import::import_paradigms(&pool, &data).await?;
  }
  let ctx = import::load_ctx(&pool).await?;

  let all = sources::all();
  for s in &slugs {
    if !all.iter().any(|x| x.slug == s) {
      bail!(
        "unknown slug {s}; known: {}",
        all.iter().map(|x| x.slug).collect::<Vec<_>>().join(", ")
      );
    }
  }
  for src in all
    .iter()
    .filter(|s| slugs.is_empty() || slugs.iter().any(|x| x == s.slug))
  {
    let dependent = matches!(
      src.kind,
      sources::Kind::DhatuWork | sources::Kind::SutraCommentary
    );
    import::import_source(&pool, &data, src, &ctx, force || (rebuilt && dependent)).await?;
  }
  // Re-imports delete + re-insert rows; reclaim that space for reuse.
  sqlx::query("vacuum analyze").execute(&pool).await?;
  Ok(())
}
