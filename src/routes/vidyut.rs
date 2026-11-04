//! `/api/vidyut/*`: the web face of [`crate::vidyut`] (Ambuda's vidyut grammar tools).
//!
//! Handlers only parse the request, call the core, and join in what the site knows: the
//! Dhātupāṭha root's page on this site, and which sandhi split parts are dictionary words.
//! Pure computation, no auth (public like the rest of the read API).

use super::{ApiError, ApiResult, stmt};
use crate::entities::dhatus;
use crate::jev::{self, JevError, JevPick, Question};
use crate::state::AppState;
use crate::translit::deva_key;
use crate::vidyut::chandas::{self, MeterInfo, Scansion};
use crate::vidyut::kosha;
use crate::vidyut::prakriya::{
  self, Conjugation, Declension, Derivation, DhatuArgs, KrtGroup, LingaId, PrayogaId, RootInfo,
  SamasaKind, SamasaPada, SamasaResult, SiteRoot, StemArgs, TaddhitaForm, WordSpec,
};
use crate::vidyut::sandhi::{self, Akshara, Part, Reading, SandhiRule};
use crate::vidyut::to_deva;
use crate::vidyut::{InputError, Result as VResult};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QueryFilter};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub fn openapi_router() -> OpenApiRouter<AppState> {
  OpenApiRouter::new()
    .routes(routes!(roots))
    .routes(routes!(root))
    .routes(routes!(site_root_match))
    .routes(routes!(tinanta))
    .routes(routes!(krdanta))
    .routes(routes!(subanta))
    .routes(routes!(stryanta))
    .routes(routes!(taddhita))
    .routes(routes!(samasa))
    .routes(routes!(derive))
    .routes(routes!(chandas_identify))
    .routes(routes!(meters))
    .routes(routes!(sandhi_join))
    .routes(routes!(sandhi_split))
    .routes(routes!(sandhi_split_at))
    .routes(routes!(jev_split))
    .routes(routes!(jev_meter))
    .routes(routes!(jev_join))
    .routes(routes!(jev_analyses))
    .routes(routes!(pada))
    .routes(routes!(sandhi_rules))
}

impl From<InputError> for ApiError {
  fn from(e: InputError) -> Self {
    ApiError::bad_request(e.0)
  }
}

fn ok<T>(r: VResult<T>) -> ApiResult<T> {
  Ok(Json(r?))
}

/// Longest text accepted (a few verses).
const MAX_TEXT: usize = 2_000;

fn check_len(s: &str) -> Result<(), ApiError> {
  if s.chars().count() > MAX_TEXT {
    return Err(ApiError::bad_request("Text is too long"));
  }
  Ok(())
}

// ---- prakriyā -------------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct RootsParams {
  q: Option<String>,
  gana: Option<u8>,
}

#[utoipa::path(
    get,
    path = "/api/vidyut/dhatus",
    operation_id = "vidyut.dhatus",
    tag = "vidyut",
    params(
        ("q" = Option<String>, Query, description = "Root in Devanagari (भू, एधँ), a meaning word, or a code (1.1, 01.0001)"),
        ("gana" = Option<u8>, Query, description = "Only this gaṇa (1-10)"),
    ),
    responses(
        (status = 200, description = "Matching roots of vidyut's Dhātupāṭha (max 60)", body = Vec<RootInfo>),
        (status = 400, description = "Not Devanagari", body = super::ErrorBody),
    )
)]
pub async fn roots(Query(p): Query<RootsParams>) -> ApiResult<Vec<RootInfo>> {
  ok(prakriya::search_roots(
    p.q.as_deref().unwrap_or(""),
    p.gana,
    60,
  ))
}

#[derive(Serialize, ToSchema)]
pub struct RootDetail {
  root: RootInfo,
  /// This root's page on the site (`/dhatu/{id}`), when the site's Dhātupāṭha has it.
  dhatu_id: Option<i32>,
}

#[utoipa::path(
    get,
    path = "/api/vidyut/dhatus/{code}",
    operation_id = "vidyut.dhatu",
    tag = "vidyut",
    params(("code" = String, Path, description = "Dhātupāṭha code (01.0001 or 1.1)")),
    responses(
        (status = 200, description = "One root", body = RootDetail),
        (status = 404, description = "Unknown code", body = super::ErrorBody),
    )
)]
pub async fn root(
  State(state): State<AppState>,
  Path(code): Path<String>,
) -> ApiResult<RootDetail> {
  let root = prakriya::root(&code).ok_or_else(|| ApiError::not_found("Unknown Dhātupāṭha code"))?;
  // The site's Dhātupāṭha is another edition: match by upadeśa / root within the gaṇa.
  let rows = dhatus::Entity::find()
    .filter(dhatus::Column::Gana.eq(root.gana as i16))
    .all(&state.db)
    .await?;
  let dhatu_id = prakriya::pick_site_root(
    &root.code,
    rows.iter().filter_map(|d| Some((d.id, site_root(d)?))),
  );
  Ok(Json(RootDetail { root, dhatu_id }))
}

fn site_root(d: &dhatus::Model) -> Option<SiteRoot<'_>> {
  let number = d.number.split_once('.')?.1.parse().ok()?;
  Some(SiteRoot {
    gana: u8::try_from(d.gana).ok()?,
    number,
    upadesha: &d.upadesha,
    root_slp1: &d.root_slp1,
  })
}

#[utoipa::path(
    get,
    path = "/api/vidyut/dhatus/site/{id}",
    operation_id = "vidyut.siteDhatu",
    tag = "vidyut",
    params(("id" = i32, Path, description = "Root id of the site's Dhātupāṭha (`/dhatu/{id}`)")),
    responses(
        (status = 200, description = "The same root in vidyut's Dhātupāṭha", body = RootInfo),
        (status = 404, description = "No such root, or vidyut has no matching root", body = super::ErrorBody),
    )
)]
pub async fn site_root_match(
  State(state): State<AppState>,
  Path(id): Path<i32>,
) -> ApiResult<RootInfo> {
  let d = dhatus::Entity::find_by_id(id)
    .one(&state.db)
    .await?
    .ok_or_else(|| ApiError::not_found("Dhātu not found"))?;
  site_root(&d)
    .and_then(|s| prakriya::match_site_root(&s))
    .map(Json)
    .ok_or_else(|| ApiError::not_found("vidyut has no matching root"))
}

#[derive(Deserialize, ToSchema)]
pub struct TinantaRequest {
  dhatu: DhatuArgs,
  prayoga: PrayogaId,
}

#[utoipa::path(
    post,
    path = "/api/vidyut/tinanta",
    operation_id = "vidyut.tinanta",
    tag = "vidyut",
    request_body = TinantaRequest,
    responses(
        (status = 200, description = "Every lakāra of the root in this prayoga, per pada", body = Conjugation),
        (status = 400, description = "Bad root / upasarga, or no forms", body = super::ErrorBody),
    )
)]
pub async fn tinanta(Json(r): Json<TinantaRequest>) -> ApiResult<Conjugation> {
  ok(prakriya::conjugate(&r.dhatu, r.prayoga))
}

#[derive(Deserialize, ToSchema)]
pub struct KrdantaRequest {
  dhatu: DhatuArgs,
}

#[utoipa::path(
    post,
    path = "/api/vidyut/krdanta",
    operation_id = "vidyut.krdanta",
    tag = "vidyut",
    request_body = KrdantaRequest,
    responses(
        (status = 200, description = "Kṛdanta stems of the root, grouped", body = Vec<KrtGroup>),
        (status = 400, description = "Bad root / upasarga", body = super::ErrorBody),
    )
)]
pub async fn krdanta(Json(r): Json<KrdantaRequest>) -> ApiResult<Vec<KrtGroup>> {
  ok(prakriya::krdantas(&r.dhatu))
}

#[derive(Deserialize, ToSchema)]
pub struct SubantaRequest {
  stem: StemArgs,
  linga: LingaId,
}

#[utoipa::path(
    post,
    path = "/api/vidyut/subanta",
    operation_id = "vidyut.subanta",
    tag = "vidyut",
    request_body = SubantaRequest,
    responses(
        (status = 200, description = "Declension table", body = Declension),
        (status = 400, description = "Bad stem", body = super::ErrorBody),
    )
)]
pub async fn subanta(Json(r): Json<SubantaRequest>) -> ApiResult<Declension> {
  ok(prakriya::decline(&r.stem, r.linga))
}

#[derive(Deserialize, ToSchema)]
pub struct StemRequest {
  stem: StemArgs,
}

#[utoipa::path(
    post,
    path = "/api/vidyut/stryanta",
    operation_id = "vidyut.stryanta",
    tag = "vidyut",
    request_body = StemRequest,
    responses(
        (status = 200, description = "Feminine stems", body = Vec<String>),
        (status = 400, description = "Bad stem", body = super::ErrorBody),
    )
)]
pub async fn stryanta(Json(r): Json<StemRequest>) -> ApiResult<Vec<String>> {
  ok(prakriya::stryantas(&r.stem))
}

#[derive(Deserialize, ToSchema)]
pub struct TaddhitaRequest {
  /// Devanagari stem.
  stem: String,
}

#[utoipa::path(
    post,
    path = "/api/vidyut/taddhita",
    operation_id = "vidyut.taddhita",
    tag = "vidyut",
    request_body = TaddhitaRequest,
    responses(
        (status = 200, description = "Taddhitānta stems (common suffixes first)", body = Vec<TaddhitaForm>),
        (status = 400, description = "Bad stem", body = super::ErrorBody),
    )
)]
pub async fn taddhita(Json(r): Json<TaddhitaRequest>) -> ApiResult<Vec<TaddhitaForm>> {
  ok(prakriya::taddhitas(&r.stem))
}

#[derive(Deserialize, ToSchema)]
pub struct SamasaRequest {
  padas: Vec<SamasaPada>,
  samasa_type: SamasaKind,
}

#[utoipa::path(
    post,
    path = "/api/vidyut/samasa",
    operation_id = "vidyut.samasa",
    tag = "vidyut",
    request_body = SamasaRequest,
    responses(
        (status = 200, description = "Compound stem and its nominative", body = SamasaResult),
        (status = 400, description = "Bad members or no result", body = super::ErrorBody),
    )
)]
pub async fn samasa(Json(r): Json<SamasaRequest>) -> ApiResult<SamasaResult> {
  if r.padas.len() > 6 {
    return Err(ApiError::bad_request("At most six members"));
  }
  ok(prakriya::compound(&r.padas, r.samasa_type))
}

#[utoipa::path(
    post,
    path = "/api/vidyut/derive",
    operation_id = "vidyut.derive",
    tag = "vidyut",
    request_body = WordSpec,
    responses(
        (status = 200, description = "Step-by-step derivations (one per result)", body = Vec<Derivation>),
        (status = 400, description = "Bad arguments", body = super::ErrorBody),
    )
)]
pub async fn derive(Json(spec): Json<WordSpec>) -> ApiResult<Vec<Derivation>> {
  if let WordSpec::Vakya { words } = &spec
    && (words.len() > 12 || words.iter().any(|w| w.chars().count() > 60))
  {
    return Err(ApiError::bad_request("Too many words"));
  }
  ok(prakriya::derive(&spec))
}

// ---- chandas --------------------------------------------------------------------------------

#[derive(Deserialize, ToSchema)]
pub struct TextRequest {
  /// Devanagari text.
  pub text: String,
}

#[utoipa::path(
    post,
    path = "/api/vidyut/chandas",
    operation_id = "vidyut.chandas",
    tag = "vidyut",
    request_body = TextRequest,
    responses(
        (status = 200, description = "Scansion and meter", body = Scansion),
        (status = 400, description = "Empty / not Devanagari / too long", body = super::ErrorBody),
    )
)]
pub async fn chandas_identify(Json(r): Json<TextRequest>) -> ApiResult<Scansion> {
  check_len(&r.text)?;
  ok(chandas::identify(&r.text))
}

#[utoipa::path(
    get,
    path = "/api/vidyut/meters",
    operation_id = "vidyut.meters",
    tag = "vidyut",
    responses((status = 200, description = "Meter catalogue", body = Vec<MeterInfo>))
)]
pub async fn meters() -> Json<Vec<MeterInfo>> {
  Json(chandas::meters())
}

// ---- sandhi ---------------------------------------------------------------------------------

#[derive(Deserialize, ToSchema)]
pub struct JoinRequest {
  /// Devanagari words, in order.
  words: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/api/vidyut/sandhi/join",
    operation_id = "vidyut.sandhiJoin",
    tag = "vidyut",
    request_body = JoinRequest,
    responses(
        (status = 200, description = "The joined phrase (variants when sandhi is optional)", body = Vec<String>),
        (status = 400, description = "Fewer than two words / not Devanagari", body = super::ErrorBody),
    )
)]
pub async fn sandhi_join(Json(r): Json<JoinRequest>) -> ApiResult<Vec<String>> {
  if r.words.len() > 12 || r.words.iter().any(|w| w.chars().count() > 60) {
    return Err(ApiError::bad_request("Too many words"));
  }
  ok(prakriya::join_words(&r.words))
}

#[derive(Serialize, ToSchema)]
pub struct Split {
  left: String,
  right: String,
  /// The part is a word: in the dictionaries or generated by the grammar.
  left_known: bool,
  right_known: bool,
}

#[derive(Serialize, ToSchema)]
pub struct ChunkSplit {
  pub chunk: String,
  /// The chunk as akṣaras of varṇas (`at` positions for `/sandhi/split-at`).
  aksharas: Vec<Akshara>,
  /// The best whole readings (all words), best first: dictionary words, then fewest words.
  pub readings: Vec<Reading>,
  /// Two-word candidates: both parts known first, then one, then the rest (each in split order).
  splits: Vec<Split>,
}

#[derive(FromQueryResult)]
struct Known {
  k: String,
}

/// Which of these keys (SLP1 letters) are headwords or inflected forms in the dictionaries.
async fn known_keys(state: &AppState, keys: HashSet<String>) -> Result<HashSet<String>, ApiError> {
  let keys: Vec<String> = keys.into_iter().filter(|k| !k.is_empty()).collect();
  if keys.is_empty() {
    return Ok(HashSet::new());
  }
  Ok(
    Known::find_by_statement(stmt(
      "select k from unnest($1::text[]) as k \
       where exists (select 1 from entry_keys e where e.key_slp1 = k) \
          or exists (select 1 from paradigm_forms f where f.form_slp1 = k)",
      vec![keys.into()],
    ))
    .all(&state.db)
    .await?
    .into_iter()
    .map(|r| r.k)
    .collect(),
  )
}

const READINGS: usize = 5;
/// Words of a reading Jev tags at once (a long verse).
const MAX_WORDS: usize = 64;

/// vidyut's kosha + cheda, decompressed on first use (off the async workers).
async fn kosha_ready() -> Result<(), ApiError> {
  tokio::task::spawn_blocking(kosha::load)
    .await
    .map_err(|e| ApiError::internal(format!("kosha: {e}")))
}

async fn split_chunks(state: &AppState, text: &str) -> Result<Vec<ChunkSplit>, ApiError> {
  check_len(text)?;
  let chunks = sandhi::chunks(text)?;
  let lattices: Vec<_> = chunks.iter().map(|c| sandhi::lattice(c)).collect();
  let pairs: Vec<_> = chunks.iter().map(|c| sandhi::two_word(c, 120)).collect();
  let mut keys: HashSet<String> = lattices.iter().flat_map(|l| l.words()).collect();
  keys.extend(
    pairs
      .iter()
      .flatten()
      .flat_map(|s| [deva_key(&s.left), deva_key(&s.right)]),
  );
  let (known, ready) = tokio::join!(known_keys(state, keys), kosha_ready());
  let (known, ()) = (known?, ready?);
  let is_known = |k: &str| known.contains(k);
  let is_word = |k: &str| known.contains(k) || kosha::contains(k);
  Ok(
    chunks
      .iter()
      .zip(lattices)
      .zip(pairs)
      .map(|((c, lat), pairs)| {
        let mut splits: Vec<Split> = pairs
          .into_iter()
          .map(|s| Split {
            left_known: is_word(&deva_key(&s.left)),
            right_known: is_word(&deva_key(&s.right)),
            left: s.left,
            right: s.right,
          })
          .collect();
        splits.sort_by_key(|s| std::cmp::Reverse(s.left_known as u8 + s.right_known as u8));
        ChunkSplit {
          chunk: to_deva(c),
          aksharas: sandhi::aksharas(c),
          readings: sandhi::with_cheda(
            sandhi::readings(&lat, &is_known, READINGS),
            c,
            &is_known,
            READINGS,
          ),
          splits,
        }
      })
      .collect(),
  )
}

#[utoipa::path(
    post,
    path = "/api/vidyut/sandhi/split",
    operation_id = "vidyut.sandhiSplit",
    tag = "vidyut",
    request_body = TextRequest,
    responses(
        (status = 200, description = "Per chunk: best whole readings, syllables and two-word candidates", body = Vec<ChunkSplit>),
        (status = 400, description = "Empty / not Devanagari / too long", body = super::ErrorBody),
        (status = 500, description = "Database error", body = super::ErrorBody),
    )
)]
pub async fn sandhi_split(
  State(state): State<AppState>,
  Json(r): Json<TextRequest>,
) -> ApiResult<Vec<ChunkSplit>> {
  Ok(Json(split_chunks(&state, &r.text).await?))
}

#[derive(Deserialize, ToSchema)]
pub struct SplitAtRequest {
  /// One chunk (a word without spaces), Devanagari.
  chunk: String,
  /// Varṇa position from `ChunkSplit.aksharas`: the words meet at this sound.
  at: usize,
}

#[derive(Serialize, ToSchema)]
pub struct PinnedSplit {
  left: Part,
  /// The rest, read as words by the optimizer.
  right: Reading,
}

#[utoipa::path(
    post,
    path = "/api/vidyut/sandhi/split-at",
    operation_id = "vidyut.sandhiSplitAt",
    tag = "vidyut",
    request_body = SplitAtRequest,
    responses(
        (status = 200, description = "Splits with the junction at that sound, dictionary words first", body = Vec<PinnedSplit>),
        (status = 400, description = "Not one Devanagari chunk", body = super::ErrorBody),
        (status = 500, description = "Database error", body = super::ErrorBody),
    )
)]
pub async fn sandhi_split_at(
  State(state): State<AppState>,
  Json(r): Json<SplitAtRequest>,
) -> ApiResult<Vec<PinnedSplit>> {
  check_len(&r.chunk)?;
  let chunks = sandhi::chunks(&r.chunk)?;
  let [chunk] = chunks.as_slice() else {
    return Err(ApiError::bad_request("Give one word (no spaces)"));
  };
  let edges = sandhi::splits_at(chunk, r.at);
  let lattices: Vec<_> = edges
    .iter()
    .map(|(_, rest)| sandhi::lattice(rest))
    .collect();
  let mut keys: HashSet<String> = lattices.iter().flat_map(|l| l.words()).collect();
  keys.extend(edges.iter().map(|(f, _)| sandhi::word_key(f)));
  let (known, ready) = tokio::join!(known_keys(&state, keys), kosha_ready());
  let (known, ()) = (known?, ready?);
  let is_known = |k: &str| known.contains(k);
  let mut out: Vec<PinnedSplit> = edges
    .iter()
    .zip(&lattices)
    .filter_map(|((first, _), lat)| {
      Some(PinnedSplit {
        left: Part::new(first, &is_known),
        right: sandhi::readings(lat, &is_known, 1).into_iter().next()?,
      })
    })
    .collect();
  let score = |p: &PinnedSplit| {
    p.left.known as usize + p.right.parts.iter().filter(|x| x.known).count()
      - p.right.parts.iter().filter(|x| !x.known).count().min(1)
  };
  out.sort_by_key(|p| std::cmp::Reverse(score(p)));
  Ok(Json(out))
}

/// A word for Jev: `उद्भवनम् [grammar]: उद्भवन (उद्भू + ल्युट्) napumsaka prathama eka | …`.
fn describe_part(p: &Part) -> String {
  let source = if p.dictionary {
    "dictionary"
  } else if p.known {
    "grammar"
  } else {
    "unknown"
  };
  let analyses: Vec<&str> = p.analyses.iter().take(3).map(|a| a.key.as_str()).collect();
  if analyses.is_empty() {
    format!("{} [{source}]", p.text)
  } else {
    format!("{} [{source}]: {}", p.text, analyses.join(" | "))
  }
}

/// A reading for Jev: its words with what each can be.
fn describe_reading(r: &Reading) -> String {
  let words: Vec<String> = r.parts.iter().map(describe_part).collect();
  format!(
    "{}{} — {}",
    r.label,
    if r.cheda { " [cheda]" } else { "" },
    words.join("; ")
  )
}

#[derive(Deserialize, utoipa::IntoParams)]
pub struct WordQuery {
  /// One Devanagari word (a form as written; a joined phrase is split).
  w: String,
}

#[derive(Serialize, ToSchema)]
pub struct WordAnalysis {
  /// The word, as a part (known / dictionary / analyses).
  word: Part,
  /// When the word is not one form: how it splits (cheda's reading, else the optimizer's best).
  split: Option<Reading>,
}

#[utoipa::path(
    get,
    path = "/api/vidyut/pada",
    operation_id = "vidyut.pada",
    tag = "vidyut",
    params(WordQuery),
    responses(
        (status = 200, description = "What a word form can be (every analysis vidyut's grammar gives), or how it splits", body = WordAnalysis),
        (status = 400, description = "Empty / not Devanagari / more than one word", body = super::ErrorBody),
        (status = 500, description = "Database error", body = super::ErrorBody),
    )
)]
pub async fn pada(
  State(state): State<AppState>,
  Query(q): Query<WordQuery>,
) -> ApiResult<WordAnalysis> {
  check_len(&q.w)?;
  let chunks = sandhi::chunks(&q.w)?;
  let [chunk] = chunks.as_slice() else {
    return Err(ApiError::bad_request("Give one word (no spaces)"));
  };
  // As written, else with its pre-sandhi final (रामो → रामः).
  let forms: Vec<String> = std::iter::once(sandhi::pada_final(chunk))
    .chain(sandhi::final_variants(chunk))
    .collect();
  let lat = sandhi::lattice(chunk);
  let mut keys = lat.words();
  keys.extend(forms.iter().map(|f| sandhi::word_key(f)));
  let (known, ready) = tokio::join!(known_keys(&state, keys), kosha_ready());
  let (known, ()) = (known?, ready?);
  let is_known = |k: &str| known.contains(k);
  let parts: Vec<Part> = forms.iter().map(|f| Part::new(f, &is_known)).collect();
  let part = parts.iter().find(|p| p.known).unwrap_or(&parts[0]).clone();
  // A form of its own needs no split; a dictionary-only headword (तथैव) may still be a joined
  // phrase.
  let split = if !part.analyses.is_empty() {
    None
  } else {
    let readings = sandhi::with_cheda(sandhi::readings(&lat, &is_known, 3), chunk, &is_known, 3);
    readings
      .iter()
      .find(|r| r.cheda)
      .or(readings.first())
      .filter(|r| r.parts.len() > 1)
      .cloned()
  };
  Ok(Json(WordAnalysis { word: part, split }))
}

#[derive(Deserialize, ToSchema)]
pub struct AnalysesRequest {
  /// The whole text (context).
  text: String,
  /// The words of the chosen reading, in order (Devanagari, as `Part.text`).
  words: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct WordPick {
  word: String,
  /// Jev's choice among the word's analyses (options = `PadaInfo.key`); none with < 2.
  pick: Option<JevPick>,
}

#[utoipa::path(
    post,
    path = "/api/vidyut/jev/analyses",
    operation_id = "vidyut.jevAnalyses",
    tag = "vidyut",
    request_body = AnalysesRequest,
    responses(
        (status = 200, description = "Jev's pick of each word's analysis in the context of the sentence (up to 24 ambiguous words)", body = Vec<WordPick>),
        (status = 400, description = "Bad text", body = super::ErrorBody),
        (status = 502, description = "Jev failed", body = super::ErrorBody),
        (status = 503, description = "Jev not configured", body = super::ErrorBody),
    )
)]
pub async fn jev_analyses(
  State(state): State<AppState>,
  Json(r): Json<AnalysesRequest>,
) -> ApiResult<Vec<WordPick>> {
  check_len(&r.text)?;
  if r.words.is_empty() || r.words.len() > MAX_WORDS {
    return Err(ApiError::bad_request(
      "Give the reading's words (at most 64)",
    ));
  }
  kosha_ready().await?;
  let text_chunks = sandhi::chunks(&r.text)?;
  let words: Vec<(String, Vec<kosha::PadaInfo>)> = r
    .words
    .iter()
    .map(|w| {
      let slp = crate::vidyut::to_slp1(w)?;
      Ok((w.clone(), kosha::analyses(&sandhi::word_key(&slp))))
    })
    .collect::<VResult<_>>()?;
  // cheda's choice per word, where its segmentation has the word: over the whole text at once,
  // so its tags see the neighbouring words (chunk by chunk if that fails).
  let cheda: Vec<(String, String)> = kosha::segment(&text_chunks.join(" "))
    .unwrap_or_else(|| {
      text_chunks
        .iter()
        .filter_map(|c| kosha::segment(c))
        .flatten()
        .collect()
    })
    .into_iter()
    .filter_map(|w| Some((to_deva(&sandhi::pada_final(&w.text)), w.pada?.key)))
    .collect();
  let questions: Vec<Question> = words
    .iter()
    .enumerate()
    .filter(|(_, (_, a))| a.len() >= 2)
    .map(|(i, (w, a))| Question {
      name: format!("w{i}"),
      instructions: format!(
        "Which grammatical analysis does «{w}» (word {} of the reading) have in this text?",
        i + 1
      ),
      options: a.iter().map(|x| x.key.clone()).collect(),
    })
    .collect();
  let state_json = json!({
    "task": "Parse a Sanskrit sentence: pick each word's grammatical analysis.",
    "text": r.text.trim(),
    "reading": r.words.join(" "),
    "words": words.iter().map(|(w, a)| json!({
      "word": w,
      "can_be": a.iter().map(|x| &x.key).collect::<Vec<_>>(),
      "cheda": cheda.iter().find(|(cw, _)| cw == w).map(|(_, k)| k),
    })).collect::<Vec<_>>(),
    "notes": [
      "Analyses: stem (kṛdanta stems with their root and suffix), gender, case, number; verbs: root, lakāra (lat = present, lan = imperfect, lot = imperative, …), person, number, voice.",
      "Choose what fits the sentence: agreement of adjectives and nouns, the subject agreeing with the verb, the usual sense of common words.",
      "cheda, a statistical tagger, gives its choice for some words: a hint, not always right.",
    ],
  });
  let mut picks = jev::choose(&state.http, state_json, questions).await?;
  Ok(Json(
    words
      .into_iter()
      .enumerate()
      .map(|(i, (word, _))| WordPick {
        word,
        pick: picks.remove(&format!("w{i}")),
      })
      .collect(),
  ))
}

// ---- Jev picks --------------------------------------------------------------------------------

impl From<JevError> for ApiError {
  fn from(e: JevError) -> Self {
    match e {
      JevError::NotConfigured => ApiError(
        StatusCode::SERVICE_UNAVAILABLE,
        "Jev is not configured (OPENROUTER_API_KEY)".into(),
      ),
      JevError::Upstream(m) => {
        eprintln!("jev: {m}");
        ApiError(StatusCode::BAD_GATEWAY, "Jev did not answer".into())
      }
    }
  }
}

#[derive(Serialize, ToSchema)]
pub struct ChunkPick {
  chunk: String,
  /// Jev's choice among the chunk's `readings` (options = their labels); none with < 2 readings.
  pick: Option<JevPick>,
}

#[utoipa::path(
    post,
    path = "/api/vidyut/jev/split",
    operation_id = "vidyut.jevSplit",
    tag = "vidyut",
    request_body = TextRequest,
    responses(
        (status = 200, description = "Jev's pick among each chunk's best readings", body = Vec<ChunkPick>),
        (status = 400, description = "Bad text", body = super::ErrorBody),
        (status = 502, description = "Jev failed", body = super::ErrorBody),
        (status = 503, description = "Jev not configured", body = super::ErrorBody),
    )
)]
pub async fn jev_split(
  State(state): State<AppState>,
  Json(r): Json<TextRequest>,
) -> ApiResult<Vec<ChunkPick>> {
  let chunks = split_chunks(&state, &r.text).await?;
  let questions = chunks
    .iter()
    .enumerate()
    .map(|(i, c)| Question {
      name: format!("c{i}"),
      instructions: format!(
        "Which reading splits chunk {} of {}, «{}», into the words this text most likely intends?",
        i + 1,
        chunks.len(),
        c.chunk
      ),
      options: c.readings.iter().map(describe_reading).collect(),
    })
    .collect();
  let state_json = json!({
    "task": "Read Sanskrit text by undoing sandhi (separating the joined words).",
    "text": r.text.trim(),
    // The whole text's candidates, so every question (batches are asked in parallel) reads its
    // chunk with the others.
    "chunks": chunks.iter().map(|c| json!({
      "chunk": c.chunk,
      "readings": c.readings.iter().map(|r| &r.label).collect::<Vec<_>>(),
    })).collect::<Vec<_>>(),
    "notes": [
      "The chunks are one text (a sentence or a verse), in order: read them together. A chunk's reading should fit the others' (a subject and its verb, an adjective and its noun, a relative and its correlative are often in different chunks).",
      "Each option is a reading of one chunk: its words, and for each word what it can be (stem, case, number / root, tense, person) as Pāṇinian grammar generates it.",
      "Readings come from an optimizer that prefers known words, then fewer words; options are in its order, best first.",
      "[dictionary] = a dictionary headword or inflected form; [grammar] = a form the grammar generates but no dictionary lists; [unknown] = neither. Very short words (ओ, न, र) match almost anywhere, so readings built from them are suspect.",
      "Prefer readings whose words fit together as a sentence (agreement, one finite verb, common words) over rare forms.",
      "A chunk's last word may show its pre-sandhi form (रामो before a space → रामः).",
      "[cheda] marks the reading of cheda, a statistical segmenter trained on Sanskrit texts: a strong hint, not always right.",
    ],
  });
  let mut picks = jev::choose(&state.http, state_json, questions).await?;
  Ok(Json(
    chunks
      .into_iter()
      .enumerate()
      .map(|(i, c)| ChunkPick {
        pick: picks.remove(&format!("c{i}")).map(|mut p| {
          // Report the plain labels as the options.
          p.options = c.readings.iter().map(|r| r.label.clone()).collect();
          p.options.truncate(p.probabilities.len());
          p
        }),
        chunk: c.chunk,
      })
      .collect(),
  ))
}

#[utoipa::path(
    post,
    path = "/api/vidyut/jev/meter",
    operation_id = "vidyut.jevMeter",
    tag = "vidyut",
    request_body = TextRequest,
    responses(
        (status = 200, description = "Jev's pick among the matching meters (options = meter names); null with < 2", body = Option<JevPick>),
        (status = 400, description = "Bad verse", body = super::ErrorBody),
        (status = 502, description = "Jev failed", body = super::ErrorBody),
        (status = 503, description = "Jev not configured", body = super::ErrorBody),
    )
)]
pub async fn jev_meter(
  State(state): State<AppState>,
  Json(r): Json<TextRequest>,
) -> ApiResult<Option<JevPick>> {
  check_len(&r.text)?;
  let scan = chandas::identify(&r.text)?;
  let mut names: Vec<(String, &'static str)> = Vec::new();
  for m in scan.best.iter().chain(&scan.others) {
    let fit = match m.fit {
      chandas::Fit::Full => "the whole text fits",
      chandas::Fit::Pada => "whole pādas fit",
      chandas::Fit::Prefix => "only the start fits (the text is not a whole pāda of it)",
    };
    if !names.iter().any(|(n, _)| *n == m.name) {
      names.push((m.name.clone(), fit));
    }
  }
  names.truncate(jev::MAX_OPTIONS);
  if names.len() < 2 {
    return Ok(Json(None));
  }
  let catalogue = chandas::meters();
  let describe = |name: &str, fit: &str| {
    let Some(m) = catalogue.iter().find(|m| m.name == name) else {
      return format!("{name}: {fit}");
    };
    let shape = match m.kind {
      chandas::MeterKind::Anushtubh => "anuṣṭubh (śloka), the commonest meter of the epics and \
        kāvya: 8 syllables a pāda, only the 5th–7th fixed (5th laghu, 6th guru)"
        .to_string(),
      chandas::MeterKind::Jati => format!("jāti counted by mātrās per pāda {:?}", m.matras),
      chandas::MeterKind::Vrtta => format!(
        "vṛtta of {} syllables a pāda, pattern {}",
        m.syllables.first().copied().unwrap_or(0),
        m.padas
          .iter()
          .map(|p| p.pattern.clone())
          .collect::<Vec<_>>()
          .join(" / ")
      ),
    };
    format!("{name}: {shape}; {fit}")
  };
  let question = Question {
    name: "meter".into(),
    instructions: "Which meter is this verse composed in?".into(),
    options: names.iter().map(|(n, f)| describe(n, f)).collect(),
  };
  let state_json = json!({
    "verse": r.text.trim(),
    "scansion": scan.lines.iter()
      .map(|l| format!("{} ({} syllables)", l.pattern, l.syllables.len()))
      .collect::<Vec<_>>(),
    "note": "Scansion per line: G = guru (heavy), L = laghu (light); '.' in a meter pattern = either.",
  });
  let mut picks = jev::choose(&state.http, state_json, vec![question]).await?;
  Ok(Json(picks.remove("meter").map(|mut p| {
    // Report plain meter names as the options.
    p.options = names.into_iter().map(|(n, _)| n).collect();
    p
  })))
}

#[utoipa::path(
    post,
    path = "/api/vidyut/jev/join",
    operation_id = "vidyut.jevJoin",
    tag = "vidyut",
    request_body = JoinRequest,
    responses(
        (status = 200, description = "Jev's pick among the joined variants; null with < 2", body = Option<JevPick>),
        (status = 400, description = "Bad words", body = super::ErrorBody),
        (status = 502, description = "Jev failed", body = super::ErrorBody),
        (status = 503, description = "Jev not configured", body = super::ErrorBody),
    )
)]
pub async fn jev_join(
  State(state): State<AppState>,
  Json(r): Json<JoinRequest>,
) -> ApiResult<Option<JevPick>> {
  if r.words.len() > 12 || r.words.iter().any(|w| w.chars().count() > 60) {
    return Err(ApiError::bad_request("Too many words"));
  }
  let joined = prakriya::join_words(&r.words)?;
  if joined.len() < 2 {
    return Ok(Json(None));
  }
  let question = Question {
    name: "join".into(),
    instructions: "Which is the usual written form of these words joined by sandhi?".into(),
    options: joined,
  };
  let state_json = json!({ "task": "Sanskrit sandhi between words", "words": r.words });
  let mut picks = jev::choose(&state.http, state_json, vec![question]).await?;
  Ok(Json(picks.remove("join")))
}

#[utoipa::path(
    get,
    path = "/api/vidyut/sandhi/rules",
    operation_id = "vidyut.sandhiRules",
    tag = "vidyut",
    responses((status = 200, description = "The sandhi rule table", body = Vec<SandhiRule>))
)]
pub async fn sandhi_rules() -> Json<Vec<SandhiRule>> {
  Json(sandhi::rules())
}
