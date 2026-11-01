//! Tiny shared helpers for the examples in `examples/`.
//!
//! All vidyut crates speak SLP1 internally. For terminal readability, examples
//! take IAST input and print IAST (and sometimes Devanagari) via vidyut-lipi.

use std::path::PathBuf;
use vidyut_kosha::entries::{PadaEntry, PratipadikaEntry};
use vidyut_lipi::{Mapping, Scheme, transliterate};
use vidyut_prakriya::Vyakarana;
use vidyut_prakriya::args::Dhatu;
use vidyut_prakriya::dhatupatha::Dhatupatha;

/// `<crate>/data` – vidyut's pre-built data (run `./fetch-data.sh` first).
pub fn data_dir() -> PathBuf {
  let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
  assert!(
    p.join("kosha").exists(),
    "vidyut data missing: run ./fetch-data.sh in {}",
    env!("CARGO_MANIFEST_DIR")
  );
  p
}

fn convert(text: &str, from: Scheme, to: Scheme) -> String {
  transliterate(text, &Mapping::new(from, to))
}

/// IAST → SLP1 (what vidyut APIs want). `kṛṣṇa` → `kfzRa`
pub fn slp(iast: &str) -> String {
  convert(iast, Scheme::Iast, Scheme::Slp1)
}

/// SLP1 → IAST (for printing). `kfzRa` → `kṛṣṇa`
pub fn iast(slp1: &str) -> String {
  convert(slp1, Scheme::Slp1, Scheme::Iast)
}

/// SLP1 → Devanagari. `kfzRa` → `कृष्ण`
pub fn deva(slp1: &str) -> String {
  convert(slp1, Scheme::Slp1, Scheme::Devanagari)
}

/// `"kfzRa"` → `"kṛṣṇa (कृष्ण)"`
pub fn show(slp1: &str) -> String {
  format!("{} ({})", iast(slp1), deva(slp1))
}

/// First CLI argument (IAST), or `default` (IAST). Lets every example take custom input:
/// `cargo run --example <name> -- "your input"`
pub fn arg_or(default: &str) -> String {
  std::env::args()
    .nth(1)
    .unwrap_or_else(|| default.to_string())
}

/// One-line, human-readable morphology of a kosha entry (used by kosha + cheda examples).
/// `Bavati` → `tiṅanta  bhū (gaṇa Bhvadi)  Lat Kartari Prathama Eka`
pub fn describe(entry: &PadaEntry) -> String {
  match entry {
    PadaEntry::Subanta(s) => {
      let stem = match s.pratipadika_entry() {
        PratipadikaEntry::Basic(b) => format!("prātipadika {}", iast(b.pratipadika().text())),
        PratipadikaEntry::Krdanta(k) => format!(
          "kṛdanta {} + {}{}",
          iast(k.dhatu_text()),
          k.krt().aupadeshika(),
          k.lakara().map(|l| format!(" ({l:?})")).unwrap_or_default()
        ),
      };
      if s.is_avyaya() {
        format!("avyaya   {stem}")
      } else {
        format!(
          "subanta  {stem}  {:?} {:?} {:?}",
          s.linga(),
          s.vibhakti(),
          s.vacana()
        )
      }
    }
    PadaEntry::Tinanta(t) => format!(
      "tiṅanta  {} (gaṇa {})  {:?} {:?} {:?} {:?}",
      iast(t.dhatu_text()),
      t.dhatu()
        .gana()
        .map(|g| format!("{g:?}"))
        .unwrap_or("-".into()),
      t.lakara(),
      t.prayoga(),
      t.purusha(),
      t.vacana()
    ),
  }
}

/// Resolve a root the way a human types it into a correct prakriya `Dhatu`.
///
/// IMPORTANT: `Dhatu::mula(Slp1String::from("kf"), Gana::Tanadi)` is *wrong* — prakriya needs
/// the aupadeśika form with it-markers + accents (`qukf\Y`), otherwise you get e.g.
/// `karitum` instead of `kartum`. So we look the root up in the shipped Dhātupāṭha.
///
/// `query` is either a Dhātupāṭha code (`"08.0010"`) or a clean IAST root (`"kṛ"`),
/// optionally with a gaṇa number (`"kṛ 8"`). Returns (code, artha in IAST, Dhatu).
pub fn dhatu(query: &str) -> (String, String, Dhatu) {
  let dp = Dhatupatha::from_path(data_dir().join("prakriya/dhatupatha.tsv")).unwrap();
  let mut parts = query.split_whitespace();
  let root = parts.next().expect("empty root");
  let gana: Option<u8> = parts.next().map(|g| g.parse().expect("gaṇa must be 1-10"));
  if root.chars().next().is_some_and(|c| c.is_ascii_digit()) {
    let e = dp
      .iter()
      .find(|e| e.code() == root)
      .expect("unknown dhātupāṭha code");
    return (e.code().into(), iast(e.artha()), e.dhatu().clone());
  }
  let want = slp(root);
  let v = Vyakarana::builder().log_steps(false).build();
  dp.iter()
    .filter(|e| gana.is_none_or(|g| e.code().starts_with(&format!("{g:02}."))))
    .find(|e| v.derive_dhatus(e.dhatu()).iter().any(|p| p.text() == want))
    .map(|e| (e.code().into(), iast(e.artha()), e.dhatu().clone()))
    .unwrap_or_else(|| panic!("root '{root}' not found in dhātupāṭha"))
}
