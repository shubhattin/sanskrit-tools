use std::env;
use tokio::net::TcpListener;

type DynError = Box<dyn std::error::Error + Send + Sync>;

/// Local dev port (the Vite dev server proxies `/api/*` here).
const PORT: u16 = 5780;

#[tokio::main]
async fn main() -> Result<(), DynError> {
  let state = sans_dict::app::init_state().await?;
  let app = sans_dict::app::router(state);

  let host = env::var("LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1".into());
  let port = env::var("PORT").unwrap_or_else(|_| PORT.to_string());
  let listener = TcpListener::bind(format!("{host}:{port}")).await?;
  eprintln!("listening on http://{}", listener.local_addr()?);
  axum::serve(listener, app).await?;
  Ok(())
}
