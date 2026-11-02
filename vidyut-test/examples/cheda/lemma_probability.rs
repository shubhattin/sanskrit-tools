//! vidyut-cheda · `chedaka.model().lemma_log_probability(lemma, POSTag)`
//! The statistical model (trained on the DCS corpus) behind cheda's ranking:
//! how frequent is a lemma as a given part of speech. Higher (less negative) = commoner.
//! Useful e.g. to rank dictionary/search results by corpus frequency.
//!
//! Run: cargo run --example cheda_lemma_probability [-- "IAST lemma"]
use vidyut_cheda::{Chedaka, POSTag};
use vidyut_test::{data_dir, echo, slp};

fn main() {
  let chedaka = Chedaka::new(&data_dir()).unwrap();
  let model = chedaka.model();
  let lemmas: Vec<String> = match std::env::args().nth(1) {
    Some(s) => vec![s],
    None => [
      "deva",
      "rāma",
      "kṛṣṇa",
      "gaja",
      "upamanyu",
      "ca",
      "iti",
      "bhū",
      "gam",
      "jṛmbh",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect(),
  };
  println!(
    "{:<10} {:>9} {:>9} {:>9}",
    "lemma", "Subanta", "Tinanta", "Avyaya"
  );
  for l in lemmas {
    let f = |t| {
      model
        .lemma_log_probability(&slp(&l), t)
        .map_or("-".into(), |p: f32| format!("{p:.2}"))
    };
    println!(
      "{:<10} {:>9} {:>9} {:>9}",
      echo(&l),
      f(POSTag::Subanta),
      f(POSTag::Tinanta),
      f(POSTag::Avyaya)
    );
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example cheda_lemma_probability
// lemma        Subanta   Tinanta    Avyaya
// dēva           -2.40     -4.26     -4.45
// rāma           -3.03     -3.85     -6.14
// kṛṣṇa          -2.83         -         -
// gaja           -3.59         -         -
// upamanyu       -4.77         -     -6.74
// cha            -5.42         -     -1.51
// iti            -5.56     -6.44     -1.88
// bhū            -2.87     -2.20     -3.53
// gam            -2.85     -2.82     -3.40
// jṛmbh          -3.77     -4.98     -5.96
