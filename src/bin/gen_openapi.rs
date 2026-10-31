//! Export OpenAPI JSON for frontend codegen (`frontend/openapi/schema.json`).
//!
//! Run: `cargo run --bin gen-openapi`

use std::fs;
use std::path::Path;

fn main() {
  let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("frontend/openapi/schema.json");
  if let Some(parent) = out.parent() {
    fs::create_dir_all(parent).expect("create openapi output directory");
  }

  let doc = sans_dict::openapi::openapi();
  let json = doc.to_pretty_json().expect("serialize OpenAPI document");
  fs::write(&out, json).expect("write OpenAPI schema");
  eprintln!("wrote {}", out.display());
}
