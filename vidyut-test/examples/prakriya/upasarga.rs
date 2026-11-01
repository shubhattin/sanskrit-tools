//! vidyut-prakriya · `Dhatu::with_prefixes(&[upasarga…])`
//! Prefix a root with upasargas (pra, sam, ā, vi …). Prakriya applies the prefix
//! sandhi + ṇatva/ṣatva AND the pada change (e.g. sam+gam → ātmanepada).
//!
//! Run: cargo run --example prakriya_upasarga [-- "IAST root [gaṇa]" | "dhātupāṭha code"]
use vidyut_prakriya::Vyakarana;
use vidyut_prakriya::args::*;
use vidyut_test::iast;

fn main() {
  let v = Vyakarana::builder().log_steps(false).build();
  let root = vidyut_test::arg_or("gam");
  let (code, artha, base) = vidyut_test::dhatu(&root);
  println!(
    "{root} → dhātupāṭha {code} {} ({artha})",
    iast(base.aupadeshika().unwrap_or(""))
  );

  // Upasargas are given in SLP1 (A = ā).
  let sets: [&[&str]; 7] = [
    &[],
    &["pra"],
    &["sam"],
    &["AN"],
    &["ava"],
    &["ni"],
    &["upa", "sam"],
  ];
  for pre in sets {
    let d = base.clone().with_prefixes(pre);
    let pres = Tinanta::new(
      d.clone(),
      Prayoga::Kartari,
      Lakara::Lat,
      Purusha::Prathama,
      Vacana::Eka,
    );
    let ger = Krdanta::new(d.clone(), BaseKrt::ktvA); // ktvā becomes lyap after a prefix
    let f = |ps: Vec<vidyut_prakriya::Prakriya>| {
      ps.iter()
        .map(|p| iast(&p.text()))
        .collect::<Vec<_>>()
        .join("/")
    };
    println!(
      "{:<12} laṭ: {:<26} gerund: {}",
      if pre.is_empty() {
        "-".to_string()
      } else {
        pre.iter().map(|p| iast(p)).collect::<Vec<_>>().join("+")
      },
      f(v.derive_tinantas(&pres)),
      f(v.derive_krdantas(&ger)),
    );
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example prakriya_upasarga
// gam → dhātupāṭha 01.1137 ga॒mḷm̐ (gatau)
// -            laṭ: gacchati                   gerund: gatvā
// pra          laṭ: pragacchati                gerund: pragatya/pragamya
// sam          laṭ: saṃgacchate/saṅgacchate/saṃgacchati/saṅgacchati gerund: saṃgatya/saṅgatya/saṃgamya/saṅgamya
// āṅ           laṭ: āgacchati                  gerund: āgatya/āgamya
// ava          laṭ: avagacchati                gerund: avagatya/avagamya
// ni           laṭ: nigacchati                 gerund: nigatya/nigamya
// upa+sam      laṭ: upasaṃgacchate/upasaṅgacchate/upasaṃgacchati/upasaṅgacchati gerund: upasaṃgatya/upasaṅgatya/upasaṃgamya/upasaṅgamya
