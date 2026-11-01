//! vidyut-prakriya · `Dhatu::nama(pratipadika, Some(Sanadi::kyac|kAmyac|kyaN))`
//! Denominative verbs: turn a noun into a root ("wish for a son" → putrīyati,
//! putrakāmyati; "act like a hawk" → śyenāyate), then conjugate it.
//!
//! Run: cargo run --example prakriya_namadhatu [-- "IAST stem"]
use vidyut_prakriya::Vyakarana;
use vidyut_prakriya::args::*;
use vidyut_test::{iast, slp};

fn main() {
  let v = Vyakarana::builder().log_steps(false).build();
  let stems: Vec<String> = match std::env::args().nth(1) {
    Some(s) => vec![s],
    None => vec!["putra".into(), "śyena".into(), "kṛṣṇa".into()],
  };
  for stem in stems {
    let p = Pratipadika::basic(Slp1String::from(slp(&stem)).unwrap());
    for sanadi in [Sanadi::kyac, Sanadi::kAmyac, Sanadi::kyaN] {
      let d = Dhatu::nama(p.clone(), Some(sanadi));
      let t = Tinanta::new(
        d,
        Prayoga::Kartari,
        Lakara::Lat,
        Purusha::Prathama,
        Vacana::Eka,
      );
      let f: Vec<_> = v
        .derive_tinantas(&t)
        .iter()
        .map(|p| iast(&p.text()))
        .collect();
      println!(
        "{:<8} + {:<7} -> {}",
        stem,
        sanadi.as_str(),
        if f.is_empty() {
          "-".into()
        } else {
          f.join(" / ")
        }
      );
    }
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example prakriya_namadhatu
// putra    + kyac    -> putrīyati
// putra    + kAmyac  -> putrakāmyati
// putra    + kyaN    -> putrāyate
// śyena    + kyac    -> śyenīyati
// śyena    + kAmyac  -> śyenakāmyati
// śyena    + kyaN    -> śyenāyate
// kṛṣṇa    + kyac    -> kṛṣṇīyati
// kṛṣṇa    + kAmyac  -> kṛṣṇakāmyati
// kṛṣṇa    + kyaN    -> kṛṣṇāyate
