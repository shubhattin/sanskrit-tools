//! vidyut-prakriya · `Vyakarana::derive_samasas(&Samasa) -> Vec<Prakriya>`
//! Build compounds from member subantas (each given with its own case) and a
//! `SamasaType` (tatpuruṣa, karmadhāraya, bahuvrīhi, dvandva …). Returns the
//! compound stem (prātipadika); decline it via `Subanta::new(samasa, …)`.
//! (Support is "moderate" upstream — good for common types.)
//!
//! Run: cargo run --example prakriya_samasa
use vidyut_prakriya::Vyakarana;
use vidyut_prakriya::args::*;
use vidyut_test::{iast, slp};

fn sup(stem: &str, vibhakti: Vibhakti) -> Subanta {
  let p = Pratipadika::basic(Slp1String::from(slp(stem)).unwrap());
  Subanta::new(p, Linga::Pum, vibhakti, Vacana::Eka)
}

fn main() {
  let v = Vyakarana::builder().log_steps(false).build();
  use SamasaType::*;
  use Vibhakti::*;
  let cases = [
    (
      "rājan(6) + puruṣa",
      Tatpurusha,
      vec![sup("rājan", Sasthi), sup("puruṣa", Prathama)],
    ),
    (
      "nīla + utpala",
      Karmadharaya,
      vec![sup("nīla", Prathama), sup("utpala", Prathama)],
    ),
    (
      "pīta + ambara",
      Bahuvrihi,
      vec![sup("pīta", Prathama), sup("ambara", Prathama)],
    ),
    (
      "rāma + kṛṣṇa",
      Dvandva,
      vec![sup("rāma", Prathama), sup("kṛṣṇa", Prathama)],
    ),
    (
      "upa + kumbha",
      Avyayibhava,
      vec![
        Subanta::avyaya(Pratipadika::avyaya(Slp1String::from("upa").unwrap())),
        sup("kumbha", Prathama),
      ],
    ),
  ];
  for (label, ty, padas) in cases {
    let s = Samasa::builder()
      .padas(padas)
      .samasa_type(ty)
      .build()
      .unwrap();
    let mut f: Vec<_> = v
      .derive_samasas(&s)
      .iter()
      .map(|p| iast(&p.text()))
      .collect();
    f.dedup();
    // Decline the compound stem as a full word (puM prathamā eka; dvandva as dvi).
    let vacana = if ty == Dvandva {
      Vacana::Dvi
    } else {
      Vacana::Eka
    };
    let word: Vec<_> = if ty == Avyayibhava {
      vec!["(avyaya, indeclinable)".into()]
    } else {
      v.derive_subantas(&Subanta::new(s.clone(), Linga::Pum, Prathama, vacana))
        .iter()
        .map(|p| iast(&p.text()))
        .collect()
    };
    println!(
      "{:<22} {:<13} stem: {:<18} word: {}",
      label,
      format!("{ty:?}"),
      f.join(" / "),
      word.join(" / ")
    );
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example prakriya_samasa
// rājan(6) + puruṣa      Tatpurusha    stem: rājapuruṣa         word: rājapuruṣaḥ
// nīla + utpala          Karmadharaya  stem: nīlotpala          word: nīlotpalaḥ
// pīta + ambara          Bahuvrihi     stem: pītāmbara          word: pītāmbaraḥ
// rāma + kṛṣṇa           Dvandva       stem: rāmakṛṣṇa          word: rāmakṛṣṇau
// upa + kumbha           Avyayibhava   stem: upakumbham         word: (avyaya, indeclinable)
