//! vidyut-prakriya · `Vyakarana::derive_tinantas(&Tinanta) -> Vec<Prakriya>`
//! Conjugate a verb root: every lakāra (tense/mood) × puruṣa × vacana. Each call can
//! return several valid forms (optional rules), so cells may hold "a / b".
//!
//! Run: cargo run --example prakriya_tinanta [-- "IAST root [gaṇa]" | "dhātupāṭha code"]
use vidyut_prakriya::Vyakarana;
use vidyut_prakriya::args::*;
use vidyut_test::iast;

fn forms(v: &Vyakarana, t: &Tinanta) -> String {
  let mut out: Vec<_> = v
    .derive_tinantas(t)
    .iter()
    .map(|p| iast(&p.text()))
    .collect();
  out.dedup();
  out.join(" / ")
}

fn main() {
  let v = Vyakarana::builder().log_steps(false).build(); // faster: we only need final text
  let root = vidyut_test::arg_or("bhū");
  let (code, artha, dhatu) = vidyut_test::dhatu(&root);
  println!(
    "{root} → dhātupāṭha {code} {} ({artha})",
    iast(dhatu.aupadeshika().unwrap_or(""))
  );
  println!("kartari prayoga; columns = Eka | Dvi | Bahu\n");

  for lakara in Lakara::iter() {
    println!("{}", format!("{lakara:?}"));
    for purusha in Purusha::iter() {
      let row: Vec<_> = Vacana::iter()
        .map(|vacana| {
          forms(
            &v,
            &Tinanta::new(dhatu.clone(), Prayoga::Kartari, lakara, purusha, vacana),
          )
        })
        .collect();
      println!("  {:<9} {}", format!("{purusha:?}"), row.join("  |  "));
    }
  }

  // Same root in karmaṇi / bhāve prayoga (passive / impersonal), laṭ prathama eka.
  for prayoga in [Prayoga::Karmani, Prayoga::Bhave] {
    let t = Tinanta::new(
      dhatu.clone(),
      prayoga,
      Lakara::Lat,
      Purusha::Prathama,
      Vacana::Eka,
    );
    println!("\n{} laṭ 3sg: {}", format!("{prayoga:?}"), forms(&v, &t));
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example prakriya_tinanta
// bhū → dhātupāṭha 01.0001 bhū (sattāyām)
// kartari prayoga; columns = Eka | Dvi | Bahu
//
// Lat
//   Prathama  bhavati  |  bhavataḥ  |  bhavanti
//   Madhyama  bhavasi  |  bhavathaḥ  |  bhavatha
//   Uttama    bhavāmi  |  bhavāvaḥ  |  bhavāmaḥ
// Lit
//   Prathama  babhūva  |  babhūvatuḥ  |  babhūvuḥ
//   Madhyama  babhūvitha  |  babhūvathuḥ  |  babhūva
//   Uttama    babhūva  |  babhūviva  |  babhūvima
// Lut
//   Prathama  bhavitā  |  bhavitārau  |  bhavitāraḥ
//   Madhyama  bhavitāsi  |  bhavitāsthaḥ  |  bhavitāstha
//   Uttama    bhavitāsmi  |  bhavitāsvaḥ  |  bhavitāsmaḥ
// Lrt
//   Prathama  bhaviṣyati  |  bhaviṣyataḥ  |  bhaviṣyanti
//   Madhyama  bhaviṣyasi  |  bhaviṣyathaḥ  |  bhaviṣyatha
//   Uttama    bhaviṣyāmi  |  bhaviṣyāvaḥ  |  bhaviṣyāmaḥ
// Let
//   Prathama  bhavāt / bhavād / bhavāti  |  bhavātaḥ  |  bhavān / bhavānti
//   Madhyama  bhavāḥ / bhavāsi  |  bhavāthaḥ  |  bhavātha
//   Uttama    bhavā / bhavāni  |  bhavāva / bhavāvaḥ  |  bhavāma / bhavāmaḥ
// Lot
//   Prathama  bhavatāt / bhavatād / bhavatu  |  bhavatām  |  bhavantu
//   Madhyama  bhavatāt / bhavatād / bhava  |  bhavatam  |  bhavata
//   Uttama    bhavāni  |  bhavāva  |  bhavāma
// Lan
//   Prathama  abhavat / abhavad  |  abhavatām  |  abhavan
//   Madhyama  abhavaḥ  |  abhavatam  |  abhavata
//   Uttama    abhavam  |  abhavāva  |  abhavāma
// VidhiLin
//   Prathama  bhavet / bhaved  |  bhavetām  |  bhaveyuḥ
//   Madhyama  bhaveḥ  |  bhavetam  |  bhaveta
//   Uttama    bhaveyam  |  bhaveva  |  bhavema
// AshirLin
//   Prathama  bhūyāt / bhūyād  |  bhūyāstām  |  bhūyāsuḥ
//   Madhyama  bhūyāḥ  |  bhūyāstam  |  bhūyāsta
//   Uttama    bhūyāsam  |  bhūyāsva  |  bhūyāsma
// Lun
//   Prathama  abhūt / abhūd  |  abhūtām  |  abhūvan
//   Madhyama  abhūḥ  |  abhūtam  |  abhūta
//   Uttama    abhūvam  |  abhūva  |  abhūma
// Lrn
//   Prathama  abhaviṣyat / abhaviṣyad  |  abhaviṣyatām  |  abhaviṣyan
//   Madhyama  abhaviṣyaḥ  |  abhaviṣyatam  |  abhaviṣyata
//   Uttama    abhaviṣyam  |  abhaviṣyāva  |  abhaviṣyāma
//
// Karmani laṭ 3sg: bhūyate
//
// Bhave laṭ 3sg: bhūyate
