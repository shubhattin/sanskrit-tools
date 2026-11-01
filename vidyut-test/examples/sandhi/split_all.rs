//! vidyut-sandhi · `Splitter::split_all(text) -> Vec<Split>`
//! Every first-word/remainder split at every position of the first chunk.
//! This is the raw candidate generator that vidyut-cheda filters with the kosha.
//!
//! Run: cargo run --example sandhi_split_all [-- "IAST text"]
use vidyut_sandhi::{Kind, Splitter};
use vidyut_test::{data_dir, iast, slp};

fn main() {
  let splitter = Splitter::from_csv(data_dir().join("sandhi/rules.csv")).unwrap();
  let text = slp(&vidyut_test::arg_or("tathaiva"));
  let splits = splitter.split_all(&text);
  let real: Vec<_> = splits
    .iter()
    .filter(|s| s.kind() == Kind::Standard && s.is_valid())
    .collect();
  println!(
    "'{}': {} splits total, {} are real sandhi reversals (Kind::Standard + valid):",
    iast(&text),
    splits.len(),
    real.len()
  );
  for s in real {
    println!("  {} + {}", iast(s.first()), iast(s.second()));
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example sandhi_split_all
// 'tathaiva': 10 splits total, 4 are real sandhi reversals (Kind::Standard + valid):
//   tatha + eva
//   tatha + aiva
//   tathā + eva
//   tathā + aiva
