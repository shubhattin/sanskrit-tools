//! vidyut-lipi · `Mapping::new(from, to)` + free fn `transliterate(text, &mapping)`
//! Lower-level API: build one explicit mapping and reuse it (no `&mut` needed,
//! so it can live in a static / be shared across threads). Also shows round-trips.
//!
//! Run: cargo run --example lipi_mapping [-- "IAST text"]
use vidyut_lipi::{Mapping, Scheme, transliterate};

fn main() {
  let input = vidyut_test::arg_or("saṃskṛtam");
  let to_deva = Mapping::new(Scheme::Iast, Scheme::Devanagari);
  let deva_to_slp = Mapping::new(Scheme::Devanagari, Scheme::Slp1);
  let slp_to_iast = Mapping::new(Scheme::Slp1, Scheme::Iast);

  let deva = transliterate(&input, &to_deva);
  let slp = transliterate(&deva, &deva_to_slp);
  let back = transliterate(&slp, &slp_to_iast);
  println!(
    "IAST  {input}\nDeva  {deva}\nSLP1  {slp}\nIAST  {back}   (round-trip ok: {})",
    back == input
  );

  // Mapping knows its endpoints.
  println!("\nmapping: {:?} -> {:?}", to_deva.from(), to_deva.to());

  // Special signs: avagraha, anusvāra, oṃ, daṇḍa, numerals.
  for s in [
    "so'ham",
    "haṃsaḥ",
    "kiṃ ca",
    "oṃ namaḥ śivāya",
    "rāmaḥ । 108 ॥",
  ] {
    println!("{:<16} -> {}", s, transliterate(s, &to_deva));
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example lipi_mapping
// IAST  saṃskṛtam
// Deva  संस्कृतम्
// SLP1  saMskftam
// IAST  saṃskṛtam   (round-trip ok: true)
//
// mapping: Iast -> Devanagari
// so'ham           -> सोऽहम्
// haṃsaḥ           -> हंसः
// kiṃ ca           -> किं च
// oṃ namaḥ śivāya  -> ॐ नमः शिवाय
// rāmaḥ । 108 ॥    -> रामः । १०८ ॥
