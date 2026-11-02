//! vidyut-prakriya · `Vyakarana::derive_taddhitantas(&Taddhitanta) -> Vec<Prakriya>`
//! Secondary derivation: add a taddhita suffix to a nominal stem (descendant-of,
//! possessing, abstract-ness …). Optionally pin a meaning with `TaddhitaArtha`.
//!
//! Run: cargo run --example prakriya_taddhitanta [-- "IAST stem"]
use vidyut_prakriya::Vyakarana;
use vidyut_prakriya::args::*;
use vidyut_test::{echo, rom, slp};

fn main() {
  let v = Vyakarana::builder().log_steps(false).build();
  let stem = vidyut_test::arg_or("vasudeva");
  let p = Pratipadika::basic(Slp1String::from(slp(&stem)).unwrap());
  let show = |t: &Taddhitanta| {
    let mut f: Vec<_> = v
      .derive_taddhitantas(t)
      .iter()
      .map(|p| rom(&p.text()))
      .collect();
    f.dedup();
    if f.is_empty() {
      "-".into()
    } else {
      f.join(" / ")
    }
  };

  use Taddhita::*;
  println!("{} + …", echo(&stem));
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
      rom(t.as_str()),
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
// vasudēva + …
//   aṇ     descendant / related to  vāsudēva
//   iñ     patronymic               vāsudēvi
//   ṭhak   dealing with             vāsudēvika
//   matum̐p possessing               vasudēvavat
//   tva    -ness (neut.)            vasudēvatva
//   tal    -ness (fem.)             vasudēvatā
//   tarap  comparative              vasudēvatara
//   tamap  superlative              vasudēvatama
//   kan    diminutive               vasudēvaka
//
// upagu + aR [artha = TasyaApatyam]: aupagava
