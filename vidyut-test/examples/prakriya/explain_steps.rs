//! vidyut-prakriya · `Prakriya::history() -> &[Step]` (+ sūtra text from data/prakriya/*.tsv)
//! Every derived word carries its full prakriyā: which rule fired (`step.rule()`:
//! code + source text) and the resulting terms after it. Joined here with the
//! sūtra text from the shipped sutrapatha/varttika/dhatupatha TSVs.
//!
//! Run: cargo run --example prakriya_explain_steps [-- "IAST root [gaṇa]" | "dhātupāṭha code"]
use std::collections::HashMap;
use vidyut_prakriya::args::*;
use vidyut_prakriya::{Rule, Vyakarana};
use vidyut_test::{data_dir, iast};

fn load(file: &str) -> HashMap<String, String> {
  let text = std::fs::read_to_string(data_dir().join("prakriya").join(file)).unwrap();
  text
    .lines()
    .skip(1)
    .filter_map(|l| l.split_once('\t'))
    .map(|(k, v)| (k.into(), v.into()))
    .collect()
}

fn main() {
  let sutras = load("sutrapatha.tsv");
  let varttikas = load("varttikas.tsv");
  let unadi = load("unadipatha.tsv");
  let text_of = |r: &Rule| -> String {
    let t = match r {
      Rule::Ashtadhyayi(c) => sutras.get(*c),
      Rule::Varttika(c) => varttikas.get(*c),
      Rule::Unadipatha(c) => unadi.get(*c),
      _ => None,
    };
    t.map(|s| iast(s))
      .unwrap_or_else(|| format!("({})", r.source_name()))
  };

  let v = Vyakarana::new();
  let root = vidyut_test::arg_or("bhū");
  let (code, artha, dhatu) = vidyut_test::dhatu(&root);
  println!(
    "{root} → dhātupāṭha {code} {} ({artha})",
    iast(dhatu.aupadeshika().unwrap_or(""))
  );
  println!("laṭ kartari prathama eka:\n");
  let args = Tinanta::new(
    dhatu,
    Prayoga::Kartari,
    Lakara::Lat,
    Purusha::Prathama,
    Vacana::Eka,
  );

  for p in v.derive_tinantas(&args) {
    println!("=> {}\n", iast(&p.text()));
    for step in p.history() {
      let terms: Vec<_> = step
        .result()
        .iter()
        .map(|t| t.text())
        .filter(|t| !t.is_empty())
        .collect();
      println!(
        "{:<9} {:<26} {}",
        step.rule().code(),
        iast(&terms.join(" + ")),
        text_of(&step.rule())
      );
    }
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example prakriya_explain_steps
// bhū → dhātupāṭha 01.0001 bhū (sattāyām)
// laṭ kartari prathama eka:
//
// => bhavati
//
// 1.3.1     bhū                        bhūvādayo dhātavaḥ
// 3.2.123   bhū + lam̐ṭ                vartamāne laṭ
// 1.3.2     bhū + lam̐ṭ                upadeśe'janunāsika it
// 1.3.3     bhū + lam̐ṭ                halantyam
// 1.3.9     bhū + l                    tasya lopaḥ
// 1.3.78    bhū + l                    śeṣāt kartari parasmaipadam
// 3.4.78    bhū + tip                  tiptasjhisipthasthamibvasmas tātāṃjhathāsāthāṃdhvamiḍvahimahiṅ
// 1.3.3     bhū + tip                  halantyam
// 1.3.9     bhū + ti                   tasya lopaḥ
// 3.1.68    bhū + śap + ti             kartari śap
// 1.3.3     bhū + śap + ti             halantyam
// 1.3.8     bhū + śap + ti             laśakvataddhite
// 1.3.9     bhū + a + ti               tasya lopaḥ
// 3.4.113   bhū + a + ti               tiṅśitsārvadhātukam
// 3.4.113   bhū + a + ti               tiṅśitsārvadhātukam
// 1.4.13    bhū + a + ti               yasmāt pratyayavidhistadādi pratyaye'ṅgam
// 7.3.84    bho + a + ti               sārvadhātukārdhadhātukayoḥ
// 6.1.78    bhav + a + ti              eco'yavāyāvaḥ
// 1.4.14    bhav + a + ti              suptiṅantaṃ padam
// 8.4.68    bhav + a + ti              a a
