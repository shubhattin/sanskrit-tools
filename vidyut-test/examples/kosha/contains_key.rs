//! vidyut-kosha · `kosha.contains_key(word) -> bool`
//! Fast yes/no: is this an attested/generated Sanskrit word form? (spell-check,
//! filtering candidates, validating user input). No allocation of entries.
//!
//! Run: cargo run --example kosha_contains_key [-- "IAST word"]
use vidyut_kosha::Kosha;
use vidyut_test::{data_dir, slp};

fn main() {
  let kosha = Kosha::new(data_dir().join("kosha")).unwrap();
  let words: Vec<String> = match std::env::args().nth(1) {
    Some(s) => vec![s],
    None => [
      "naras",
      "narās",
      "narasya",
      "narasyā",
      "gacchati",
      "gacchiti",
      "putrīyati",
      "sundarā",
      "sundarī",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect(),
  };
  for w in words {
    println!(
      "{:<10} {}",
      w,
      if kosha.contains_key(&slp(&w)) {
        "✓"
      } else {
        "✗"
      }
    );
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example kosha_contains_key
// naras      ✓
// narās      ✓
// narasya    ✓
// narasyā    ✗
// gacchati   ✓
// gacchiti   ✗
// putrīyati  ✗
// sundarā    ✗
// sundarī    ✓
