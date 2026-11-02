//! vidyut-cheda · `Chedaka::new(data_dir)` + `chedaka.run(text) -> Vec<Token>`
//! Segment running Sanskrit (sandhi + no spaces) into words, each with its lemma,
//! POS tag and full morphology from the kosha. Input must be ASCII SLP1.
//! Loads data/{sandhi/rules.csv, kosha/, cheda/model.msgpack}.
//!
//! ⚠ UPSTREAM BUG (rev 8da2f90): `impl From<&Option<PadaEntry>> for State` in
//! vidyut-cheda/src/scoring.rs has the avyaya check inverted, so every ordinary noun is
//! scored as an avyaya → poor rankings below (rāmo → "rāmas" ram+kvip pl., vanaṃ → van+am).
//! Swapping the two branches gives: rāmas=rāma Pum Prathama Eka | vanam=vana | gacchati,
//! and tathā/eva/na/atra all as Avyaya (verified with a locally patched build).
//! Introduced in commit 178cc8f "[kosha] Support tinanta prefixes" (Jan 2025, after py-0.3.1);
//! this is the likely root cause of open issue ambuda-org/vidyut#254 "worse cheda?".
//!
//! Run: cargo run --example cheda_segment [-- "IAST sentence"]
use vidyut_cheda::Chedaka;
use vidyut_test::{data_dir, echo, rom, slp};

fn main() {
  let chedaka = Chedaka::new(&data_dir()).unwrap();
  let inputs: Vec<String> = match std::env::args().nth(1) {
    Some(s) => vec![s],
    None => vec![
      "dharmakṣetre kurukṣetre samavetā yuyutsavaḥ".into(),
      "rāmo vanaṃ gacchati".into(),
      "tathaiva".into(),
      "nāstyatra".into(),
    ],
  };
  for text in inputs {
    println!("\n» {}", echo(&text));
    for t in chedaka.run(&slp(&text)).unwrap() {
      let morph = t
        .data()
        .as_ref()
        .map(vidyut_test::describe)
        .unwrap_or("-".into());
      println!(
        "  {:<14} {:<8} {morph}",
        rom(t.text()),
        format!("{:?}", t.pos_tag())
      );
    }
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example cheda_segment
//
// » dharmakṣētrē kurukṣētrē samavētā yuyutsavaḥ
//   dharmakṣētrē   Subanta  subanta  prātipadika dharmakṣētra  Napumsaka Saptami Eka
//   kurukṣētrē     Subanta  subanta  prātipadika kurukṣētra  Napumsaka Saptami Eka
//   samavēt        Tinanta  tiṅanta  saṁvid (gaṇa Adadi)  Lan Kartari Prathama Eka
//   ās             Subanta  subanta  prātipadika a  Pum Prathama Bahu
//   yuyutsavas     Subanta  subanta  prātipadika yuyutsu  Pum Prathama Bahu
//
// » rāmō vanaṁ gachChati
//   rāmas          Subanta  subanta  kṛdanta ram + kvim̐p  Pum Prathama Bahu
//   van            Subanta  subanta  kṛdanta av + kvasum̐ (Lit)  Pum Sambodhana Eka
//   am             Subanta  subanta  prātipadika a  Pum Dvitiya Eka
//   gachChati      Tinanta  tiṅanta  gam (gaṇa Bhvadi)  Lat Kartari Prathama Eka
//
// » tathaiva
//   tathā          Avyaya   avyaya   prātipadika tathā
//   ēva            Subanta  subanta  prātipadika ēva  Napumsaka Sambodhana Eka
//
// » nāstyatra
//   na             Subanta  subanta  prātipadika na  Napumsaka Sambodhana Eka
//   asti           Tinanta  tiṅanta  as (gaṇa Adadi)  Lat Kartari Prathama Eka
//   atra           Subanta  subanta  prātipadika atra  Napumsaka Sambodhana Eka
