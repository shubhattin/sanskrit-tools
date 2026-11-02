//! vidyut-prakriya · `Vyakarana::derive_vakyas(&[Pada]) -> Vec<Prakriya>`
//! Join already-formed words into a sentence applying (Pāṇinian) external sandhi
//! between them — the morphology-aware counterpart of vidyut-sandhi's rule table.
//! Words are given in SLP1 with final s/r (not visarga). Returns all optional variants.
//!
//! Run: cargo run --example prakriya_vakya [-- "IAST words with final s, space separated"]
use vidyut_prakriya::Vyakarana;
use vidyut_prakriya::args::Pada;
use vidyut_test::{echo, rom, slp};

fn main() {
  let v = Vyakarana::new();
  let sentences: Vec<String> = match std::env::args().nth(1) {
    Some(s) => vec![s],
    None => vec![
      "rāmas gacchati".into(),
      "rāmas atra".into(),
      "tat ca".into(),
      "ca iti".into(),
      "devas indras".into(),
      "vāk īśas".into(),
      "tam katham citrapakṣam".into(),
    ],
  };
  for s in sentences {
    let padas: Vec<_> = slp(&s).split_whitespace().map(Pada::from_text).collect();
    // `p.text()` concatenates all terms with no spaces; the last history step keeps the
    // terms separate, shown in [..]. Note: no avagraha is inserted (rāmo 'tra → "rāmo tra").
    let mut out: Vec<_> = v
      .derive_vakyas(&padas)
      .iter()
      .map(|p| {
        let last = p.history().last().unwrap();
        let words: Vec<_> = last
          .result()
          .iter()
          .map(|t| t.text())
          .filter(|t| !t.is_empty())
          .collect();
        format!("{}  [{}]", rom(&p.text()), rom(&words.join(" ")))
      })
      .collect();
    out.dedup();
    println!("{:<24} -> {}", echo(&s), out.join("   |   "));
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example prakriya_vakya
// rāmas gachChati          -> rāmōgachChati  [rāmō gachChati]
// rāmas atra               -> rāmōtra  [rāmō tra]
// tat cha                  -> tachcha  [tach cha]
// cha iti                  -> chēti  [ch ēti]
// dēvas indras             -> devaindraH  [dēva indraḥ]
// vāk īśas                 -> vāgīśaḥ  [vāg īśaḥ]
// tam katham chitrapakṣam  -> taṁkathaṁchitrapakṣam  [taṁ kathaṁ chitrapakṣam]   |   taṅkathañchitrapakṣam  [taṅ kathañ chitrapakṣam]
