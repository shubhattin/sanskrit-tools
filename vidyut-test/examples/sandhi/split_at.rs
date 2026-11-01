//! vidyut-sandhi · `Splitter::split_at(text, i) -> Vec<Split>`
//! Undo sandhi at ONE position: every way `text` could be split so that the first
//! part ends around byte `i`. Purely phonetic (no dictionary) → overgenerates;
//! `split.is_valid()` drops some impossible splits. (cheda adds the dictionary.)
//!
//! Run: cargo run --example sandhi_split_at [-- "IAST text" [byte index in SLP1]]
use vidyut_sandhi::Splitter;
use vidyut_test::{data_dir, iast, slp};

fn main() {
  let splitter = Splitter::from_csv(data_dir().join("sandhi/rules.csv")).unwrap();
  let text = slp(&vidyut_test::arg_or("ceti"));
  let i: usize = std::env::args()
    .nth(2)
    .and_then(|s| s.parse().ok())
    .unwrap_or(1);

  println!(
    "splits of '{}' at SLP1 byte {i} ('{}'):",
    iast(&text),
    &text[i..i + 1]
  );
  for s in splitter.split_at(&text, i) {
    println!(
      "  {:<10} + {:<12} kind={:?} end_of_chunk={} valid={}",
      iast(s.first()),
      iast(s.second()),
      s.kind(),
      s.is_end_of_chunk(),
      s.is_valid()
    );
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example sandhi_split_at
// splits of 'ceti' at SLP1 byte 1 ('e'):
//   ce         + ti           kind=Prefix end_of_chunk=false valid=true
//   ca         + iti          kind=Standard end_of_chunk=false valid=true
//   ca         + īti          kind=Standard end_of_chunk=false valid=true
//   cā         + iti          kind=Standard end_of_chunk=false valid=true
//   cā         + īti          kind=Standard end_of_chunk=false valid=true
