//! vidyut-prakriya · `ganapatha::all_sutras()` (the built-in Gaṇapāṭha)
//! The word-lists (gaṇas) Pāṇini refers to, e.g. sarvādi (pronouns), svarādi
//! (indeclinables). Each entry: sūtra code, name, kind (Basic / Ākṛti), items.
//!
//! Run: cargo run --example prakriya_ganapatha [-- "IAST gaṇa name, e.g. sarvādi"]
use vidyut_prakriya::ganapatha::all_sutras;
use vidyut_test::{rom, slp};

fn main() {
  let all: Vec<_> = all_sutras().collect();
  println!("{} gaṇas\n", all.len());
  for g in all.iter().take(5) {
    println!(
      "{:<8} {:<14} {:?} ({} items)",
      g.code(),
      rom(g.name()),
      g.kind(),
      g.items().len()
    );
  }
  let q = slp(&vidyut_test::arg_or("sarvādi"));
  if let Some(g) = all.iter().find(|g| g.name().starts_with(q.as_str())) {
    let items: Vec<_> = g.items().iter().map(|i| rom(i)).collect();
    println!("\n{} ({}): {}", rom(g.name()), g.code(), items.join(", "));
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example prakriya_ganapatha
// 262 gaṇas
//
// 1.1.27   sarvādiḥ       Basic (35 items)
// 1.1.37   svarādiḥ       Akrti (160 items)
// 1.4.57   chādiḥ         Akrti (155 items)
// 1.4.58   prādiḥ         Basic (22 items)
// 1.4.61   ūryādiḥ        Basic (39 items)
//
// sarvādiḥ (1.1.27): sarva, viśva, ubha, ubhaya, ḍatara, ḍatama, anya, anyatara, itara, tvat, tva, nēma, sama, sima, pūrva, para, avara, dakṣiṇa, uttara, apara, adhara, sva, antara, tyad, tad, yad, ētad, idam, adas, ēka, dvi, yuṣmad, asmad, bhavatu, kim
