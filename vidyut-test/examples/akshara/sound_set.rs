//! vidyut-akshara · `Set::from(chars)` / `set.contains(c)` / `Display`
//! Tiny low-level helper used inside prakriya/sandhi/chandas: a const 128-byte
//! bitset of SLP1 sounds. `Display` prints members in traditional varṇa order.
//! (Not much else in this crate yet — it's a building block, not a user API.)
//!
//! Run: cargo run --example akshara_sound_set [-- "IAST word to classify"]
use vidyut_akshara::Set;
use vidyut_test::{iast, slp};

// const-constructible, so sets can be statics (as in vidyut's own code).
const AC: Set = Set::from("aAiIuUfFxXeEoO"); // vowels
const HAL: Set = Set::from("kKgGNcCjJYwWqQRtTdDnpPbBmyrlvSzsh"); // consonants
const GHOSHA: Set = Set::from("gGNjJYqQRdDnbBmyrlvh"); // voiced consonants

fn main() {
  // Display gives SLP1 in varṇa order; spaced IAST alongside for readability.
  let spaced = |s: &Set| {
    s.to_string()
      .chars()
      .map(|c| iast(&c.to_string()))
      .collect::<Vec<_>>()
      .join(" ")
  };
  println!("AC     = {:<16} {}", AC.to_string(), spaced(&AC));
  println!("HAL    = {:<16} {}", HAL.to_string(), spaced(&HAL));
  println!("GHOSHA = {:<16} {}\n", GHOSHA.to_string(), spaced(&GHOSHA));

  let mut aspirates = Set::new();
  for c in "KGCJWQTDPB".chars() {
    aspirates.add(c);
  }
  println!("custom aspirates set = {}\n", spaced(&aspirates));

  let word = slp(&vidyut_test::arg_or("bhagavadgītā"));
  for c in word.chars() {
    let kind = if AC.contains(c) {
      "ac (vowel)"
    } else if GHOSHA.contains(c) {
      "hal, ghoṣa"
    } else if HAL.contains(c) {
      "hal, aghoṣa"
    } else {
      "?"
    };
    let asp = if aspirates.contains(c) {
      " + mahāprāṇa"
    } else {
      ""
    };
    println!("  {:<3} {kind}{asp}", iast(&c.to_string()));
  }
  println!("\ncontains_any vowel? {}", AC.contains_any(&word));
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example akshara_sound_set
// AC     = aAiIuUfFxXeEoO   a ā i ī u ū ṛ ṝ ḷ ḹ e ai o au
// HAL    = kKgGNcCjJYwWqQRtTdDnpPbBmyrlvSzsh k kh g gh ṅ c ch j jh ñ ṭ ṭh ḍ ḍh ṇ t th d dh n p ph b bh m y r l v ś ṣ s h
// GHOSHA = gGNjJYqQRdDnbBmyrlvh g gh ṅ j jh ñ ḍ ḍh ṇ d dh n b bh m y r l v h
//
// custom aspirates set = kh gh ch jh ṭh ḍh th dh ph bh
//
//   bh  hal, ghoṣa + mahāprāṇa
//   a   ac (vowel)
//   g   hal, ghoṣa
//   a   ac (vowel)
//   v   hal, ghoṣa
//   a   ac (vowel)
//   d   hal, ghoṣa
//   g   hal, ghoṣa
//   ī   ac (vowel)
//   t   hal, aghoṣa
//   ā   ac (vowel)
//
// contains_any vowel? true
