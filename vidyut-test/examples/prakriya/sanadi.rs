//! vidyut-prakriya · `Dhatu::with_sanadi(&[Sanadi])`
//! Derived roots: causative (ṇic), desiderative (san), intensive (yaṅ, yaṅ-luk),
//! and stacks of them (e.g. desiderative of causative). Then conjugate as usual.
//!
//! Run: cargo run --example prakriya_sanadi [-- "IAST root [gaṇa]" | "dhātupāṭha code"]
use vidyut_prakriya::Vyakarana;
use vidyut_prakriya::args::*;
use vidyut_test::{echo, rom};

fn main() {
  let v = Vyakarana::builder().log_steps(false).build();
  let root = vidyut_test::arg_or("bhū");
  let (code, artha, base) = vidyut_test::dhatu(&root);
  println!(
    "{} → dhātupāṭha {code} {} ({artha})",
    echo(&root),
    rom(base.aupadeshika().unwrap_or(""))
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
    let mut stem: Vec<_> = v.derive_dhatus(&d).iter().map(|p| rom(&p.text())).collect();
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
      .map(|p| rom(&p.text()))
      .collect();
    f.dedup();
    println!(
      "{:<22} root: {:<14} laṭ 3sg: {}",
      // romanize only the Sanskrit part of the label, not the English gloss
      match label.split_once(" (") {
        Some((sa, gloss)) => format!("{} ({gloss}", echo(sa)),
        None if label == "plain" => label.to_string(),
        None => echo(label),
      },
      stem.join("/"),
      f.join(" / ")
    );
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example prakriya_sanadi
// bhū → dhātupāṭha 01.0001 bhū (sattāyām)
// plain                  root: bhū            laṭ 3sg: bhavati
// ṇich (causative)       root: bhāvi          laṭ 3sg: bhāvayatē / bhāvayati
// san (desiderative)     root: bubhūṣa        laṭ 3sg: bubhūṣati
// yaṅ (intensive)        root: bōbhūya        laṭ 3sg: bōbhūyatē
// yaṅ-luk (intensive)    root: bōbhū          laṭ 3sg: bōbhavīti / bōbhōti
// ṇich + san             root: bibhāvayiṣa    laṭ 3sg: bibhāvayiṣatē / bibhāvayiṣati
