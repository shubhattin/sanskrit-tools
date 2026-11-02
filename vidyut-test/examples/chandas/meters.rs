//! vidyut-chandas · `chandas.vrttas()` / `chandas.jatis()` (the meter catalogue)
//! Inspect the meter database: each vṛtta's pādas as L/G weights + gaṇas
//! (ya-ma-tā-rā-ja-bhā-na-sa) and each jāti's mātrā counts.
//!
//! Run: cargo run --example chandas_meters [-- "meter name substring (SLP1)"]
use vidyut_chandas::Chandas;
use vidyut_test::{data_dir, rom};

fn main() {
  let chandas = Chandas::from_file(data_dir().join("chandas/meters.tsv")).unwrap();
  println!(
    "{} vṛttas, {} jātis\n",
    chandas.vrttas().len(),
    chandas.jatis().len()
  );
  let q = std::env::args().nth(1);
  let wanted = [
    "indravajrA",
    "vasantatilakA",
    "mandAkrAntA",
    "SArdulavikrIqita",
  ];
  for v in chandas.vrttas() {
    let hit = match &q {
      Some(q) => v.name().contains(q.as_str()),
      None => wanted.contains(&v.name()),
    };
    if !hit {
      continue;
    }
    println!("{}", rom(v.name()));
    for (i, p) in v.padas().iter().enumerate() {
      let w: String = p.weights().iter().map(|w| w.as_str()).collect();
      println!(
        "  pāda {}: {:<22} gaṇas={:?} yati={:?}",
        i + 1,
        w,
        p.ganas(),
        p.yati()
      );
    }
  }
  println!("\njātis:");
  for j in chandas.jatis() {
    println!("  {:<12} mātrās per pāda {:?}", rom(j.name()), j.matras());
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example chandas_meters
// 145 vṛttas, 7 jātis
//
// indravajrā
//   pāda 1: GGLGGLLGLGG            gaṇas=[Ta, Ta, Ja, Ga, Ga] yati=[]
// vasantatilakā
//   pāda 1: GGLGLLLGLLGLGG         gaṇas=[Ta, Bha, Ja, Ja, Ga, Ga] yati=[]
// mandākrāntā
//   pāda 1: GGGGLLLLLGGLGGLGG      gaṇas=[Ma, Bha, Na, Ta, Ta, Ga, Ga] yati=[4, 10]
// śārdulavikrīḍita
//   pāda 1: GGGLLGLGLLLGGGLGGLG    gaṇas=[Ma, Sa, Ja, Sa, Ta, Ta, Ga] yati=[12]
//
// jātis:
//   vaitālīyam   mātrās per pāda [14, 16, 14, 16]
//   upagīti      mātrās per pāda [12, 15, 12, 15]
//   āryāgīti     mātrās per pāda [12, 20, 12, 20]
//   gīti         mātrās per pāda [12, 18, 12, 18]
//   udgīti       mātrās per pāda [12, 15, 12, 18]
//   aupachChandasikam mātrās per pāda [16, 18, 16, 18]
//   āryā         mātrās per pāda [12, 18, 12, 15]
