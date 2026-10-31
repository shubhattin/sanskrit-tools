//! OpenAPI document composition (paths collected via [`utoipa_axum::OpenApiRouter`] in route modules).

use utoipa::OpenApi;

use crate::entities::enums::{DictKind, Lang};
use crate::routes::browse::{BrowseItem, BrowseResponse, EntryResponse};
use crate::routes::dhatus::{Dhatu, DhatuDetail};
use crate::routes::paradigms::{Paradigm, ParadigmSummary};
use crate::routes::search::{DictionaryHits, EntryHit, FormAnalysis, SearchResponse, Suggestion};
use crate::routes::sutras::{Sutra, SutraDetail};
use crate::routes::{DictEntries, Dictionary, Entry, ErrorBody};

/// Root OpenAPI document; operation paths are registered by nested [`utoipa_axum::OpenApiRouter`]s.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Sanskrit Dictionary API",
        version = "0.1.0",
        description = "Read-only dictionary API: lexicons, thesauri, dhatu works, sutra commentaries, paradigms."
    ),
    components(schemas(
      ErrorBody,
      DictKind, Lang,
      Dictionary, Entry, DictEntries, EntryResponse, BrowseItem, BrowseResponse,
      EntryHit, DictionaryHits, FormAnalysis, SearchResponse, Suggestion,
      Sutra, SutraDetail,
      Dhatu, DhatuDetail,
      ParadigmSummary, Paradigm,
    )),
    tags(
        (name = "dictionaries", description = "Dictionaries, entries, book-order browsing"),
        (name = "search", description = "Sanskrit / English search and autocomplete"),
        (name = "sutras", description = "Astadhyayi sutras and commentaries"),
        (name = "dhatus", description = "Dhatupatha roots and dhatu works"),
        (name = "paradigms", description = "Inflection tables")
    )
)]
pub struct ApiDoc;

/// Build the merged OpenAPI spec (used by `gen-openapi`).
pub fn openapi() -> utoipa::openapi::OpenApi {
  let (_, api) = crate::app::openapi_router().split_for_parts();
  api
}
