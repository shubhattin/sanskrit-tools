//! `/api/vidyut/*`: the web face of [`crate::vidyut`] (Ambuda's vidyut grammar tools).
//!
//! Handlers only parse the request, call the core, and join in what the site knows: the
//! Dhātupāṭha root's page on this site, and which sandhi split parts are dictionary words.
//! Pure computation, no auth (public like the rest of the read API).

use super::{ApiError, ApiResult, stmt};
use crate::entities::dhatus;
use crate::state::AppState;
use crate::translit::deva_key;
use crate::vidyut::chandas::{self, MeterInfo, Scansion};
use crate::vidyut::prakriya::{
  self, Conjugation, Declension, Derivation, DhatuArgs, KrtGroup, LingaId, PrayogaId, RootInfo,
  SamasaKind, SamasaPada, SamasaResult, SiteRoot, StemArgs, TaddhitaForm, WordSpec,
};
use crate::vidyut::sandhi::{self, SandhiRule};
use crate::vidyut::{InputError, Result as VResult};
use axum::Json;
use axum::extract::{Path, Query, State};
use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QueryFilter};
use serde::{Deserialize, Serialize};
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
  /// The part is a headword or a known inflected form in the dictionaries.
  left_known: bool,
  right_known: bool,
}

#[derive(Serialize, ToSchema)]
pub struct ChunkSplit {
  chunk: String,
  /// Both parts known first, then one part known, then the rest (each in split order).
  splits: Vec<Split>,
}

#[derive(FromQueryResult)]
struct Known {
  k: String,
}

#[utoipa::path(
    post,
    path = "/api/vidyut/sandhi/split",
    operation_id = "vidyut.sandhiSplit",
    tag = "vidyut",
    request_body = TextRequest,
    responses(
        (status = 200, description = "Two-word split candidates per chunk", body = Vec<ChunkSplit>),
        (status = 400, description = "Empty / not Devanagari / too long", body = super::ErrorBody),
        (status = 500, description = "Database error", body = super::ErrorBody),
    )
)]
pub async fn sandhi_split(
  State(state): State<AppState>,
  Json(r): Json<TextRequest>,
) -> ApiResult<Vec<ChunkSplit>> {
  check_len(&r.text)?;
  let chunks = sandhi::split(&r.text, 120)?;
  let keys: Vec<String> = chunks
    .iter()
    .flat_map(|c| c.splits.iter())
    .flat_map(|s| [deva_key(&s.left), deva_key(&s.right)])
    .collect::<HashSet<_>>()
    .into_iter()
    .collect();
  let known: HashSet<String> = Known::find_by_statement(stmt(
    "select k from unnest($1::text[]) as k \
     where exists (select 1 from entry_keys e where e.key_slp1 = k) \
        or exists (select 1 from paradigm_forms f where f.form_slp1 = k)",
    vec![keys.into()],
  ))
  .all(&state.db)
  .await?
  .into_iter()
  .map(|r| r.k)
  .collect();
  Ok(Json(
    chunks
      .into_iter()
      .map(|c| {
        let mut splits: Vec<Split> = c
          .splits
          .into_iter()
          .map(|s| Split {
            left_known: known.contains(&deva_key(&s.left)),
            right_known: known.contains(&deva_key(&s.right)),
            left: s.left,
            right: s.right,
          })
          .collect();
        splits.sort_by_key(|s| std::cmp::Reverse(s.left_known as u8 + s.right_known as u8));
        ChunkSplit {
          chunk: c.chunk,
          splits,
        }
      })
      .collect(),
  ))
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
