//! vidyut-prakriya · `Dhatu::with_sanadi(&[Sanadi])`
//! Derived roots: causative (ṇic), desiderative (san), intensive (yaṅ, yaṅ-luk),
//! and stacks of them (e.g. desiderative of causative). Then conjugate as usual.
//!
//! Run: cargo run --example prakriya_sanadi [-- "IAST root [gaṇa]" | "dhātupāṭha code"]
use vidyut_prakriya::Vyakarana;
use vidyut_prakriya::args::*;
use vidyut_test::iast;

fn main() {
  let v = Vyakarana::builder().log_steps(false).build();
  let root = vidyut_test::arg_or("bhū");
  let (code, artha, base) = vidyut_test::dhatu(&root);
  println!(
    "{root} → dhātupāṭha {code} {} ({artha})",
    iast(base.aupadeshika().unwrap_or(""))
  );

  use Sanadi::*;
  let combos: [(&str, &[Sanadi]); 6] = [
    ("plain", &[]),
    ("ṇic (causative)", &[Ric]),
    ("san (desiderative)", &[san]),
    ("yaṅ (intensive)", &[yaN]),
    ("yaṅ-luk (intensive)", &[yaNluk]),
    ("ṇic + san", &[Ric, san]),
  ];
  for (label, sanadi) in combos {
    let d = base.clone().with_sanadi(sanadi);
    let mut stem: Vec<_> = v
      .derive_dhatus(&d)
      .iter()
      .map(|p| iast(&p.text()))
      .collect();
    stem.dedup();
    let t = Tinanta::new(
      d,
      Prayoga::Kartari,
      Lakara::Lat,
      Purusha::Prathama,
      Vacana::Eka,
    );
    let mut f: Vec<_> = v
      .derive_tinantas(&t)
      .iter()
      .map(|p| iast(&p.text()))
      .collect();
    f.dedup();
    println!(
      "{:<22} root: {:<14} laṭ 3sg: {}",
      label,
      stem.join("/"),
      f.join(" / ")
    );
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example prakriya_sanadi
// bhū → dhātupāṭha 01.0001 bhū (sattāyām)
// plain                  root: bhū            laṭ 3sg: bhavati
// ṇic (causative)        root: bhāvi          laṭ 3sg: bhāvayate / bhāvayati
// san (desiderative)     root: bubhūṣa        laṭ 3sg: bubhūṣati
// yaṅ (intensive)        root: bobhūya        laṭ 3sg: bobhūyate
// yaṅ-luk (intensive)    root: bobhū          laṭ 3sg: bobhavīti / bobhoti
// ṇic + san              root: bibhāvayiṣa    laṭ 3sg: bibhāvayiṣate / bibhāvayiṣati
