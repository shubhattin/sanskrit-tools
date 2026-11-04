//! vidyut's kosha (every form its grammar generates, with its analysis) and cheda (a statistical
//! segmenter over it).
//!
//! The data (Ambuda's py-0.4.0 release, ~81 MB) is embedded as zstd copies (~23 MB, see
//! `scripts/vidyut-data.sh`) and decompressed on first use: a few hundred ms and ~250 MB of
//! memory, paid once per process and only by requests that need it.

use super::prakriya::{
  LakaraId, LingaId, PrayogaId, PurushaId, StemArgs, VacanaId, VibhaktiId, WordSpec, dhatu_args,
  dhatu_label, krdanta_stem, suffix_label,
};
use super::to_deva;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::sync::LazyLock;
use utoipa::ToSchema;
use vidyut_cheda::{Chedaka, Model};
use vidyut_kosha::Kosha;
use vidyut_kosha::entries::{PadaEntry, PratipadikaEntry};
use vidyut_prakriya::args::{BaseKrt, Krdanta, Krt, Linga};

const PADAS: &[u8] = include_bytes!("data/kosha/padas.fst.zst");
const REGISTRY: &[u8] = include_bytes!("data/kosha/registry.msgpack.zst");
const MODEL: &[u8] = include_bytes!("data/cheda/model.msgpack.zst");

fn unzstd(bytes: &[u8]) -> Vec<u8> {
  let mut out = Vec::new();
  ruzstd::decoding::StreamingDecoder::new(bytes)
    .expect("embedded zstd data has a valid header")
    .read_to_end(&mut out)
    .expect("embedded zstd data decodes");
  out
}

static CHEDAKA: LazyLock<Chedaka> = LazyLock::new(|| {
  // The three files decode in parallel; the FST is the big one.
  let (padas, registry, model) = std::thread::scope(|s| {
    let padas = s.spawn(|| unzstd(PADAS));
    let model = s.spawn(|| Model::from_bytes(&unzstd(MODEL)).expect("embedded cheda model loads"));
    let registry = unzstd(REGISTRY);
    (padas.join().unwrap(), registry, model.join().unwrap())
  });
  let kosha = Kosha::from_bytes(padas, &registry).expect("embedded kosha loads");
  Chedaka::from_parts(super::sandhi::splitter().clone(), kosha, model)
});

/// Loads the data now (blocking; call from `spawn_blocking` in async code).
pub fn load() {
  LazyLock::force(&CHEDAKA);
}

fn kosha() -> &'static Kosha {
  CHEDAKA.kosha()
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PadaKind {
  Subanta,
  Avyaya,
  Tinanta,
}

/// One grammatical reading of a word form.
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, PartialEq)]
pub struct PadaInfo {
  pub kind: PadaKind,
  /// Prātipadika (subanta, avyaya) or dhātu (tiṅanta), Devanagari.
  pub lemma: String,
  /// Kṛdanta stems: the root and kṛt pratyaya (उद्भू, ल्युट्).
  pub dhatu: Option<String>,
  pub krt: Option<String>,
  pub linga: Option<LingaId>,
  pub vibhakti: Option<VibhaktiId>,
  pub vacana: Option<VacanaId>,
  pub purusha: Option<PurushaId>,
  pub lakara: Option<LakaraId>,
  pub prayoga: Option<PrayogaId>,
  /// One-line summary (`उद्भवन (उद्भू + ल्युट्) napumsaka prathama eka`): a stable key for the
  /// analysis (Jev's option label) and the agent's view of it.
  pub key: String,
  /// The word to derive (`/derive`) or open in the generators; none for plain indeclinables and
  /// roots outside the Dhātupāṭha.
  pub word: Option<WordSpec>,
  /// Grammatical but rarely meant (ranked last).
  #[serde(skip)]
  rare: bool,
}

impl PadaInfo {
  fn new(kind: PadaKind, lemma: String) -> Self {
    PadaInfo {
      kind,
      lemma,
      key: String::new(),
      word: None,
      rare: false,
      dhatu: None,
      krt: None,
      linga: None,
      vibhakti: None,
      vacana: None,
      purusha: None,
      lakara: None,
      prayoga: None,
    }
  }

  fn summary(&self) -> String {
    fn name<T: Serialize>(v: &Option<T>) -> Option<String> {
      serde_json::to_value(v.as_ref()?)
        .ok()?
        .as_str()
        .map(str::to_string)
    }
    let mut out = self.lemma.clone();
    if let (Some(d), Some(k)) = (&self.dhatu, &self.krt) {
      out += &format!(" ({d} + {k})");
    }
    let tags: Vec<String> = match self.kind {
      PadaKind::Avyaya => vec!["avyaya".into()],
      PadaKind::Tinanta => [
        name(&self.lakara),
        name(&self.purusha),
        name(&self.vacana),
        name(&self.prayoga),
      ]
      .into_iter()
      .flatten()
      .collect(),
      PadaKind::Subanta => [name(&self.linga), name(&self.vibhakti), name(&self.vacana)]
        .into_iter()
        .flatten()
        .collect(),
    };
    format!("{out} {}", tags.join(" "))
  }

  /// Vocative singular: also how a bare stem (a compound's first member) reads.
  pub fn is_bare_stem(&self) -> bool {
    self.vibhakti == Some(VibhaktiId::Sambodhana) && self.vacana == Some(VacanaId::Eka)
  }
}

fn info(e: &PadaEntry) -> PadaInfo {
  let mut out = match e {
    PadaEntry::Tinanta(t) => PadaInfo {
      purusha: PurushaId::of(t.purusha()),
      vacana: VacanaId::of(t.vacana()),
      lakara: LakaraId::of(t.lakara()),
      prayoga: PrayogaId::of(t.prayoga()),
      ..PadaInfo::new(
        PadaKind::Tinanta,
        dhatu_label(t.dhatu()).unwrap_or_else(|| to_deva(t.dhatu_text())),
      )
    },
    PadaEntry::Subanta(s) => {
      let p = s.pratipadika_entry();
      let mut out = match p {
        PratipadikaEntry::Basic(b) => {
          PadaInfo::new(PadaKind::Subanta, to_deva(b.pratipadika().text()))
        }
        PratipadikaEntry::Krdanta(k) => {
          let stem = krdanta_stem(&Krdanta::from(k)).unwrap_or_else(|| k.dhatu_text().to_string());
          PadaInfo {
            dhatu: Some(dhatu_label(k.dhatu()).unwrap_or_else(|| to_deva(k.dhatu_text()))),
            krt: Some(suffix_label(k.krt().as_str())),
            ..PadaInfo::new(PadaKind::Subanta, to_deva(&stem))
          }
        }
      };
      if let PratipadikaEntry::Krdanta(k) = p {
        out.rare = !s.is_avyaya() && !usual_linga(k.krt(), s.linga());
      }
      if s.is_avyaya() {
        out.kind = PadaKind::Avyaya;
      } else {
        out.linga = LingaId::of(s.linga());
        out.vibhakti = VibhaktiId::of(s.vibhakti());
        out.vacana = VacanaId::of(s.vacana());
      }
      out
    }
  };
  out.key = out.summary();
  out.word = word_spec(e);
  out
}

fn stem_args(p: &PratipadikaEntry) -> Option<StemArgs> {
  match p {
    PratipadikaEntry::Basic(b) => Some(StemArgs::Basic {
      stem: to_deva(b.pratipadika().text()),
    }),
    PratipadikaEntry::Krdanta(k) => match k.krt() {
      Krt::Base(krt) => Some(StemArgs::Krdanta {
        dhatu: dhatu_args(k.dhatu())?,
        krt: krt.as_str().to_string(),
        lakara: k.lakara().and_then(LakaraId::of),
        prayoga: k.prayoga().and_then(PrayogaId::of),
      }),
      Krt::Unadi(_) => None,
    },
  }
}

/// The analysis as a derivable word.
fn word_spec(e: &PadaEntry) -> Option<WordSpec> {
  match e {
    PadaEntry::Tinanta(t) => Some(WordSpec::Tinanta {
      dhatu: dhatu_args(t.dhatu())?,
      prayoga: PrayogaId::of(t.prayoga())?,
      lakara: LakaraId::of(t.lakara())?,
      purusha: PurushaId::of(t.purusha())?,
      vacana: VacanaId::of(t.vacana())?,
      pada: None,
    }),
    // Kṛdanta indeclinables (गत्वा, गन्तुम्) derive as stems; plain ones (च, तथा) have nothing to
    // derive.
    PadaEntry::Subanta(s) if s.is_avyaya() => match s.pratipadika_entry() {
      p @ PratipadikaEntry::Krdanta(_) => Some(WordSpec::Stem {
        stem: stem_args(p)?,
      }),
      PratipadikaEntry::Basic(_) => None,
    },
    PadaEntry::Subanta(s) => Some(WordSpec::Subanta {
      stem: stem_args(s.pratipadika_entry())?,
      linga: LingaId::of(s.linga())?,
      vibhakti: VibhaktiId::of(s.vibhakti())?,
      vacana: VacanaId::of(s.vacana())?,
    }),
  }
}

/// Whether a kṛdanta usually takes this gender: action nouns in ल्युट् are neuter, in घञ् masculine,
/// in क्तिन् feminine (vidyut lists genders for a few suffixes only).
fn usual_linga(krt: Krt, linga: Linga) -> bool {
  let Krt::Base(b) = krt else { return true };
  let listed = b.lingas();
  if !listed.is_empty() {
    return listed.contains(&linga);
  }
  match b {
    BaseKrt::lyuw => linga == Linga::Napumsaka,
    BaseKrt::GaY => linga == Linga::Pum,
    _ => true,
  }
}

/// Unlikely readings: a neuter of a long-vowel stem (दत्ता → दत्त by 1.2.47: grammatical, but
/// almost always the short-stem word is meant) or a kvip noun from a root (राम् ← रम्).
fn unlikely(i: &PadaInfo) -> bool {
  let long_neuter = i.linga == Some(LingaId::Napumsaka)
    && i.krt.is_none()
    && ['ा', 'ी', 'ू'].iter().any(|c| i.lemma.ends_with(*c));
  i.rare || long_neuter || i.krt.as_deref() == Some("क्विप्")
}

/// SLP1 key as the kosha stores it: a final visarga is written `s` (or `r`), as before a pause
/// the tables use.
fn kosha_keys(slp1: &str) -> Vec<String> {
  match slp1.strip_suffix('H') {
    Some(stem) if !stem.is_empty() => {
      vec![format!("{stem}s"), format!("{stem}r"), slp1.to_string()]
    }
    _ => vec![slp1.to_string()],
  }
}

/// Whether the kosha has this form (SLP1 letters).
pub fn contains(slp1: &str) -> bool {
  let k = kosha();
  kosha_keys(slp1).iter().any(|key| k.contains_key(key))
}

const MAX_ANALYSES: usize = 6;

/// Analyses of a form (SLP1 letters), most useful first (vocatives last), without duplicates.
pub fn analyses(slp1: &str) -> Vec<PadaInfo> {
  let k = kosha();
  let mut out: Vec<PadaInfo> = Vec::new();
  for key in kosha_keys(slp1) {
    for e in k.get_all(&key) {
      let i = info(&e);
      // Same reading from homonymous roots: listed once.
      if !out.iter().any(|o| o.key == i.key) {
        out.push(i);
      }
    }
  }
  // Plain stems before kṛdantas (rāmaḥ: rāma, not ram + kvip), unlikely ones and vocatives last.
  out.sort_by_key(|i| {
    (
      i.vibhakti == Some(VibhaktiId::Sambodhana),
      unlikely(i),
      i.krt.is_some(),
    )
  });
  out.truncate(MAX_ANALYSES);
  out
}

/// A word of cheda's segmentation (SLP1) and the analysis it chose.
pub struct ChedaWord {
  pub text: String,
  pub pada: Option<PadaInfo>,
}

/// cheda's best segmentation of one chunk (SLP1, no spaces). `None` when it fails.
pub fn segment(chunk: &str) -> Option<Vec<ChedaWord>> {
  let tokens = CHEDAKA.run(chunk).ok()?;
  if tokens.is_empty() {
    return None;
  }
  Some(
    tokens
      .iter()
      .map(|t| ChedaWord {
        text: t.text().to_string(),
        pada: t.data().as_ref().map(info),
      })
      .collect(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn kosha_and_cheda() {
    let t = std::time::Instant::now();
    load();
    eprintln!("kosha + cheda loaded in {:?}", t.elapsed());
    for w in [
      "rAmaH",
      "rAmas",
      "gacCati",
      "udBavanam",
      "tTA",
      "taTA",
      "datta",
    ] {
      eprintln!("{w}: contains={} {:#?}", contains(w), analyses(w));
    }
    assert!(contains("gacCati"));
    let causative = analyses("gamayizyati");
    // गमयिष्यति: गम् + णिच्, or इ + णिच् (2.4.46 इणो गमिरबोधने).
    assert!(
      causative.iter().any(|a| a.lemma == "गम् + णिच्"),
      "{causative:?}"
    );
    assert!(contains("udBavanam"));
    let a = analyses("udBavanam");
    assert!(
      a.iter()
        .any(|p| p.lemma == "उद्भवन" && p.krt.as_deref() == Some("ल्युट्")),
      "{a:?}"
    );
    let words: Vec<String> = segment("dattodBavanam")
      .unwrap()
      .into_iter()
      .map(|w| w.text)
      .collect();
    eprintln!("dattodBavanam: {words:?}");
    let datta = analyses("datta");
    // दत्त: "give!" (dā, loṭ 2pl) or the stem (vocative / compound member); not neuter दत्ता first.
    assert_ne!(datta[0].lemma, "दत्ता", "{datta:?}");
    assert!(
      datta.iter().any(|a| a.krt.as_deref() == Some("क्त")),
      "{datta:?}"
    );
    let gacchati = analyses("gacCati");
    assert!(
      gacchati.iter().any(
        |a| matches!(&a.word, Some(WordSpec::Tinanta { dhatu, .. }) if dhatu.code == "01.1137")
      ),
      "{gacchati:?}"
    );
    assert_eq!(words.last().map(String::as_str), Some("udBavanam"));
    let tathaiva = segment("taTEva").unwrap();
    assert!(
      tathaiva
        .iter()
        .all(|w| w.pada.as_ref().is_some_and(|p| p.kind == PadaKind::Avyaya))
    );
  }
}
