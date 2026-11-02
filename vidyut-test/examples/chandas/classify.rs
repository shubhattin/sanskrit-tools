//! vidyut-chandas · `Chandas::from_file(meters.tsv)` + `chandas.classify(verse) -> Match`
//! Identify the meter of a verse (SLP1, one pāda/half-verse per line). Returns the
//! best meter (vṛtta by syllable pattern, else jāti by mātrā count) and how well it
//! matched: Full / Pada (a pāda boundary matches) / Prefix / None.
//!
//! Run: cargo run --example chandas_classify [-- "IAST verse, lines separated by \n"]
use vidyut_chandas::Chandas;
use vidyut_test::{data_dir, echo, rom, slp};

fn main() {
  let chandas = Chandas::from_file(data_dir().join("chandas/meters.tsv")).unwrap();
  let verses: Vec<String> = match std::env::args().nth(1) {
        Some(s) => vec![s.replace("\\n", "\n")],
        None => vec![
            // anuṣṭubh (Gītā 1.1) — NOTE: anuṣṭubh is not in meters.tsv
            "dharmakṣetre kurukṣetre samavetā yuyutsavaḥ\nmāmakāḥ pāṇḍavāś caiva kim akurvata sañjaya".into(),
            // vasantatilakā, one pāda
            "mātaḥ samastajagatāṃ madhukaiṭabhāreḥ".into(),
            // mandākrāntā (Meghadūta 1)
            "kaścit kāntāvirahaguruṇā svādhikārāt pramattaḥ\nśāpenāstaṅgamitamahimā varṣabhogyeṇa bhartuḥ".into(),
            // śārdūlavikrīḍita (Sarasvatī vandanā)
            "yā kundendutuṣārahāradhavalā yā śubhravastrāvṛtā".into(),
            // junk text still gets *some* tiny meter (1–4 syllable meters exist), so check
            // match_type and length before trusting a result.
            "not a verse".into(),
        ],
    };
  for v in verses {
    let m = chandas.classify(slp(&v));
    let name = m
      .padya()
      .as_ref()
      .map(|p| rom(p.name()))
      .unwrap_or("-".into());
    println!(
      "{:<12} {:<8} | {}",
      name,
      format!("{:?}", m.match_type()),
      echo(&v).replace('\n', " / ")
    );
  }
}

// ==== SAMPLE OUTPUT ====
// $ cargo run --example chandas_classify
// -            None     | dharmakṣētrē kurukṣētrē samavētā yuyutsavaḥ / māmakāḥ pāṇḍavāś chaiva kim akurvata sañjaya
// vasantatilakā Pada     | mātaḥ samastajagatāṁ madhukaiṭabhārēḥ
// mandākrāntā  Pada     | kaśchit kāntāvirahaguruṇā svādhikārāt pramattaḥ / śāpēnāstaṅgamitamahimā varṣabhōgyēṇa bhartuḥ
// śārdulavikrīḍita Pada     | yā kundēndutuṣārahāradhavalā yā śubhravastrāvṛtā
// śrī          Full     | nōt a vērsē
