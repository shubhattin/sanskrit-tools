//! vidyut-prakriya · `Vyakarana::derive_subantas(&Subanta) -> Vec<Prakriya>`
//! Decline a nominal stem: 8 vibhaktis × 3 vacanas for a given liṅga.
//!
//! Run: cargo run --example prakriya_subanta [-- "IAST stem" "puM|strI|napuMsaka"]
use vidyut_prakriya::Vyakarana;
use vidyut_prakriya::args::*;
use vidyut_test::{echo, rom, slp};

fn main() {
  let v = Vyakarana::builder().log_steps(false).build();
  let stem = vidyut_test::arg_or("rāma");
  let linga: Linga = std::env::args()
    .nth(2)
    .map(|s| s.parse().unwrap())
    .unwrap_or(Linga::Pum);
  let prati = Pratipadika::basic(Slp1String::from(slp(&stem)).unwrap());
  println!(
    "{} ({}) — columns = eka / dvi / bahu\n",
    echo(&stem),
    format!("{linga:?}")
  );

  for vibhakti in Vibhakti::iter() {
    let row: Vec<_> = Vacana::iter()
      .map(|vacana| {
        let s = Subanta::new(prati.clone(), linga, vibhakti, vacana);
        let mut f: Vec<_> = v
          .derive_subantas(&s)
          .iter()
          .map(|p| rom(&p.text()))
          .collect();
        f.dedup();
        f.join("/")
      })
      .collect();
    println!(
      "{:<11} {:<14} {:<16} {}",
      format!("{vibhakti:?}"),
      row[0],
      row[1],
      row[2]
    );
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example prakriya_subanta
// rāma (Pum) — columns = eka / dvi / bahu
//
// Prathama    rāmaḥ          rāmau            rāmāḥ
// Dvitiya     rāmam          rāmau            rāmān
// Trtiya      rāmēṇa         rāmābhyām        rāmaiḥ
// Caturthi    rāmāya         rāmābhyām        rāmēbhyaḥ
// Panchami    rāmāt/rāmād    rāmābhyām        rāmēbhyaḥ
// Sasthi      rāmasya        rāmayōḥ          rāmāṇām
// Saptami     rāmē           rāmayōḥ          rāmēṣu
// Sambodhana  rāma           rāmau            rāmāḥ
