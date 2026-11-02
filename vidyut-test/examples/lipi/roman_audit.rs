//! vidyut-lipi + lipilekhika · `vidyut_test::rom_checked(slp1)` (round-trip audit)
//! How the display romanization works: SLP1 → Devanagari (vidyut-lipi) → lipilekhika
//! `Romanized`, then converted back and compared. This audits every SLP1 string vidyut
//! exposes as a *name/label* (roots, kṛt / taddhita / uṇādi / sanadi suffixes, meters,
//! gaṇas) and lists the ones that do NOT survive the round trip (those stay as plain SLP1).
//!
//! Run: cargo run --example lipi_roman_audit
use vidyut_chandas::Chandas;
use vidyut_prakriya::args::{BaseKrt, Sanadi, Taddhita, Unadi};
use vidyut_prakriya::dhatupatha::Dhatupatha;
use vidyut_prakriya::ganapatha::all_sutras;
use vidyut_test::{data_dir, rom_checked};

fn main() {
  let dp = Dhatupatha::from_path(data_dir().join("prakriya/dhatupatha.tsv")).unwrap();
  let chandas = Chandas::from_file(data_dir().join("chandas/meters.tsv")).unwrap();

  let mut sets: Vec<(&str, Vec<String>)> = vec![
    (
      "dhātupāṭha roots",
      dp.iter()
        .map(|e| e.dhatu().aupadeshika().unwrap_or("").to_string())
        .collect(),
    ),
    (
      "kṛt suffixes",
      BaseKrt::iter().map(|k| k.as_str().to_string()).collect(),
    ),
    (
      "taddhita suffixes",
      Taddhita::iter().map(|t| t.as_str().to_string()).collect(),
    ),
    (
      "uṇādi suffixes",
      Unadi::iter().map(|un| un.as_str().to_string()).collect(),
    ),
    (
      "sanadi suffixes",
      Sanadi::iter().map(|s| s.as_str().to_string()).collect(),
    ),
    (
      "meter names",
      chandas
        .vrttas()
        .iter()
        .map(|v| v.name().to_string())
        .collect(),
    ),
    (
      "gaṇa names",
      all_sutras().map(|g| g.name().to_string()).collect(),
    ),
  ];
  let mut total_bad = 0;
  for (label, items) in sets.drain(..) {
    let bad: Vec<_> = items.iter().filter_map(|s| rom_checked(s).err()).collect();
    println!(
      "{:<18} {:>5} checked, {} lossy",
      label,
      items.len(),
      bad.len()
    );
    for b in bad.iter().take(10) {
      println!("    {b}");
    }
    total_bad += bad.len();
  }
  println!("\ntotal lossy: {total_bad}");
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example lipi_roman_audit
// dhātupāṭha roots    2229 checked, 0 lossy
// kṛt suffixes         122 checked, 0 lossy
// taddhita suffixes    175 checked, 0 lossy
// uṇādi suffixes       312 checked, 1 lossy
//     slp1=qau	deva=डउ	rom=ḍau	back_deva=डौ	back_slp1=qO
// sanadi suffixes        7 checked, 0 lossy
// meter names          145 checked, 0 lossy
// gaṇa names           262 checked, 0 lossy
//
// total lossy: 1
