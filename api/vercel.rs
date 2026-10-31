//! Vercel Rust function: serves `/api/*` (see the root `vercel.json`).
//! The frontend is static (`frontend/.output/public`) and served by Vercel.

use tower::ServiceBuilder;
use vercel_runtime::Error;
use vercel_runtime::axum::VercelLayer;

#[tokio::main]
async fn main() -> Result<(), Error> {
  let state = sans_dict::app::init_state()
    .await
    .map_err(|e| -> Error { Box::new(std::io::Error::other(e)) })?;
  let router = sans_dict::app::router(state);

  let app = ServiceBuilder::new()
    .layer(VercelLayer::new())
    .service(router);
  vercel_runtime::run(app).await
}
