//! vidyut-prakriya · `Vyakarana::derive_taddhitantas(&Taddhitanta) -> Vec<Prakriya>`
//! Secondary derivation: add a taddhita suffix to a nominal stem (descendant-of,
//! possessing, abstract-ness …). Optionally pin a meaning with `TaddhitaArtha`.
//!
//! Run: cargo run --example prakriya_taddhitanta [-- "IAST stem"]
use vidyut_prakriya::Vyakarana;
use vidyut_prakriya::args::*;
use vidyut_test::{iast, slp};

fn main() {
  let v = Vyakarana::builder().log_steps(false).build();
  let stem = vidyut_test::arg_or("vasudeva");
  let p = Pratipadika::basic(Slp1String::from(slp(&stem)).unwrap());
  let show = |t: &Taddhitanta| {
    let mut f: Vec<_> = v
      .derive_taddhitantas(t)
      .iter()
      .map(|p| iast(&p.text()))
      .collect();
    f.dedup();
    if f.is_empty() {
      "-".into()
    } else {
      f.join(" / ")
    }
  };

  use Taddhita::*;
  println!("{stem} + …");
  for (t, gloss) in [
    (aR, "descendant / related to"),
    (iY, "patronymic"),
    (Wak, "dealing with"),
    (matup, "possessing"),
    (tva, "-ness (neut.)"),
    (tal, "-ness (fem.)"),
    (tarap, "comparative"),
    (tamap, "superlative"),
    (kan, "diminutive"),
  ] {
    println!(
      "  {:<6} {:<24} {}",
      t.as_str(),
      gloss,
      show(&Taddhitanta::new(p.clone(), t))
    );
  }

  // Pin a specific meaning: aR in the sense "tasya apatyam" (offspring) — upagu → aupagava.
  let upagu = Pratipadika::basic(Slp1String::from("upagu").unwrap());
  let t = Taddhitanta::builder()
    .pratipadika(upagu)
    .taddhita(aR)
    .artha(TaddhitaArtha::TasyaApatyam)
    .build()
    .unwrap();
  println!("\nupagu + aR [artha = TasyaApatyam]: {}", show(&t));
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example prakriya_taddhitanta
// vasudeva + …
//   aR     descendant / related to  vāsudeva
//   iY     patronymic               vāsudevi
//   Wak    dealing with             vāsudevika
//   matu~p possessing               vasudevavat
//   tva    -ness (neut.)            vasudevatva
//   tal    -ness (fem.)             vasudevatā
//   tarap  comparative              vasudevatara
//   tamap  superlative              vasudevatama
//   kan    diminutive               vasudevaka
//
// upagu + aR [artha = TasyaApatyam]: aupagava
