//! vidyut-kosha · `kosha.dhatus() -> impl Iterator<Item = DhatuEntry>`
//! Iterate every verb root in the kosha with its metadata: clean text, gaṇa,
//! and meanings in Sanskrit / English / Hindi (`artha_sa/en/hi`).
//!
//! ⚠ UPSTREAM BUG: `artha_hi` is garbage (S/A/D) in ALL 45,395 entries. In
//! vidyut-kosha/src/entries.rs, `DhatuMetaBuilder::karmatva()` writes to `self.artha_hi`
//! (copy-paste bug), so building the kosha overwrites the Hindi meaning with the karmatva
//! code (S = sakarmaka, A = akarmaka, D = dvikarmaka). `karmatva` itself is left empty.
//!
//! Run: cargo run --example kosha_dhatus [-- "IAST root to search, e.g. gam"]
use vidyut_kosha::Kosha;
use vidyut_test::{data_dir, rom, slp};

fn main() {
  let kosha = Kosha::new(data_dir().join("kosha")).unwrap();
  let all: Vec<_> = kosha.dhatus().collect();
  println!(
    "{} dhatu entries (includes sanadi/upasarga variants)\n",
    all.len()
  );

  let q = slp(&vidyut_test::arg_or("gam"));
  for d in all.iter().filter(|d| d.clean_text() == q) {
    let dh = d.dhatu();
    println!(
      "{} | aupadeśika={} gaṇa={} prefixes={:?} sanadi={:?}\n   sa: {}\n   en: {}\n   hi: {}",
      rom(d.clean_text()),
      dh.aupadeshika().map(rom).unwrap_or_default(),
      dh.gana().map(|g| format!("{g:?}")).unwrap_or("-".into()),
      dh.prefixes().iter().map(|p| rom(p)).collect::<Vec<_>>(),
      dh.sanadi(),
      d.artha_sa().map(rom).unwrap_or_default(),
      d.artha_en().unwrap_or("-"),
      d.artha_hi().unwrap_or("-"),
    );
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example kosha_dhatus
// 45395 dhatu entries (includes sanadi/upasarga variants)
//
// gam | aupadeśika=ga↓ml̥m̐ gaṇa=Bhvadi prefixes=[] sanadi=[]
//    sa: gatau
//    en: to go
//    hi: S
