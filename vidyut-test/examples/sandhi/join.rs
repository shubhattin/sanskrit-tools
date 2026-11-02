//! vidyut-sandhi · `Rule::{first, second, result}` used FORWARD (joining words)
//! vidyut-sandhi has no `join()` API, but its rule table can be applied forward:
//! find rules where `first` is a suffix of word 1 and `second` a prefix of word 2.
//! Phonetic only (for morphology-aware joining use prakriya `derive_vakyas`).
//! The table covers vowel, visarga(s/r), m/n and t/k/p/w finals — e.g. no rule for
//! k + vowel, so `vāk + īśaḥ` stays unjoined (prakriya_vakya handles that).
//!
//! Run: cargo run --example sandhi_join [-- "IAST word1" "IAST word2"]
use std::collections::BTreeSet;
use vidyut_sandhi::generate_rules;
use vidyut_test::{echo, rom, slp};

fn join(rules: &[vidyut_sandhi::Rule], a: &str, b: &str) -> BTreeSet<String> {
  // The rule table writes word-final visarga as its underlying `s` (rAmas, not rAmaH).
  let a = &match a.strip_suffix('H') {
    Some(x) => format!("{x}s"),
    None => a.to_string(),
  };
  let mut out = BTreeSet::new();
  for r in rules {
    if a.ends_with(r.first()) && b.starts_with(r.second()) {
      let head = &a[..a.len() - r.first().len()];
      let tail = &b[r.second().len()..];
      out.insert(format!("{head}{}{tail}", r.result()));
    }
  }
  if out.is_empty() {
    out.insert(format!("{a} {b}")); // no rule fires
  }
  out
}

fn main() {
  let rules = generate_rules();
  let args: Vec<String> = std::env::args().skip(1).collect();
  let pairs: Vec<(String, String)> = if args.len() == 2 {
    vec![(args[0].clone(), args[1].clone())]
  } else {
    [
      ("ca", "iti"),
      ("deva", "indraḥ"),
      ("iti", "api"),
      ("rāmaḥ", "gacchati"),
      ("tat", "ca"),
      ("vāk", "īśaḥ"),
      ("guruḥ", "atra"),
      ("mahā", "ṛṣiḥ"),
    ]
    .iter()
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .collect()
  };
  for (a, b) in pairs {
    let res: Vec<_> = join(&rules, &slp(&a), &slp(&b))
      .iter()
      .map(|s| rom(s))
      .collect();
    println!("{} + {} -> {}", echo(&a), echo(&b), res.join(" | "));
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example sandhi_join
// cha + iti -> chēti
// dēva + indraḥ -> dēvēndraḥ
// iti + api -> ity api
// rāmaḥ + gachChati -> rāmaḥgachChati | rāmō gachChati
// tat + cha -> tach cha
// vāk + īśaḥ -> vāk īśaḥ
// guruḥ + atra -> gurutra | gurur atra
// mahā + ṛṣiḥ -> maharṣiḥ
