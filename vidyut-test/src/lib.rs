//! Tiny shared helpers for the examples in `examples/`.
//!
//! All vidyut crates speak SLP1 internally. For terminal readability, examples
//! take IAST input (`slp()`, vidyut-lipi) and print lipilekhika `Romanized` output
//! (`rom()`: SLP1 → Devanagari via vidyut-lipi → Romanized via lipilekhika).

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

/// SLP1 → Devanagari. `kfzRa` → `कृष्ण`
pub fn deva(slp1: &str) -> String {
  convert(slp1, Scheme::Slp1, Scheme::Devanagari)
}

/// Romanize SLP1 for ALL display output (words, roots, suffixes, sūtras, meter names), including
/// aupadeśika forms with it-markers/accents (`qukf\Y`, `la~w`, `agni/H`).
///
/// Path: SLP1 → Devanagari (vidyut-lipi) → lipilekhika `Romanized`. lipilekhika covers more
/// than vidyut-lipi's IAST (accents, ~ , etc.).
///
/// Lossless-or-nothing: the result is converted back (Romanized → Devanagari → SLP1) and must
/// give the same Devanagari. If it doesn't, the text is returned UNCONVERTED (plain SLP1) and the case
/// is appended to the file named by env `VIDYUT_LOSS_LOG` (if set) so it can be reported.
pub fn rom(slp1: &str) -> String {
  match rom_checked(slp1) {
    Ok(r) => r,
    Err(loss) => {
      if let Ok(path) = std::env::var("VIDYUT_LOSS_LOG") {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
          .create(true)
          .append(true)
          .open(path)
        {
          let _ = writeln!(f, "{loss}");
        }
      }
      slp1.to_string()
    }
  }
}

/// Like [`rom`] but returns the failed round-trip as an `Err` (a one-line description)
/// instead of silently falling back. Used by the audit example.
pub fn rom_checked(slp1: &str) -> Result<String, String> {
  use lipilekhika::{Script, transliterate as lk};
  let d = deva(slp1);
  let r = lk(&d, Script::Devanagari, Script::Romanized, None).to_string();
  let back_d = lk(&r, Script::Romanized, Script::Devanagari, None).to_string();
  let back_slp = convert(&back_d, Scheme::Devanagari, Scheme::Slp1);
  // Lossless iff (1) Romanized → Devanagari gives back exactly what we sent in and
  // (2) Devanagari ↔ SLP1 is stable. (Compared in Devanagari, not raw SLP1, because
  // vidyut-lipi writes the marks of `eDa~\` as `eDa\~` — same text, different mark order.)
  if back_d == d && deva(&back_slp) == d {
    Ok(r)
  } else {
    Err(format!(
      "slp1={slp1}\tdeva={d}\trom={r}\tback_deva={back_d}\tback_slp1={back_slp}"
    ))
  }
}

/// `"kfzRa"` → `"kṛṣṇa (कृष्ण)"`
pub fn show(slp1: &str) -> String {
  format!("{} ({})", rom(slp1), deva(slp1))
}

/// Echo user input (IAST) in the same `Romanized` style as all other output: `rom(&slp(x))`.
pub fn echo(iast_input: &str) -> String {
  rom(&slp(iast_input))
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
        PratipadikaEntry::Basic(b) => format!("prātipadika {}", rom(b.pratipadika().text())),
        PratipadikaEntry::Krdanta(k) => format!(
          "kṛdanta {} + {}{}",
          rom(k.dhatu_text()),
          rom(k.krt().aupadeshika()),
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
      rom(t.dhatu_text()),
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
/// optionally with a gaṇa number (`"kṛ 8"`). Returns (code, romanized artha, Dhatu).
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
    return (e.code().into(), rom(e.artha()), e.dhatu().clone());
  }
  let want = slp(root);
  let v = Vyakarana::builder().log_steps(false).build();
  dp.iter()
    .filter(|e| gana.is_none_or(|g| e.code().starts_with(&format!("{g:02}."))))
    .find(|e| v.derive_dhatus(e.dhatu()).iter().any(|p| p.text() == want))
    .map(|e| (e.code().into(), rom(e.artha()), e.dhatu().clone()))
    .unwrap_or_else(|| panic!("root '{root}' not found in dhātupāṭha"))
}
