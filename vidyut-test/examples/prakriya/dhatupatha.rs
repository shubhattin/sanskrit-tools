//! vidyut-prakriya · `Dhatupatha::from_path(tsv)` / `.get(code)` / `.iter()` + `derive_dhatus`
//! Load the shipped Dhātupāṭha (data/prakriya/dhatupatha.tsv, ~2.2k roots). Roots are
//! keyed by code "GG.NNNN" (gaṇa.number) and stored in aupadeśika form (with it-markers
//! and accents, e.g. `eDa~\`); `derive_dhatus` gives the clean root after it-lopa.
//! This is the reliable way to get a correct `Dhatu` (right gaṇa/antargaṇa/pada).
//!
//! Run: cargo run --example prakriya_dhatupatha [-- "IAST clean root to search"]
use vidyut_prakriya::Vyakarana;
use vidyut_prakriya::dhatupatha::Dhatupatha;
use vidyut_test::{data_dir, iast, slp};

fn main() {
  let dp = Dhatupatha::from_path(data_dir().join("prakriya/dhatupatha.tsv")).unwrap();
  let v = Vyakarana::builder().log_steps(false).build();
  println!("{} roots loaded", dp.iter().count());

  let d = dp.get("01.0001").unwrap();
  println!(
    "get(\"01.0001\") -> aupadeśika {:?}, gaṇa {:?}\n",
    d.aupadeshika(),
    d.gana()
  );

  // Search by clean root: all homonymous entries across gaṇas, with meaning.
  let q = slp(&vidyut_test::arg_or("kṛ"));
  for e in dp.iter() {
    let clean: Vec<_> = v
      .derive_dhatus(e.dhatu())
      .iter()
      .map(|p| p.text())
      .collect();
    if clean.iter().any(|c| *c == q) {
      println!(
        "{}  {:<10} clean={:<6} gaṇa={:<8} artha: {}",
        e.code(),
        e.dhatu().aupadeshika().unwrap_or(""),
        iast(&clean[0]),
        format!("{:?}", e.dhatu().gana().unwrap()),
        iast(e.artha())
      );
    }
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example prakriya_dhatupatha
// 2229 roots loaded
// get("01.0001") -> aupadeśika Some("BU"), gaṇa Some(Bhvadi)
//
// 05.0007  kf\Y       clean=kṛ     gaṇa=Svadi    artha: hiṃsāyām
// 08.0010  qukf\Y     clean=kṛ     gaṇa=Tanadi   artha: karaṇe
