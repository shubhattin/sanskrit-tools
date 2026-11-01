//! vidyut-sandhi · `generate_rules() -> Vec<Rule>`
//! Produce the full table of sandhi rules as (first, second, result) triples,
//! e.g. a + i → e. This is the same table shipped as `data/sandhi/rules.csv`.
//!
//! Run: cargo run --example sandhi_generate_rules [-- "IAST result to filter on, e.g. e"]
use vidyut_sandhi::generate_rules;
use vidyut_test::{iast, slp};

fn main() {
  let rules = generate_rules();
  println!("{} rules generated\n", rules.len());

  // Which (first + second) pairs produce this result?
  let target = slp(&vidyut_test::arg_or("o"));
  println!("rules whose result is '{}':", iast(&target));
  for r in rules.iter().filter(|r| r.result() == target) {
    println!(
      "  {:>4} + {:<4} -> {}",
      iast(r.first()),
      iast(r.second()),
      iast(r.result())
    );
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example sandhi_generate_rules
// 1468 rules generated
//
// rules whose result is 'o':
//      a + u    -> o
//      a + ū    -> o
//      ā + u    -> o
//      ā + ū    -> o
