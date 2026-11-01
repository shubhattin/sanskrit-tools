//! vidyut-lipi · `detect(text) -> Option<Scheme>`
//! Guess which script/romanization a string is written in (best effort,
//! based on characteristic characters). Useful for "type in any script" search boxes.
//!
//! Run: cargo run --example lipi_detect [-- "some text"]
use vidyut_lipi::detect;

fn main() {
  let inputs: Vec<String> = match std::env::args().nth(1) {
    Some(s) => vec![s],
    None => [
      "धर्मक्षेत्रे",       // Devanagari
      "dharmakṣetre",  // IAST
      "Darmakzetre",   // SLP1 (capital = aspirate, z = ṣ)
      "dharmakSetre",  // Harvard-Kyoto
      "dharmakShetre", // ITRANS
      "dharmak.setre", // Velthuis
      "ధర్మక్షేత్రే",       // Telugu
      "ಧರ್ಮಕ್ಷೇತ್ರೇ",       // Kannada
      "dharma",        // ambiguous plain ASCII
    ]
    .iter()
    .map(|s| s.to_string())
    .collect(),
  };
  for s in inputs {
    println!("{:<16} -> {:?}", s, detect(&s));
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example lipi_detect
// धर्मक्षेत्रे     -> Some(Devanagari)
// dharmakṣetre     -> Some(Iast)
// Darmakzetre      -> Some(Slp1)
// dharmakSetre     -> Some(HarvardKyoto)
// dharmakShetre    -> Some(Itrans)
// dharmak.setre    -> Some(Velthuis)
// ధర్మక్షేత్రే     -> Some(Telugu)
// ಧರ್ಮಕ್ಷೇತ್ರೇ     -> Some(Kannada)
// dharma           -> Some(HarvardKyoto)
