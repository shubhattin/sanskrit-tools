//! Sanskrit grammar tools built on Ambuda's vidyut (<https://github.com/ambuda-org/vidyut>).
//!
//! - [`prakriya`]: Pāṇinian word generation (tiṅanta, subanta, kṛdanta, taddhita, strī, samāsa)
//!   with step-by-step derivations.
//! - [`chandas`]: meter identification and the meter catalogue.
//! - [`sandhi`]: joining words and listing phonetic split candidates.
//!
//! This module is the core: plain functions over vidyut, no HTTP or database. `routes::vidyut`
//! exposes it to the web, and the agent calls it as tools.
//!
//! Everything public speaks **Devanagari** (input and output). vidyut itself works in SLP1, so
//! text is converted at the edges with vidyut-lipi ([`to_slp1`] / [`to_deva`]).
//!
//! Data (`data/`, vidyut's py-0.4.0 release) is compiled into the binary, so the tools need no
//! files at runtime (Vercel functions ship only the binary).

pub mod chandas;
pub mod kosha;
pub mod prakriya;
pub mod sandhi;

use crate::translit::{convert, nfc};
use vidyut_lipi::Scheme;

/// An input the tools cannot use. The message is shown to the user as is.
#[derive(Debug)]
pub struct InputError(pub String);

impl std::fmt::Display for InputError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

impl std::error::Error for InputError {}

pub type Result<T> = std::result::Result<T, InputError>;

pub(crate) fn err<T>(msg: impl Into<String>) -> Result<T> {
  Err(InputError(msg.into()))
}

/// SLP1 → Devanagari (keeps svaras, it-markers' anunāsika etc.).
pub fn to_deva(slp1: &str) -> String {
  convert(slp1, Scheme::Slp1, Scheme::Devanagari)
}

/// Devanagari → SLP1. Rejects Latin letters: the tools take Devanagari only (Lipi Lekhika typing
/// in the UI covers romanized typing), and a stray ASCII word would silently be read as SLP1.
pub fn to_slp1(deva: &str) -> Result<String> {
  let s = nfc(deva.trim());
  if s.chars().any(|c| c.is_ascii_alphabetic()) {
    return err("Please enter the text in Devanagari");
  }
  Ok(convert(&s, Scheme::Devanagari, Scheme::Slp1))
}

/// One Devanagari word → SLP1 letters only (drops spaces, danda, digits, ZWJ…). Errors when empty.
pub fn word_slp1(deva: &str, what: &str) -> Result<String> {
  let s: String = to_slp1(deva)?
    .chars()
    .filter(|c| c.is_ascii_alphabetic())
    .collect();
  if s.is_empty() {
    return err(format!("Enter a {what}"));
  }
  Ok(s)
}

/// Dedups while keeping first-seen order (vidyut returns optional variants as separate prakriyās,
/// sometimes with the same final text).
pub(crate) fn dedup(items: impl IntoIterator<Item = String>) -> Vec<String> {
  let mut out: Vec<String> = Vec::new();
  for s in items {
    if !out.contains(&s) {
      out.push(s);
    }
  }
  out
}
