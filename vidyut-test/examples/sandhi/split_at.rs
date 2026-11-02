//! vidyut-sandhi · `Splitter::split_at(text, i) -> Vec<Split>`
//! Undo sandhi at ONE position: every way `text` could be split so that the first
//! part ends around byte `i`. Purely phonetic (no dictionary) → overgenerates;
//! `split.is_valid()` drops some impossible splits. (cheda adds the dictionary.)
//!
//! Run: cargo run --example sandhi_split_at [-- "IAST text" [byte index in SLP1]]
use vidyut_sandhi::Splitter;
use vidyut_test::{data_dir, rom, slp};

fn main() {
  let splitter = Splitter::from_csv(data_dir().join("sandhi/rules.csv")).unwrap();
  let text = slp(&vidyut_test::arg_or("ceti"));
  let i: usize = std::env::args()
    .nth(2)
    .and_then(|s| s.parse().ok())
    .unwrap_or(1);

  println!(
    "splits of '{}' at SLP1 byte {i} ('{}'):",
    rom(&text),
    &text[i..i + 1]
  );
  for s in splitter.split_at(&text, i) {
    println!(
      "  {:<10} + {:<12} kind={:?} end_of_chunk={} valid={}",
      rom(s.first()),
      rom(s.second()),
      s.kind(),
      s.is_end_of_chunk(),
      s.is_valid()
    );
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example sandhi_split_at
// splits of 'chēti' at SLP1 byte 1 ('e'):
//   chē        + ti           kind=Prefix end_of_chunk=false valid=true
//   cha        + iti          kind=Standard end_of_chunk=false valid=true
//   cha        + īti          kind=Standard end_of_chunk=false valid=true
//   chā        + iti          kind=Standard end_of_chunk=false valid=true
//   chā        + īti          kind=Standard end_of_chunk=false valid=true
