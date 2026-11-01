//! Better Auth-backed caller identity (agent API only; dictionary reads are public).
//!
//! Same pattern as `tsc-links` (`src/auth.rs` there): forward the browser's
//! `Cookie` header to `${BETTER_AUTH_URL}/api/auth/get-session` (the tsc-users
//! auth server) and read the session it returns. The cookie reaches us because
//! the app is served on a subdomain of the auth cookie domain
//! (`crossSubdomainCookies`), or on `localhost` in dev.
//!
//! [`require_admin`] is a route layer: it rejects unauthenticated (401) and
//! non-admin (403) callers, blocks cross-site writes, and inserts [`AuthUser`]
//! into request extensions so handlers just take `auth: AuthUser`.

use crate::routes::ApiError;
use crate::state::AppState;
use axum::extract::{FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use std::ops::Deref;
use std::sync::Arc;

/// `user` part of the Better Auth `get-session` response (admin plugin adds `role`).
#[derive(Debug, Deserialize)]
struct SessionUser {
  id: String,
  email: Option<String>,
  role: Option<String>,
}

/// Better Auth `get-session` body; the endpoint returns JSON `null` when signed out.
#[derive(Debug, Deserialize)]
struct SessionResponse {
  user: SessionUser,
}

/// Authenticated admin (one allocation per request, cheap `Arc` clone).
#[derive(Clone, Debug)]
pub struct AuthUserInner {
  /// Better Auth user id.
  pub user_id: String,
  pub email: Option<String>,
}

/// Extractor for the authenticated admin, inserted by [`require_admin`].
#[derive(Clone, Debug)]
pub struct AuthUser(pub Arc<AuthUserInner>);

impl Deref for AuthUser {
  type Target = AuthUserInner;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<S> FromRequestParts<S> for AuthUser
where
  S: Send + Sync,
{
  type Rejection = Response;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    // Absence means the route is not wrapped in `require_admin` (mis-wired).
    parts
      .extensions
      .get::<AuthUser>()
      .cloned()
      .ok_or_else(|| ApiError(StatusCode::UNAUTHORIZED, "not authenticated".into()).into_response())
  }
}

/// Better Auth's admin plugin stores roles as a comma-separated string.
fn has_admin_role(role: Option<&str>) -> bool {
  role.is_some_and(|r| r.split(',').any(|r| r.trim() == "admin"))
}

/// Cookie auth means the browser attaches credentials to cross-site requests
/// too (auth cookies are `SameSite=None`). Writes must come from our own page:
/// browsers send `Sec-Fetch-Site` on every fetch, so anything other than
/// `same-origin` on a mutating method is refused.
fn is_cross_site_write(method: &Method, headers: &HeaderMap) -> bool {
  if matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS) {
    return false;
  }
  headers
    .get("sec-fetch-site")
    .and_then(|v| v.to_str().ok())
    .is_some_and(|site| site != "same-origin" && site != "none")
}

/// Look up the Better Auth session for a raw `Cookie` header.
async fn get_session_from_cookie(
  state: &AppState,
  cookie: &str,
) -> Result<Option<SessionUser>, reqwest::Error> {
  let res = state
    .http
    .get(format!("{}/api/auth/get-session", state.better_auth_url))
    .header(header::COOKIE, cookie)
    .send()
    .await?
    .error_for_status()?;
  let body = res.json::<Option<SessionResponse>>().await?;
  Ok(body.map(|s| s.user))
}

/// Route layer: admin-only access via the Better Auth session cookie.
pub async fn require_admin(
  State(state): State<AppState>,
  mut req: Request,
  next: Next,
) -> Response {
  if is_cross_site_write(req.method(), req.headers()) {
    return ApiError(StatusCode::FORBIDDEN, "cross-site request refused".into()).into_response();
  }
  let Some(cookie) = req
    .headers()
    .get(header::COOKIE)
    .and_then(|v| v.to_str().ok())
    .filter(|v| !v.is_empty())
  else {
    return ApiError(StatusCode::UNAUTHORIZED, "not authenticated".into()).into_response();
  };

  let user = match get_session_from_cookie(&state, cookie).await {
    Ok(Some(user)) => user,
    Ok(None) => {
      return ApiError(StatusCode::UNAUTHORIZED, "not authenticated".into()).into_response();
    }
    Err(e) => {
      eprintln!("better auth get-session failed: {e}");
      return ApiError(StatusCode::BAD_GATEWAY, "auth service unavailable".into()).into_response();
    }
  };
  if !has_admin_role(user.role.as_deref()) {
    return ApiError(StatusCode::FORBIDDEN, "admin access required".into()).into_response();
  }

  req
    .extensions_mut()
    .insert(AuthUser(Arc::new(AuthUserInner {
      user_id: user.id,
      email: user.email,
    })));
  next.run(req).await
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn admin_role_parsing() {
    assert!(has_admin_role(Some("admin")));
    assert!(has_admin_role(Some("user, admin")));
    assert!(!has_admin_role(Some("user")));
    assert!(!has_admin_role(Some("administrator")));
    assert!(!has_admin_role(None));
  }

  #[test]
  fn cross_site_writes_are_refused() {
    let mut headers = HeaderMap::new();
    assert!(!is_cross_site_write(&Method::POST, &headers));
    headers.insert("sec-fetch-site", "same-origin".parse().unwrap());
    assert!(!is_cross_site_write(&Method::POST, &headers));
    headers.insert("sec-fetch-site", "cross-site".parse().unwrap());
    assert!(is_cross_site_write(&Method::DELETE, &headers));
    assert!(!is_cross_site_write(&Method::GET, &headers));
    headers.insert("sec-fetch-site", "same-site".parse().unwrap());
    assert!(is_cross_site_write(&Method::PATCH, &headers));
  }
}
