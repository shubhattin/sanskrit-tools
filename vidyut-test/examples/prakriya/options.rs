//! vidyut-prakriya · `Vyakarana::builder()` options + `Prakriya::rule_choices()`
//! - `log_steps(false)`  faster, no history (word-list generation)
//! - `nlp_mode(true)`    keep final s/r instead of visarga (matches kosha keys)
//! - `use_svaras(true)`  run accent rules (output carries Vedic svara marks)
//! - `is_chandasi(true)` allow Vedic-only (chāndasa) rules
//! - `rule_choices(..)`  force an optional rule on/off; `p.rule_choices()` lists the
//!   optional rules each output depended on (why there are 2+ results).
//!
//! Run: cargo run --example prakriya_options
use vidyut_prakriya::args::*;
use vidyut_prakriya::{Decision, RuleChoice, Vyakarana};
use vidyut_test::rom;

fn rama_nom() -> Subanta {
  Subanta::new(
    Pratipadika::basic(Slp1String::from("rAma").unwrap()),
    Linga::Pum,
    Vibhakti::Prathama,
    Vacana::Eka,
  )
}

fn main() {
  let texts = |v: &Vyakarana, s: &Subanta| {
    v.derive_subantas(s)
      .iter()
      .map(|p| rom(&p.text()))
      .collect::<Vec<_>>()
  };
  println!(
    "default          : {:?}",
    texts(&Vyakarana::new(), &rama_nom())
  );
  println!(
    "nlp_mode(true)   : {:?}",
    texts(&Vyakarana::builder().nlp_mode(true).build(), &rama_nom())
  );

  // Svaras: output contains accent marks (SLP1 \ = anudātta, ^ = svarita).
  let agni = Subanta::new(
    Pratipadika::basic(Slp1String::from("agni").unwrap()),
    Linga::Pum,
    Vibhakti::Prathama,
    Vacana::Eka,
  );
  let sv = Vyakarana::builder().use_svaras(true).build();
  let raw: Vec<_> = sv.derive_subantas(&agni).iter().map(|p| p.text()).collect();
  let shown: Vec<_> = raw.iter().map(|t| rom(t)).collect();
  println!(
    "use_svaras(true) : agni -> {:?}  (/ marks the udātta)",
    shown
  );

  // Optional rules: guh (ūdit → optional iṭ by 7.2.44) gives several lṛṭ forms.
  // Show which optional rule each output depended on.
  let v = Vyakarana::new();
  let (_, _, dhatu) = vidyut_test::dhatu("guh");
  let t = Tinanta::new(
    dhatu.clone(),
    Prayoga::Kartari,
    Lakara::Lrt,
    Purusha::Prathama,
    Vacana::Eka,
  );
  println!("\nguh lṛṭ 3sg has {} results:", v.derive_tinantas(&t).len());
  for p in v.derive_tinantas(&t) {
    let ch: Vec<_> = p
      .rule_choices()
      .iter()
      .map(|c| format!("{}={:?}", c.rule.code(), c.decision))
      .collect();
    println!("  {:<12} choices: {}", rom(&p.text()), ch.join(", "));
  }

  // Force-decline the first optional rule → only the other branch remains.
  if let Some(first) = v
    .derive_tinantas(&t)
    .first()
    .and_then(|p| p.rule_choices().first().cloned())
  {
    let forced = Vyakarana::builder()
      .rule_choices(vec![RuleChoice {
        rule: first.rule,
        decision: Decision::Decline,
      }])
      .build();
    let f: Vec<_> = forced
      .derive_tinantas(&t)
      .iter()
      .map(|p| rom(&p.text()))
      .collect();
    println!("  forcing {} = Decline -> {:?}", first.rule.code(), f);
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example prakriya_options
// default          : ["rāmaḥ"]
// nlp_mode(true)   : ["rāmas"]
// use_svaras(true) : agni -> ["agni/ḥ"]  (/ marks the udātta)
//
// guh lṛṭ 3sg has 4 results:
//   ghōkṣyatē    choices: 1.3.72=Accept, 7.2.44=Accept
//   gūhiṣyatē    choices: 1.3.72=Accept, 7.2.44=Decline
//   ghōkṣyati    choices: 1.3.72=Decline, 7.2.44=Accept
//   gūhiṣyati    choices: 1.3.72=Decline, 7.2.44=Decline
//   forcing 1.3.72 = Decline -> ["ghōkṣyati", "gūhiṣyati"]
