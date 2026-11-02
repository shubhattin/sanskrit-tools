//! vidyut-chandas · `chandas.classify_all(verse) -> Matches`
//! All vṛtta (syllabic) meters the text is compatible with, not just the best one.
//! Handy when a single pāda is ambiguous between several meters.
//!
//! Run: cargo run --example chandas_classify_all [-- "IAST line"]
use vidyut_chandas::Chandas;
use vidyut_test::{data_dir, echo, rom, slp};

fn main() {
  let chandas = Chandas::from_file(data_dir().join("chandas/meters.tsv")).unwrap();
  let line = vidyut_test::arg_or("vāgarthāviva saṃpṛktau");
  let all = chandas.classify_all(slp(&line));
  println!(
    "'{}' is compatible with {} meters:",
    echo(&line),
    all.padyas().len()
  );
  for (p, t) in all.padyas().iter().zip(all.match_types()) {
    println!("  {:<16} {:?}", rom(p.name()), t);
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example chandas_classify_all
// 'vāgarthāviva saṁpṛktau' is compatible with 2 meters:
//   madalēkhā        Prefix
//   alōlā            Prefix
