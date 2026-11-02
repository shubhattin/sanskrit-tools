//! vidyut-kosha · `kosha.pratipadikas() -> impl Iterator<Item = PratipadikaEntry>`
//! Iterate every nominal stem: plain stems (with their allowed genders) and
//! kṛdanta stems (root + kṛt suffix, e.g. gam + kta → gata).
//!
//! Run: cargo run --example kosha_pratipadikas [-- "IAST stem prefix"]
use vidyut_kosha::Kosha;
use vidyut_kosha::entries::PratipadikaEntry;
use vidyut_test::{data_dir, rom, slp};

fn main() {
  let kosha = Kosha::new(data_dir().join("kosha")).unwrap();
  let (mut basic, mut krdanta) = (0, 0);
  for p in kosha.pratipadikas() {
    match p {
      PratipadikaEntry::Basic(_) => basic += 1,
      PratipadikaEntry::Krdanta(_) => krdanta += 1,
    }
  }
  println!("{basic} basic stems, {krdanta} kṛdanta stems\n");

  let q = slp(&vidyut_test::arg_or("dharm"));
  println!("stems starting with '{}':", rom(&q));
  for p in kosha
    .pratipadikas()
    .filter(|p| p.lemma().starts_with(&q))
    .take(25)
  {
    let lingas: Vec<_> = p.lingas().iter().map(|l| format!("{l:?}")).collect();
    let kind = match &p {
      PratipadikaEntry::Basic(_) => "basic".to_string(),
      PratipadikaEntry::Krdanta(k) => format!(
        "kṛdanta {} + {}",
        rom(k.dhatu_text()),
        rom(k.krt().aupadeshika())
      ),
    };
    println!("  {:<16} {:<22} {}", rom(p.lemma()), lingas.join(","), kind);
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example kosha_pratipadikas
// 169328 basic stems, 1367307 kṛdanta stems
//
// stems starting with 'dharm':
//   dharma           Pum                    basic
//   dharmaṁdada      Pum,Stri,Napumsaka     basic
//   dharmaṁdadā      Stri                   basic
//   dharmaka         Pum,Stri,Napumsaka     basic
//   dharmakañchuka   Pum                    basic
//   dharmakañchukapravēśin Pum,Stri,Napumsaka     basic
//   dharmakathaka    Pum                    basic
//   dharmakathā      Stri                   basic
//   dharmakarōpādhyāya Pum                    basic
//   dharmakarman     Napumsaka              basic
//   dharmakāṅkṣiṇī   Stri                   basic
//   dharmakāṇḍa      Pum                    basic
//   dharmakāma       Pum,Stri,Napumsaka     basic
//   dharmakāmārtha   Pum                    basic
//   dharmakāmārthasaṁbaddha Pum,Stri,Napumsaka     basic
//   dharmakāmārthasambandha Pum                    basic
//   dharmakāya       Pum                    basic
//   dharmakāra       Pum                    basic
//   dharmakāraṇa     Napumsaka              basic
//   dharmakārya      Napumsaka              basic
//   dharmakāla       Pum                    basic
//   dharmakīrti      Pum                    basic
//   dharmakīla       Pum                    basic
//   dharmakūṭa       Pum                    basic
//   dharmakūpa       Pum                    basic
