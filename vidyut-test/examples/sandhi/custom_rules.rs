//! vidyut-sandhi · `SplitsMap` + `Splitter::from_map(map)`
//! Build a splitter from your own hand-written rules instead of the CSV
//! (result → (first, second)). Handy for tests or domain-specific splitting.
//!
//! Run: cargo run --example sandhi_custom_rules
use vidyut_sandhi::{Kind, SplitsMap, Splitter};
use vidyut_test::{iast, slp};

fn main() {
  let mut map = SplitsMap::new();
  // result "e" may come from a+i or A+I ; "o" from a+u
  map.insert("e".into(), ("a".into(), "i".into()));
  map.insert("e".into(), ("A".into(), "I".into()));
  map.insert("o".into(), ("a".into(), "u".into()));
  let s = Splitter::from_map(map);

  for word in ["ceti", "gaṅgodakam"] {
    let w = slp(word);
    print!("{word}: ");
    let found: Vec<_> = s
      .split_all(&w)
      .into_iter()
      .filter(|x| x.kind() == Kind::Standard)
      .map(|x| format!("{} + {}", iast(x.first()), iast(x.second())))
      .collect();
    println!("{}", found.join("  |  "));
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example sandhi_custom_rules
// ceti: ca + iti  |  cā + īti
// gaṅgodakam: gaṅga + udakam
