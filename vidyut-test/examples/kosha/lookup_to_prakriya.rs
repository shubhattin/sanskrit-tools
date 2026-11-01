//! vidyut-kosha + vidyut-prakriya · `Pada::from(PadaEntry)` → `Vyakarana::derive_*`
//! Analyse a word with the kosha, convert the entry back into prakriya arguments,
//! and re-derive it to get the full Pāṇinian derivation (which sūtras made it).
//!
//! Run: cargo run --example kosha_lookup_to_prakriya [-- "IAST word"]
use vidyut_kosha::Kosha;
use vidyut_prakriya::Vyakarana;
use vidyut_prakriya::args::Pada;
use vidyut_test::{data_dir, describe, iast, slp};

fn main() {
  let kosha = Kosha::new(data_dir().join("kosha")).unwrap();
  let v = Vyakarana::new();
  let word = slp(&vidyut_test::arg_or("jagāma"));

  let entries = kosha.get_all(&word);
  println!("{} analyses of '{}':", entries.len(), iast(&word));
  for e in &entries {
    println!("  {}", describe(e));
  }

  // Derive the first analysis back through the grammar.
  let Some(entry) = entries.into_iter().next() else {
    return;
  };
  let prakriyas = match Pada::from(entry) {
    Pada::Tinanta(t) => v.derive_tinantas(&t),
    Pada::Subanta(s) => v.derive_subantas(&s),
    _ => vec![],
  };
  // A spec may yield several valid forms; show the derivation that produced our word.
  if let Some(p) = prakriyas.iter().find(|p| p.text() == word) {
    println!(
      "
prakriyā of the first analysis:"
    );
    for step in p.history() {
      let terms: Vec<_> = step
        .result()
        .iter()
        .map(|t| t.text())
        .filter(|t| !t.is_empty())
        .collect();
      println!("   {:<10} {}", step.rule().code(), iast(&terms.join(" + ")));
    }
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example kosha_lookup_to_prakriya
// 2 analyses of 'jagāma':
//   tiṅanta  gam (gaṇa Bhvadi)  Lit Kartari Prathama Eka
//   tiṅanta  gam (gaṇa Bhvadi)  Lit Kartari Uttama Eka
//
// prakriyā of the first analysis:
//    1.3.1      ga॒mḷm̐
//    1.3.2      ga॒mḷm̐
//    1.3.9      gam
//    01.0934    gam
//    3.2.115    gam + lim̐ṭ
//    1.3.2      gam + lim̐ṭ
//    1.3.3      gam + lim̐ṭ
//    1.3.9      gam + l
//    1.3.78     gam + l
//    3.4.78     gam + tip
//    1.3.3      gam + tip
//    1.3.9      gam + ti
//    3.4.82     gam + ṇal
//    1.3.3      gam + ṇal
//    1.3.7      gam + ṇal
//    1.3.9      gam + a
//    3.4.115    gam + a
//    6.1.8      gam + gam + a
//    6.1.4      gam + gam + a
//    6.1.5      gam + gam + a
//    1.4.13     gam + gam + a
//    7.2.116    gam + gām + a
//    7.4.60     ga + gām + a
//    7.4.62     ja + gām + a
//    1.4.14     ja + gām + a
//    8.4.68     ja + gām + a
