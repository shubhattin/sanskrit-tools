//! vidyut-prakriya · `Vyakarana::derive_krdantas(&Krdanta) -> Vec<Prakriya>`
//! Build verbal derivatives (participles, gerunds, infinitives, agent nouns …) by
//! adding a kṛt pratyaya to a root. Also works with uṇādi suffixes (`Unadi::*`).
//! Result is a prātipadika (stem) or avyaya; decline it via `Subanta::new(krdanta, …)`.
//!
//! Run: cargo run --example prakriya_krdanta [-- "IAST root [gaṇa]" | "dhātupāṭha code"]
use vidyut_prakriya::Vyakarana;
use vidyut_prakriya::args::*;
use vidyut_test::{echo, rom};

fn main() {
  let v = Vyakarana::builder().log_steps(false).build();
  let root = vidyut_test::arg_or("kṛ 8");
  let (code, artha, dhatu) = vidyut_test::dhatu(&root);
  let show = |k: &Krdanta| {
    let mut f: Vec<_> = v
      .derive_krdantas(k)
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

  use BaseKrt::*;
  let krts = [
    (kta, "past passive participle"),
    (ktavatu, "past active participle"),
    (ktvA, "gerund (absolutive)"),
    (tumun, "infinitive"),
    (tavya, "gerundive"),
    (anIyar, "gerundive"),
    (yat, "gerundive"),
    (Ryat, "gerundive"),
    (tfc, "agent noun"),
    (Rvul, "agent noun"),
    (lyuw, "action noun"),
    (GaY, "action noun"),
    (ktin, "abstract fem. noun"),
  ];
  println!(
    "{} → dhātupāṭha {code} {} ({artha})",
    echo(&root),
    rom(dhatu.aupadeshika().unwrap_or(""))
  );
  for (krt, gloss) in krts {
    println!(
      "  {:<8} {:<24} {}",
      rom(krt.as_str()),
      gloss,
      show(&Krdanta::new(dhatu.clone(), krt))
    );
  }

  // Present participles need a lakara (śatṛ/śānac replace laṭ endings).
  for krt in [Satf, SAnac] {
    let k = Krdanta::builder()
      .dhatu(dhatu.clone())
      .krt(krt)
      .lakara(Lakara::Lat)
      .build()
      .unwrap();
    println!(
      "  {:<8} {:<24} {}",
      rom(krt.as_str()),
      "present participle",
      show(&k)
    );
  }

  // Decline a kṛdanta stem like any noun: kṛta → kṛtā (strī prathamā eka).
  let krta = Krdanta::new(dhatu.clone(), kta);
  let s = Subanta::new(krta, Linga::Stri, Vibhakti::Prathama, Vacana::Eka);
  let f: Vec<_> = v
    .derive_subantas(&s)
    .iter()
    .map(|p| rom(&p.text()))
    .collect();
  println!("\nkta-stem declined (strī prathamā eka): {}", f.join(" / "));
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example prakriya_krdanta
// kṛ 8 → dhātupāṭha 08.0010 ḍukṛ↓ñ (karaṇē)
//   kta      past passive participle  kṛta
//   ktavatum̐ past active participle   kṛtavat
//   ktvā     gerund (absolutive)      kṛtvā
//   tumum̐n  infinitive               kartum
//   tavya    gerundive                kartavya
//   anīyar   gerundive                karaṇīya
//   yat      gerundive                -
//   ṇyat     gerundive                kārya
//   tṛch     agent noun               kartṛ
//   ṇvum̐l   agent noun               kāraka
//   lyum̐ṭ   action noun              karaṇa
//   ghañ     action noun              kāra
//   ktin     abstract fem. noun       kṛti
//   śatṛm̐   present participle       kurvat
//   śānach   present participle       kurvāṇa
//
// kta-stem declined (strī prathamā eka): kṛtā
