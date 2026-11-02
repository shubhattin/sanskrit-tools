//! vidyut-prakriya · `Vyakarana::derive_stryantas(&Pratipadika) -> Vec<Prakriya>`
//! Feminine stem formation (strī-pratyayas ṭāp, ṅīp, ṅīṣ, ūṅ …): aja → ajā,
//! gaura → gaurī, rājan → rājñī, kartṛ → kartrī.
//! Limits seen below: a *basic* stem carries no derivational info, so `kumāra`
//! gives kumārā (probably needs the "first age" sense of 4.1.20; unverified) and `bhavat`
//! gets no ṅīp: it must be built as `Krdanta::new(bhā, Unadi::qavatu)` (upstream test 4.1.6).
//!
//! Run: cargo run --example prakriya_stryanta [-- "IAST masculine stem"]
use vidyut_prakriya::Vyakarana;
use vidyut_prakriya::args::*;
use vidyut_test::{echo, rom, slp};

fn main() {
  let v = Vyakarana::builder().log_steps(false).build();
  let stems: Vec<String> = match std::env::args().nth(1) {
    Some(s) => vec![s],
    None => [
      "aja", "aśva", "kumāra", "gaura", "nartaka", "bhavat", "kartṛ", "rājan", "sundara",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect(),
  };
  for s in stems {
    let p = Pratipadika::basic(Slp1String::from(slp(&s)).unwrap());
    let mut f: Vec<_> = v
      .derive_stryantas(&p)
      .iter()
      .map(|p| rom(&p.text()))
      .collect();
    f.dedup();
    println!(
      "{:<9} -> {}",
      echo(&s),
      if f.is_empty() {
        "-".into()
      } else {
        f.join(" / ")
      }
    );
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example prakriya_stryanta
// aja       -> ajā
// aśva      -> aśvā
// kumāra    -> kumārā
// gaura     -> gaurī
// nartaka   -> nartakā
// bhavat    -> bhavat
// kartṛ     -> kartrī
// rājan     -> rājñī
// sundara   -> sundarī
