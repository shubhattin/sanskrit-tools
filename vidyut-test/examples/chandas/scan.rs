//! vidyut-chandas · `match.aksharas() -> &[Vec<Akshara>]` (syllable scansion)
//! Every classify result carries the scansion: syllables (akṣara) per line with
//! their weight — L (laghu, 1 mātrā) or G (guru, 2 mātrās).
//!
//! Run: cargo run --example chandas_scan [-- "IAST line"]
use vidyut_chandas::Chandas;
use vidyut_test::{data_dir, rom, slp};

fn main() {
  let chandas = Chandas::from_file(data_dir().join("chandas/meters.tsv")).unwrap();
  let line = vidyut_test::arg_or("mātaḥ samastajagatāṃ madhukaiṭabhāreḥ");
  let m = chandas.classify(slp(&line));
  for row in m.aksharas() {
    let sylls: Vec<_> = row.iter().map(|a| rom(a.text())).collect();
    let weights: String = row.iter().map(|a| a.weight().as_str()).collect();
    let matras: i32 = row.iter().map(|a| a.num_matras()).sum();
    println!("syllables: {}", sylls.join(" · "));
    println!(
      "pattern  : {weights}   ({} akṣaras, {matras} mātrās)",
      row.len()
    );
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example chandas_scan
// syllables: mā · taḥ · sa · ma · sta · ja · ga · tāṁ · ma · dhu · kai · ṭa · bhā · rēḥ
// pattern  : GGLGLLLGLLGLGG   (14 akṣaras, 21 mātrās)
