//! vidyut-kosha · `Kosha::new(dir)` + `kosha.get_all(word) -> Vec<PadaEntry>`
//! Exact reverse lookup of an inflected form (~tens of millions of forms in a compact
//! FST): every (stem, gender, case, number) or (root, tense, person, number) analysis.
//! Keys are SLP1; a word-final visarga is stored as `s`/`r` (so look up `rAmas`, not `rAmaH`).
//!
//! Run: cargo run --example kosha_lookup [-- "IAST word"]
use vidyut_kosha::Kosha;
use vidyut_test::{data_dir, describe, iast, slp};

/// Kosha stores final visarga as s (or r): rAmaH -> rAmas / rAmar.
fn keys(word: &str) -> Vec<String> {
  match word.strip_suffix('H') {
    Some(stem) => vec![format!("{stem}s"), format!("{stem}r")],
    None => vec![word.to_string()],
  }
}

fn main() {
  let kosha = Kosha::new(data_dir().join("kosha")).unwrap();
  println!("kosha size: {} entries", kosha.len());
  let words: Vec<String> = match std::env::args().nth(1) {
    Some(s) => vec![s],
    None => [
      "bhavati",
      "rāmaḥ",
      "phalāni",
      "gatvā",
      "devebhyaḥ",
      "agacchat",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect(),
  };
  for w in words {
    println!("\n» {w}");
    // get_all can return exact duplicates (same analysis via different stems/paths).
    let mut seen = std::collections::BTreeSet::new();
    for key in keys(&slp(&w)) {
      for e in kosha.get_all(&key) {
        seen.insert(format!("[{}] {}", iast(&key), describe(&e)));
      }
    }
    for line in seen.iter().take(8) {
      println!("  {line}");
    }
    if seen.len() > 8 {
      println!("  … +{} more analyses", seen.len() - 8);
    }
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example kosha_lookup
// kosha size: 167774751 entries
//
// » bhavati
//   [bhavati] subanta  kṛdanta bhū + Satf~ (Lat)  Napumsaka Saptami Eka
//   [bhavati] subanta  kṛdanta bhū + Satf~ (Lat)  Pum Saptami Eka
//   [bhavati] subanta  prātipadika bhavat  Napumsaka Saptami Eka
//   [bhavati] subanta  prātipadika bhavat  Pum Saptami Eka
//   [bhavati] subanta  prātipadika bhavat  Stri Sambodhana Eka
//   [bhavati] subanta  prātipadika bhavatī  Napumsaka Dvitiya Eka
//   [bhavati] subanta  prātipadika bhavatī  Napumsaka Prathama Eka
//   [bhavati] subanta  prātipadika bhavatī  Napumsaka Sambodhana Eka
//   … +4 more analyses
//
// » rāmaḥ
//   [rāmas] subanta  kṛdanta ram + GaY  Pum Prathama Eka
//   [rāmas] subanta  kṛdanta ram + Ra  Pum Prathama Eka
//   [rāmas] subanta  kṛdanta ram + kvi~p  Napumsaka Panchami Eka
//   [rāmas] subanta  kṛdanta ram + kvi~p  Napumsaka Sasthi Eka
//   [rāmas] subanta  kṛdanta ram + kvi~p  Pum Dvitiya Bahu
//   [rāmas] subanta  kṛdanta ram + kvi~p  Pum Panchami Eka
//   [rāmas] subanta  kṛdanta ram + kvi~p  Pum Prathama Bahu
//   [rāmas] subanta  kṛdanta ram + kvi~p  Pum Sambodhana Bahu
//   … +8 more analyses
//
// » phalāni
//   [phalāni] subanta  kṛdanta phal + Ga  Napumsaka Dvitiya Bahu
//   [phalāni] subanta  kṛdanta phal + Ga  Napumsaka Prathama Bahu
//   [phalāni] subanta  kṛdanta phal + Ga  Napumsaka Sambodhana Bahu
//   [phalāni] subanta  kṛdanta phal + ac  Napumsaka Dvitiya Bahu
//   [phalāni] subanta  kṛdanta phal + ac  Napumsaka Prathama Bahu
//   [phalāni] subanta  kṛdanta phal + ac  Napumsaka Sambodhana Bahu
//   [phalāni] subanta  prātipadika phala  Napumsaka Dvitiya Bahu
//   [phalāni] subanta  prātipadika phala  Napumsaka Prathama Bahu
//   … +2 more analyses
//
// » gatvā
//   [gatvā] avyaya   kṛdanta gam + ktvA
//
// » devebhyaḥ
//   [devebhyas] subanta  kṛdanta dev + Ga  Napumsaka Caturthi Bahu
//   [devebhyas] subanta  kṛdanta dev + Ga  Napumsaka Panchami Bahu
//   [devebhyas] subanta  kṛdanta dev + Ga  Pum Caturthi Bahu
//   [devebhyas] subanta  kṛdanta dev + Ga  Pum Panchami Bahu
//   [devebhyas] subanta  kṛdanta dev + GaY  Napumsaka Caturthi Bahu
//   [devebhyas] subanta  kṛdanta dev + GaY  Napumsaka Panchami Bahu
//   [devebhyas] subanta  kṛdanta dev + GaY  Pum Caturthi Bahu
//   [devebhyas] subanta  kṛdanta dev + GaY  Pum Panchami Bahu
//   … +28 more analyses
//
// » agacchat
//   [agacchat] tiṅanta  gam (gaṇa Bhvadi)  Lan Kartari Prathama Eka
