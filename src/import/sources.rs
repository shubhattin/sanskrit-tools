//! Registry of works + per-source parsers. Each parser returns records in book
//! order; the DB layer assigns `seq` by position.

use super::markup::{self, Opts, escape, text_to_html, xref};
use super::stardict;
use crate::translit::{deva_key, has_devanagari, iast_key};
use anyhow::{Context, Result};
use regex::Regex;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyKind {
  Headword,
  Pratipadika,
  Variant,
  Syn,
}

impl KeyKind {
  pub fn as_str(self) -> &'static str {
    match self {
      KeyKind::Headword => "headword",
      KeyKind::Pratipadika => "pratipadika",
      KeyKind::Variant => "variant",
      KeyKind::Syn => "syn",
    }
  }
}

#[derive(Default, Debug)]
pub struct Rec {
  pub headword: String,
  pub senses: Vec<String>,
  pub data: Option<Value>,
  pub sutra_id: Option<String>,
  /// Set directly for the Dhātupāṭha itself (index into parsed dhatus)
  pub dhatu_index: Option<usize>,
  /// Try to link to a unique Dhātupāṭha root with this SLP1 key
  pub dhatu_root_key: Option<String>,
  pub gloss_en: Option<String>,
  /// Extra SLP1 keys (the headword key is added automatically if Devanagari)
  pub keys: Vec<(String, KeyKind)>,
  /// Skip automatic headword key (English headwords)
  pub no_headword_key: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Kind {
  Lexicon,
  Thesaurus,
  DhatuWork,
  SutraCommentary,
  Concordance,
}

impl Kind {
  pub fn as_str(self) -> &'static str {
    match self {
      Kind::Lexicon => "lexicon",
      Kind::Thesaurus => "thesaurus",
      Kind::DhatuWork => "dhatu_work",
      Kind::SutraCommentary => "sutra_commentary",
      Kind::Concordance => "concordance",
    }
  }
}

pub struct Source {
  pub slug: &'static str,
  pub name: &'static str,
  pub short_name: &'static str,
  pub kind: Kind,
  pub src_lang: &'static str,
  pub tgt_lang: &'static str,
  pub description: &'static str,
  pub attribution: &'static str,
  pub sort_order: i32,
  /// Files (relative to the data dir) whose bytes make up the source hash
  pub files: &'static [&'static str],
  pub parse: fn(&Path) -> Result<Vec<Rec>>,
}

const SC: &str = "StarDict edition by the sanskrit-coders stardict-sanskrit project, distributed by Arsha Drishti.";
const CDSL: &str = "Digitization: Cologne Digital Sanskrit Dictionaries (CC BY-SA). StarDict edition by the sanskrit-coders stardict-sanskrit project.";

pub fn all() -> Vec<Source> {
  vec![
    Source {
      slug: "mw",
      name: "Monier-Williams Sanskrit-English Dictionary",
      short_name: "MW",
      kind: Kind::Lexicon,
      src_lang: "sa",
      tgt_lang: "en",
      description: "Sir Monier Monier-Williams, A Sanskrit-English Dictionary (1899). One entry per headword; senses in book order.",
      attribution: CDSL,
      sort_order: 10,
      files: &["MW/mw-sa/mw-sa.tsv"],
      parse: |d| {
        lexicon_tsv(
          d,
          "MW/mw-sa/mw-sa.tsv",
          Opts {
            mw_as: true,
            ..Default::default()
          },
          true,
        )
      },
    },
    Source {
      slug: "apte",
      name: "Apte Practical Sanskrit-English Dictionary",
      short_name: "Apte",
      kind: Kind::Lexicon,
      src_lang: "sa",
      tgt_lang: "en",
      description: "V. S. Apte, The Practical Sanskrit-English Dictionary (1890). Nominal headwords are often given declined (ज्ञानम्); stem forms are indexed too.",
      attribution: CDSL,
      sort_order: 20,
      files: &[
        "Apte/apte-sa/apte-sa.tsv",
        "Apte/apte-prAtipadikam-sa/apte-prAtipadikam-sa.tsv",
      ],
      parse: parse_apte,
    },
    Source {
      slug: "shabda-sagara",
      name: "Shabda-Sagara Sanskrit-English Dictionary",
      short_name: "ŚS",
      kind: Kind::Lexicon,
      src_lang: "sa",
      tgt_lang: "en",
      description: "Wilson / Yates, Śabda-Sāgara (1900).",
      attribution: CDSL,
      sort_order: 30,
      files: &["Other/shabda-sAgara/shabda-sAgara.tsv"],
      parse: |d| {
        lexicon_tsv(
          d,
          "Other/shabda-sAgara/shabda-sAgara.tsv",
          Opts {
            cologne: true,
            ..Default::default()
          },
          true,
        )
      },
    },
    Source {
      slug: "bhsd",
      name: "Buddhist Hybrid Sanskrit Dictionary (Edgerton)",
      short_name: "BHSD",
      kind: Kind::Lexicon,
      src_lang: "sa",
      tgt_lang: "en",
      description: "Franklin Edgerton, Buddhist Hybrid Sanskrit Grammar and Dictionary, Vol. II (1953). Headwords in IAST.",
      attribution: "Cologne 2013 edition (CC BY-NC-SA 3.0); TEI and StarDict by Simon Wiles.",
      sort_order: 40,
      files: &[
        "Other/stardict-bhsd/bhsd.idx",
        "Other/stardict-bhsd/bhsd.dict.dz",
      ],
      parse: parse_bhsd,
    },
    Source {
      slug: "vacaspatyam",
      name: "Vācaspatyam",
      short_name: "वाचस्पत्यम्",
      kind: Kind::Lexicon,
      src_lang: "sa",
      tgt_lang: "sa",
      description: "Tārānātha Tarkavācaspati, Vācaspatyam — Sanskrit-Sanskrit encyclopaedic dictionary.",
      attribution: CDSL,
      sort_order: 50,
      files: &["Sanskrit-Sanskrit/vAchAspatyam-sa/vachaspatyam-sa.tsv"],
      parse: |d| {
        lexicon_tsv(
          d,
          "Sanskrit-Sanskrit/vAchAspatyam-sa/vachaspatyam-sa.tsv",
          Opts {
            cologne: true,
            ..Default::default()
          },
          false,
        )
      },
    },
    Source {
      slug: "kalpadruma",
      name: "Śabdakalpadruma",
      short_name: "कल्पद्रुमः",
      kind: Kind::Lexicon,
      src_lang: "sa",
      tgt_lang: "sa",
      description: "Rādhākāntadeva, Śabdakalpadruma — Sanskrit-Sanskrit encyclopaedic dictionary.",
      attribution: CDSL,
      sort_order: 60,
      files: &["Sanskrit-Sanskrit/kalpadruma-sa/kalpadruma-sa.tsv"],
      parse: |d| {
        lexicon_tsv(
          d,
          "Sanskrit-Sanskrit/kalpadruma-sa/kalpadruma-sa.tsv",
          Opts {
            cologne: true,
            ..Default::default()
          },
          false,
        )
      },
    },
    Source {
      slug: "amarakosha",
      name: "Amarakośa (ontology)",
      short_name: "अमरकोशः",
      kind: Kind::Thesaurus,
      src_lang: "sa",
      tgt_lang: "sa",
      description: "Amarakośa with synonym sets, verse references and semantic relations.",
      attribution: "Data from Smt. Amba Kulkarni / University of Hyderabad (SCL); StarDict by sanskrit-coders.",
      sort_order: 70,
      files: &["Sanskrit-Sanskrit/amara-onto/amara-onto.tsv"],
      parse: parse_amara,
    },
    Source {
      slug: "akhyatachandrika",
      name: "Ākhyātacandrikā",
      short_name: "आख्यातचन्द्रिका",
      kind: Kind::Thesaurus,
      src_lang: "sa",
      tgt_lang: "sa",
      description: "Synonymous verbs grouped by meaning, with chapter references.",
      attribution: SC,
      sort_order: 75,
      files: &["Sanskrit-Sanskrit/AkhyAtachandrikA/AkhyAtachandrikA_dict.tsv"],
      parse: parse_akhyata,
    },
    Source {
      slug: "ranade-vedic-rituals",
      name: "Vedic Ritual Terms (Rānaḍe, Sanskrit-Hindi)",
      short_name: "रानडे",
      kind: Kind::Lexicon,
      src_lang: "sa",
      tgt_lang: "hi",
      description: "Glossary of Śrauta / Vedic ritual terminology with Hindi explanations and references.",
      attribution: SC,
      sort_order: 80,
      files: &["rAnADe-vedic-rituals__2024-01-01_14-24-34Z__0MB.tar/rAnADe-vedic-rituals.idx"],
      parse: parse_ranade,
    },
    Source {
      slug: "computer-shrikanta",
      name: "English-Sanskrit Computer Terms (Śrīkānta)",
      short_name: "Computer",
      kind: Kind::Lexicon,
      src_lang: "en",
      tgt_lang: "sa",
      description: "Āṅglasaṃskṛta-saṅgaṇaka-śabdakośa: English computing terms with Sanskrit equivalents. Search in English mode.",
      attribution: SC,
      sort_order: 90,
      files: &["Other/computer-shrIkAnta/computer-shrIkAnta.tsv"],
      parse: parse_computer,
    },
    Source {
      slug: "dhatupatha",
      name: "Pāṇinīya Dhātupāṭha",
      short_name: "धातुपाठः",
      kind: Kind::DhatuWork,
      src_lang: "sa",
      tgt_lang: "sa",
      description: "Roots with gaṇa, it-status, pada, meaning and Dhātupāṭha number.",
      attribution: SC,
      sort_order: 100,
      files: &["Panini/dhAtupATha-sa/dhatupatha-sa.tsv"],
      parse: parse_dhatupatha_entries,
    },
    Source {
      slug: "madhaviya-dhatuvrtti",
      name: "Mādhavīyā Dhātuvṛttiḥ",
      short_name: "माधवीया",
      kind: Kind::DhatuWork,
      src_lang: "sa",
      tgt_lang: "sa",
      description: "Sāyaṇa-Mādhava's commentary on the Dhātupāṭha.",
      attribution: SC,
      sort_order: 110,
      files: &["Sanskrit-Sanskrit/mAdhavIya-dhAtu-vRtti/mAdhavIya-dhAtu-vRtti.tsv"],
      parse: |d| {
        dhatu_work_tsv(
          d,
          "Sanskrit-Sanskrit/mAdhavIya-dhAtu-vRtti/mAdhavIya-dhAtu-vRtti.tsv",
        )
      },
    },
    Source {
      slug: "kshiratarangini",
      name: "Kṣīrataraṅgiṇī",
      short_name: "क्षीरतरङ्गिणी",
      kind: Kind::DhatuWork,
      src_lang: "sa",
      tgt_lang: "sa",
      description: "Kṣīrasvāmin's commentary on the Dhātupāṭha.",
      attribution: SC,
      sort_order: 120,
      files: &["Other/kShIratarangiNI/kShIratarangiNI.tsv"],
      parse: |d| dhatu_work_tsv(d, "Other/kShIratarangiNI/kShIratarangiNI.tsv"),
    },
    Source {
      slug: "dhatupradipa",
      name: "Dhātupradīpaḥ",
      short_name: "धातुप्रदीपः",
      kind: Kind::DhatuWork,
      src_lang: "sa",
      tgt_lang: "sa",
      description: "Maitreya-rakṣita's Dhātupradīpa.",
      attribution: SC,
      sort_order: 130,
      files: &["Grammar/dhAtupradIpa/dhAtupradIpa.tsv"],
      parse: |d| dhatu_work_tsv(d, "Grammar/dhAtupradIpa/dhAtupradIpa.tsv"),
    },
    Source {
      slug: "krdanta-rupa-mala",
      name: "Kṛdantarūpamālā",
      short_name: "कृदन्तरूपमाला",
      kind: Kind::DhatuWork,
      src_lang: "sa",
      tgt_lang: "sa",
      description: "S. Ramasubba Sastri, Kṛdantarūpamālā: kṛt derivatives of each root.",
      attribution: "Cologne (Jim Funderburk) digitization; StarDict by sanskrit-coders.",
      sort_order: 140,
      files: &["Other/kRdanta-rUpa-mAlA/kRdanta-rUpa-mAlA.tsv"],
      parse: |d| dhatu_work_tsv(d, "Other/kRdanta-rUpa-mAlA/kRdanta-rUpa-mAlA.tsv"),
    },
    Source {
      slug: "kashika",
      name: "Kāśikāvṛttiḥ",
      short_name: "काशिका",
      kind: Kind::SutraCommentary,
      src_lang: "sa",
      tgt_lang: "sa",
      description: "Vāmana & Jayāditya's Kāśikā on the Aṣṭādhyāyī.",
      attribution: SC,
      sort_order: 200,
      files: &[
        "Grammar/kashika/kashika.idx",
        "Grammar/kashika/kashika.dict",
      ],
      parse: |d| sutra_stardict(d, "Grammar/kashika/kashika"),
    },
    Source {
      slug: "nyasa",
      name: "Nyāsaḥ (Kāśikāvivaraṇapañjikā)",
      short_name: "न्यासः",
      kind: Kind::SutraCommentary,
      src_lang: "sa",
      tgt_lang: "sa",
      description: "Jinendrabuddhi's Nyāsa on the Kāśikā. Some glyphs were lost in the source digitization (shown as ◌).",
      attribution: SC,
      sort_order: 210,
      files: &["Panini/nyasa/nyasa.tsv"],
      parse: |d| sutra_tsv(d, "Panini/nyasa/nyasa.tsv"),
    },
    Source {
      slug: "laghu-kaumudi",
      name: "Laghusiddhāntakaumudī",
      short_name: "लघुकौमुदी",
      kind: Kind::SutraCommentary,
      src_lang: "sa",
      tgt_lang: "sa",
      description: "Varadarāja's Laghusiddhāntakaumudī.",
      attribution: SC,
      sort_order: 220,
      files: &[
        "Grammar/laghu-kaumudi/laghu-kaumudi.idx",
        "Grammar/laghu-kaumudi/laghu-kaumudi.dict",
      ],
      parse: |d| sutra_stardict(d, "Grammar/laghu-kaumudi/laghu-kaumudi"),
    },
    Source {
      slug: "balamanorama",
      name: "Bālamanoramā",
      short_name: "बालमनोरमा",
      kind: Kind::SutraCommentary,
      src_lang: "sa",
      tgt_lang: "sa",
      description: "Vāsudeva Dīkṣita's Bālamanoramā on the Siddhāntakaumudī. Some glyphs were lost in the source digitization (shown as ◌).",
      attribution: SC,
      sort_order: 230,
      files: &[
        "Grammar/balamanorama/balamanorama.idx",
        "Grammar/balamanorama/balamanorama.dict",
      ],
      parse: |d| sutra_stardict(d, "Grammar/balamanorama/balamanorama"),
    },
    Source {
      slug: "tattvabodhini",
      name: "Tattvabodhinī",
      short_name: "तत्त्वबोधिनी",
      kind: Kind::SutraCommentary,
      src_lang: "sa",
      tgt_lang: "sa",
      description: "Jñānendra Sarasvatī's Tattvabodhinī on the Siddhāntakaumudī. Some glyphs were lost in the source digitization (shown as ◌).",
      attribution: SC,
      sort_order: 240,
      files: &["Panini/tattvabodhini/tattvabodhini.tsv"],
      parse: |d| sutra_tsv(d, "Panini/tattvabodhini/tattvabodhini.tsv"),
    },
    Source {
      slug: "rv-padapatha",
      name: "Ṛgveda Saṃhitā & Padapāṭha (by pāda)",
      short_name: "ऋग्वेदः",
      kind: Kind::Concordance,
      src_lang: "sa",
      tgt_lang: "sa",
      description: "Every Ṛgveda pāda with location, accented saṃhitā text and padapāṭha.",
      attribution: "TITUS Ṛgveda text; StarDict by sanskrit-coders.",
      sort_order: 300,
      files: &["Other/rv-padapATha-dev/rv-padapATha-dev.tsv"],
      parse: parse_rv_padapatha,
    },
    Source {
      slug: "rv-padasvara",
      name: "Ṛgveda accented word index",
      short_name: "ऋक्पदस्वरः",
      kind: Kind::Concordance,
      src_lang: "sa",
      tgt_lang: "sa",
      description: "Accented Ṛgveda words with all their locations (maṇḍala sūkta_(global) ṛk pāda).",
      attribution: "TITUS Ṛgveda text; StarDict by sanskrit-coders.",
      sort_order: 310,
      files: &["Other/rv-padasvara-dev/rv-padasvara-dev.tsv"],
      parse: |d| {
        plain_tsv(
          d,
          "Other/rv-padasvara-dev/rv-padasvara-dev.tsv",
          Opts::default(),
        )
      },
    },
    Source {
      slug: "bloomfield",
      name: "Bloomfield's Vedic Concordance",
      short_name: "Bloomfield",
      kind: Kind::Concordance,
      src_lang: "sa",
      tgt_lang: "en",
      description: "Maurice Bloomfield, A Vedic Concordance (1906): every mantra pāda with its occurrences.",
      attribution: SC,
      sort_order: 320,
      files: &["Other/bloomfield-vedic-sa/bloomfield-vedic-sa.tsv"],
      parse: |d| {
        plain_tsv(
          d,
          "Other/bloomfield-vedic-sa/bloomfield-vedic-sa.tsv",
          Opts::default(),
        )
      },
    },
    Source {
      slug: "purana-index",
      name: "Purāṇa Index",
      short_name: "पुराणानुक्रमणी",
      kind: Kind::Concordance,
      src_lang: "sa",
      tgt_lang: "en",
      description: "V. R. Ramachandra Dikshitar, The Purāṇa Index: proper names with Purāṇa references.",
      attribution: CDSL,
      sort_order: 330,
      files: &["Other/purANa-index-dev/purANa-index-dev.tsv"],
      parse: parse_purana,
    },
  ]
}

// ---------------------------------------------------------------- helpers

pub fn read_tsv(dir: &Path, rel: &str) -> Result<Vec<(String, String)>> {
  let p: PathBuf = dir.join(rel);
  let text = std::fs::read_to_string(&p).with_context(|| format!("reading {p:?}"))?;
  Ok(
    text
      .split('\n')
      .filter(|l| !l.trim().is_empty())
      .filter_map(|l| {
        let (h, b) = l.split_once('\t')?;
        let h = h.trim_start_matches('\u{feff}').trim();
        if h.is_empty() {
          return None;
        }
        Some((crate::translit::nfc(h), crate::translit::nfc(b)))
      })
      .collect(),
  )
}

/// Contiguous runs of the same headword → one record each.
fn group<T>(rows: Vec<(String, T)>) -> Vec<(String, Vec<T>)> {
  let mut out: Vec<(String, Vec<T>)> = Vec::new();
  for (h, b) in rows {
    match out.last_mut() {
      Some((lh, v)) if *lh == h => v.push(b),
      _ => out.push((h, vec![b])),
    }
  }
  out
}

fn lexicon_tsv(dir: &Path, rel: &str, o: Opts, english: bool) -> Result<Vec<Rec>> {
  let rows = read_tsv(dir, rel)?;
  Ok(
    group(rows)
      .into_iter()
      .map(|(h, bodies)| {
        let senses: Vec<String> = bodies
          .iter()
          .map(|b| text_to_html(&strip_lead_headword(&markup::unescape_tsv(b)), o))
          .filter(|s| !s.is_empty())
          .collect();
        let gloss_en = english.then(|| markup::english_gloss(&senses));
        Rec {
          headword: h,
          senses,
          gloss_en,
          ..Default::default()
        }
      })
      .collect(),
  )
}

/// Śabdasāgara / Vācaspatyam bodies start with `hw¦` or `{@hw@}¦`; the
/// headword is shown separately, so drop that prefix.
fn strip_lead_headword(b: &str) -> String {
  let t = b.trim_start();
  if let Some((pre, rest)) = t.split_once('¦')
    && pre.chars().count() <= 60
    && !pre.contains('\n')
  {
    return rest.trim_start().to_string();
  }
  t.to_string()
}

fn plain_tsv(dir: &Path, rel: &str, o: Opts) -> Result<Vec<Rec>> {
  let rows = read_tsv(dir, rel)?;
  Ok(
    group(rows)
      .into_iter()
      .map(|(h, bodies)| Rec {
        headword: h,
        senses: bodies
          .iter()
          .map(|b| text_to_html(&markup::unescape_tsv(b), o))
          .collect(),
        ..Default::default()
      })
      .collect(),
  )
}

// ---------------------------------------------------------------- lexicons

fn parse_apte(dir: &Path) -> Result<Vec<Rec>> {
  let sa = read_tsv(dir, "Apte/apte-sa/apte-sa.tsv")?;
  let prati = read_tsv(dir, "Apte/apte-prAtipadikam-sa/apte-prAtipadikam-sa.tsv")?;
  // The prātipadika file is line-aligned with apte-sa: same body, prefixed by
  // the original headword; its key is the stem. Verify alignment per line.
  let aligned = sa.len() == prati.len();
  let mut rows: Vec<(String, (String, Option<String>))> = Vec::with_capacity(sa.len());
  for (i, (h, b)) in sa.into_iter().enumerate() {
    let stem = if aligned {
      let (ph, pb) = &prati[i];
      (pb.trim_end().ends_with(b.trim_end()) && ph != &h).then(|| ph.clone())
    } else {
      None
    };
    rows.push((h, (b, stem)));
  }
  Ok(
    group(rows)
      .into_iter()
      .map(|(h, items)| {
        let mut keys = Vec::new();
        for (_, stem) in &items {
          if let Some(s) = stem {
            keys.push((deva_key(s), KeyKind::Pratipadika));
          }
        }
        let senses: Vec<String> = items
          .iter()
          .map(|(b, _)| text_to_html(&markup::unescape_tsv(b), Opts::default()))
          .filter(|s| !s.is_empty())
          .collect();
        let gloss_en = Some(markup::english_gloss(&senses));
        Rec {
          headword: h,
          senses,
          gloss_en,
          keys,
          ..Default::default()
        }
      })
      .collect(),
  )
}

fn parse_bhsd(dir: &Path) -> Result<Vec<Rec>> {
  let sd = stardict::load(&dir.join("Other/stardict-bhsd/bhsd"))?;
  // Book order: BHSD payloads carry the Cologne record number `[N]`.
  static NUM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<small>\[(\d+)\]").unwrap());
  let mut rows: Vec<(u32, String, String)> = sd
    .entries
    .into_iter()
    .map(|(w, p)| {
      let n = NUM
        .captures(&p)
        .and_then(|c| c[1].parse().ok())
        .unwrap_or(u32::MAX);
      (n, crate::translit::nfc(&w), p)
    })
    .collect();
  rows.sort_by_key(|r| r.0);
  Ok(
    rows
      .into_iter()
      .map(|(_, w, p)| {
        let senses = vec![markup::sanitize_html(&p)];
        let gloss_en = Some(markup::english_gloss(&senses));
        let key = iast_key(&w);
        Rec {
          headword: w,
          senses,
          gloss_en,
          keys: vec![(key, KeyKind::Headword)],
          no_headword_key: true,
          ..Default::default()
        }
      })
      .collect(),
  )
}

fn parse_ranade(dir: &Path) -> Result<Vec<Rec>> {
  let sd = stardict::load(
    &dir.join("rAnADe-vedic-rituals__2024-01-01_14-24-34Z__0MB.tar/rAnADe-vedic-rituals"),
  )?;
  let rows: Vec<(String, String)> = sd
    .entries
    .into_iter()
    .map(|(w, p)| {
      let w = crate::translit::nfc(&w);
      // Payload repeats the headword before the first <br>.
      let body = match p.split_once("<br>") {
        Some((first, rest)) if first.trim() == w => rest.to_string(),
        _ => p,
      };
      let body = markup::sanitize_html(&body);
      static MD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*\*(.+?)\*\*").unwrap());
      (w, MD.replace_all(&body, "<b>$1</b>").into_owned())
    })
    .collect();
  Ok(
    group(rows)
      .into_iter()
      .map(|(h, senses)| Rec {
        headword: h,
        senses,
        ..Default::default()
      })
      .collect(),
  )
}

fn parse_computer(dir: &Path) -> Result<Vec<Rec>> {
  let rows = read_tsv(dir, "Other/computer-shrIkAnta/computer-shrIkAnta.tsv")?;
  Ok(
    group(rows)
      .into_iter()
      .map(|(h, bodies)| {
        let senses: Vec<String> = bodies
          .iter()
          .map(|b| text_to_html(&markup::unescape_tsv(b), Opts::default()))
          .collect();
        let gloss_en = Some(format!("{h}\n{}", markup::english_gloss(&senses)));
        Rec {
          headword: h,
          senses,
          gloss_en,
          no_headword_key: true,
          ..Default::default()
        }
      })
      .collect(),
  )
}

// ---------------------------------------------------------------- thesauri

fn links_list(list: &str, sep: char) -> String {
  list
    .split(sep)
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .map(|w| {
      let label = w.replace('_', " ");
      xref(&label, &deva_key(&label))
    })
    .collect::<Vec<_>>()
    .join(", ")
}

fn parse_amara(dir: &Path) -> Result<Vec<Rec>> {
  let rows = read_tsv(dir, "Sanskrit-Sanskrit/amara-onto/amara-onto.tsv")?;
  static REF: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+।\d+।\d+।\d+।\d+$").unwrap());
  Ok(
    group(rows)
      .into_iter()
      .map(|(h, bodies)| {
        let mut senses = Vec::new();
        let mut synsets = Vec::new();
        let mut refs = Vec::new();
        for b in bodies {
          let body = markup::unescape_tsv(&b);
          let mut parts = Vec::new();
          for line in body.split('\n').map(str::trim).filter(|l| !l.is_empty()) {
            if let Some(list) = line.strip_prefix("समानार्थक:") {
              synsets.push(
                list
                  .split(',')
                  .map(|s| s.trim().to_string())
                  .collect::<Vec<_>>(),
              );
              parts.push(format!("<b>समानार्थकाः</b> {}", links_list(list, ',')));
            } else if REF.is_match(line) {
              refs.push(line.to_string());
              parts.push(format!(r#"<span class="ref">{}</span>"#, escape(line)));
            } else if let Some((label, list)) =
              line.split_once(" : ").or_else(|| line.split_once(": "))
            {
              let label = label.trim();
              if label.chars().count() <= 24
                && !list.trim().is_empty()
                && list.contains(|c: char| has_devanagari(&c.to_string()))
              {
                parts.push(format!(
                  "<b>{}</b> {}",
                  escape(label),
                  links_list(list, ',')
                ));
              } else {
                parts.push(escape(line));
              }
            } else {
              parts.push(escape(line));
            }
          }
          senses.push(parts.join("<br>"));
        }
        let data = json!({ "synonyms": synsets, "refs": refs });
        Rec {
          headword: h,
          senses,
          data: Some(data),
          ..Default::default()
        }
      })
      .collect(),
  )
}

fn parse_akhyata(dir: &Path) -> Result<Vec<Rec>> {
  let rows = read_tsv(
    dir,
    "Sanskrit-Sanskrit/AkhyAtachandrikA/AkhyAtachandrikA_dict.tsv",
  )?;
  static RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?P<artha>.*?)\s+(?P<ref>\d+\.\d+\.\d+)\s+(?P<syn>.*)$").unwrap()
  });
  Ok(
    group(rows)
      .into_iter()
      .map(|(h, bodies)| {
        let senses = bodies
          .iter()
          .map(|b| match RE.captures(b.trim()) {
            Some(c) => format!(
              r#"<b>{}</b> <span class="ref">{}</span><br>{}"#,
              escape(&c["artha"]),
              escape(&c["ref"]),
              escape(c["syn"].trim())
            ),
            None => text_to_html(b, Opts::default()),
          })
          .collect();
        Rec {
          headword: h,
          senses,
          ..Default::default()
        }
      })
      .collect(),
  )
}

// ---------------------------------------------------------------- dhātu

#[derive(Debug, Clone)]
pub struct Dhatu {
  pub number: String,
  pub upadesha: String,
  pub root: String,
  pub root_slp1: String,
  pub artha: String,
  pub gana: i16,
  pub pada: Option<String>,
  pub it: Option<String>,
  pub extra: Option<String>,
  pub raw: String,
}

fn deva_digits(s: &str) -> String {
  s.chars()
    .map(|c| match c {
      '०'..='९' => char::from_digit(c as u32 - '०' as u32, 10).unwrap(),
      c => c,
    })
    .collect()
}

/// Parse `dhatupatha-sa.tsv` lines, e.g.
/// `आप् \t चु० सेट् उ० (१.३.७४); आपॢ (आपॢ)लम्भने १०.३७६ ;;`
pub fn parse_dhatus(dir: &Path) -> Result<Vec<Dhatu>> {
  let rows = read_tsv(dir, "Panini/dhAtupATha-sa/dhatupatha-sa.tsv")?;
  static RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
      r"^(?:(?P<pre>[^;]*?);\s*)?(?P<g>\S+०)\s+(?P<it>सेट्|अनिट्|वेट्)\s+(?P<pada>[पआउ])०\s*(?:\((?P<sref>[^)]*)\))?\s*;\s*(?P<rest>.*?)\s*(?P<num>[०-९]+\.[०-९]+)\s*;;\s*$",
    )
    .unwrap()
  });
  static NUM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([०-९]+)\.([०-९]+)\s*;;\s*$").unwrap());
  let mut out = Vec::new();
  let mut unparsed = 0;
  for (root, body) in rows {
    let body = body.trim().to_string();
    let root = root.trim().to_string();
    let root_slp1 = deva_key(&root);
    if let Some(c) = RE.captures(&body) {
      let num = deva_digits(&c["num"]);
      let gana = num
        .split('.')
        .next()
        .and_then(|g| g.parse().ok())
        .unwrap_or(0);
      let rest = c["rest"].trim().to_string();
      let (upadesha, artha) = split_upadesha(&rest);
      let mut extra = Vec::new();
      if let Some(p) = c.name("pre") {
        extra.push(p.as_str().trim().to_string());
      }
      if let Some(s) = c.name("sref") {
        extra.push(format!("({})", s.as_str()));
      }
      out.push(Dhatu {
        number: num,
        upadesha,
        root,
        root_slp1,
        artha,
        gana,
        pada: Some(
          match &c["pada"] {
            "प" => "P",
            "आ" => "A",
            _ => "U",
          }
          .to_string(),
        ),
        it: Some(c["it"].to_string()),
        extra: (!extra.is_empty()).then(|| extra.join(" ")),
        raw: body,
      });
    } else {
      unparsed += 1;
      let num = NUM
        .captures(&body)
        .map(|c| format!("{}.{}", deva_digits(&c[1]), deva_digits(&c[2])));
      let gana = num
        .as_deref()
        .and_then(|n| n.split('.').next()?.parse().ok())
        .unwrap_or(0);
      out.push(Dhatu {
        number: num.unwrap_or_default(),
        upadesha: root.clone(),
        root,
        root_slp1,
        artha: String::new(),
        gana,
        pada: None,
        it: None,
        extra: Some(body.clone()),
        raw: body,
      });
    }
  }
  if unparsed > 0 {
    eprintln!("  dhatupatha: {unparsed} lines kept unparsed (raw text in `extra`)");
  }
  Ok(out)
}

/// `आपॢ (आपॢ)लम्भने` → (`आपॢ`, `लम्भने`); `आचि आयामे` → (`आचि`, `आयामे`).
/// A bracketed group right after the upadeśa is a variant reading, unless
/// nothing follows it — then it is the meaning (`लगि- [गत्यर्थः]`).
fn split_upadesha(rest: &str) -> (String, String) {
  static RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([^\s(\[]+)(?:\s*\(([^)]*)\)|\s*\[([^\]]*)\])?\s*(.*)$").unwrap()
  });
  match RE.captures(rest) {
    Some(c) => {
      let up = c[1].trim_end_matches('-').to_string();
      let after = c[4].trim();
      let artha = if after.is_empty() {
        c.get(2)
          .or(c.get(3))
          .map(|m| m.as_str().trim_matches(['[', ']']).trim())
          .unwrap_or("")
      } else {
        after
      };
      (up, artha.to_string())
    }
    None => (rest.to_string(), String::new()),
  }
}

fn parse_dhatupatha_entries(dir: &Path) -> Result<Vec<Rec>> {
  let dhatus = parse_dhatus(dir)?;
  Ok(
    dhatus
      .iter()
      .enumerate()
      .map(|(i, d)| {
        let mut parts = vec![format!(
          "<b>{}</b> {}",
          escape(&d.upadesha),
          escape(&d.artha)
        )];
        let pada = match d.pada.as_deref() {
          Some("P") => "परस्मैपदी",
          Some("A") => "आत्मनेपदी",
          Some("U") => "उभयपदी",
          _ => "",
        };
        parts.push(format!(
          "गणः {} · {} · {} · <span class=\"ref\">{}</span>",
          d.gana,
          escape(d.it.as_deref().unwrap_or("")),
          pada,
          escape(&d.number)
        ));
        if let Some(e) = &d.extra {
          parts.push(format!("<small>{}</small>", escape(e)));
        }
        let mut keys = Vec::new();
        let up = deva_key(&d.upadesha);
        if !up.is_empty() && up != d.root_slp1 {
          keys.push((up, KeyKind::Variant));
        }
        Rec {
          headword: d.root.clone(),
          senses: vec![parts.join("<br>")],
          dhatu_index: Some(i),
          data: Some(json!({ "number": d.number, "gana": d.gana })),
          keys,
          ..Default::default()
        }
      })
      .collect(),
  )
}

fn dhatu_work_tsv(dir: &Path, rel: &str) -> Result<Vec<Rec>> {
  let rows = read_tsv(dir, rel)?;
  Ok(
    group(rows)
      .into_iter()
      .map(|(h, bodies)| Rec {
        dhatu_root_key: Some(deva_key(&h)),
        senses: bodies
          .iter()
          .map(|b| {
            text_to_html(
              &markup::unescape_tsv(b),
              Opts {
                cologne: true,
                ..Default::default()
              },
            )
          })
          .collect(),
        headword: h,
        ..Default::default()
      })
      .collect(),
  )
}

// ---------------------------------------------------------------- sūtras

pub static SUTRA_KEY: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(\d)\.(\d)\.(\d{3})\s*(.*)$").unwrap());

fn sutra_rec(key: &str, body: &str, from_tsv: bool) -> Option<Rec> {
  let c = SUTRA_KEY.captures(key.trim())?;
  let id = format!("{}.{}.{}", &c[1], &c[2], &c[3]);
  let text = c[4].trim().to_string();
  let body = if from_tsv {
    markup::unescape_tsv(body)
  } else {
    body.to_string()
  };
  // Bodies repeat the sūtra as their first paragraph; keep it (it is the
  // commentary's own reading) but render it bold.
  let (first, rest) = body.trim().split_once('\n').unwrap_or((body.trim(), ""));
  let o = Opts {
    accent_quotes: true,
    ..Default::default()
  };
  let mut html = format!("<b>{}</b>", text_to_html(first, o));
  let rest_html = text_to_html(rest, o);
  if !rest_html.is_empty() {
    html.push_str("<br><br>");
    html.push_str(&rest_html);
  }
  Some(Rec {
    headword: if text.is_empty() { id.clone() } else { text },
    senses: vec![html],
    sutra_id: Some(id),
    no_headword_key: false,
    ..Default::default()
  })
}

fn sutra_stardict(dir: &Path, base: &str) -> Result<Vec<Rec>> {
  let sd = stardict::load(&dir.join(base))?;
  // idx order is sorted by key; `a.p.nnn` keys sort into book order.
  let mut v: Vec<Rec> = sd
    .entries
    .iter()
    .filter_map(|(k, b)| sutra_rec(k, b, false))
    .collect();
  v.sort_by(|a, b| a.sutra_id.cmp(&b.sutra_id));
  Ok(v)
}

fn sutra_tsv(dir: &Path, rel: &str) -> Result<Vec<Rec>> {
  let rows = read_tsv(dir, rel)?;
  let mut v: Vec<Rec> = rows
    .iter()
    .filter_map(|(k, b)| sutra_rec(k, b, true))
    .collect();
  // Nyāsa splits long sūtra commentaries into several rows (stable sort keeps their order).
  v.sort_by(|a, b| a.sutra_id.cmp(&b.sutra_id));
  Ok(v)
}

/// Sūtra list (id, text) taken from the Kāśikā keys.
pub fn parse_sutras(dir: &Path) -> Result<Vec<(String, String)>> {
  let sd = stardict::load(&dir.join("Grammar/kashika/kashika"))?;
  let mut v: Vec<(String, String)> = sd
    .entries
    .iter()
    .filter_map(|(k, _)| {
      let c = SUTRA_KEY.captures(k.trim())?;
      Some((
        format!("{}.{}.{}", &c[1], &c[2], &c[3]),
        crate::translit::nfc(c[4].trim()),
      ))
    })
    .collect();
  v.sort();
  v.dedup_by(|a, b| a.0 == b.0);
  Ok(v)
}

#[cfg(test)]
mod tests {
  use super::split_upadesha;
  #[test]
  fn upadesha() {
    let s = |x: &str| split_upadesha(x);
    assert_eq!(s("भू सत्तायाम्"), ("भू".into(), "सत्तायाम्".into()));
    assert_eq!(s("आपॢ (आपॢ)लम्भने"), ("आपॢ".into(), "लम्भने".into()));
    assert_eq!(
      s("ओलडि [ओलडि]उत्क्षेपने ; उलडि इत्यन्ये"),
      ("ओलडि".into(), "उत्क्षेपने ; उलडि इत्यन्ये".into())
    );
    assert_eq!(s("लगि- [गत्यर्थः]"), ("लगि".into(), "गत्यर्थः".into()));
  }
}

// ---------------------------------------------------------------- concordances

fn parse_rv_padapatha(dir: &Path) -> Result<Vec<Rec>> {
  let rows = read_tsv(dir, "Other/rv-padapATha-dev/rv-padapATha-dev.tsv")?;
  static LOC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+)\s+(\d+)(?:_\((\d+)\))?\s+(\d+)\s+([a-z])$").unwrap());
  Ok(
    group(rows)
      .into_iter()
      .map(|(h, bodies)| {
        let mut locs = Vec::new();
        let senses = bodies
          .iter()
          .map(|b| {
            let body = markup::unescape_tsv(b);
            let mut lines = body.trim().splitn(2, '\n');
            let first = lines.next().unwrap_or("").trim();
            let rest = lines.next().unwrap_or("");
            match LOC.captures(first) {
              Some(c) => {
                locs.push(json!({ "mandala": c[1].parse::<u32>().ok(), "sukta": c[2].parse::<u32>().ok(), "rk": c[4].parse::<u32>().ok(), "pada": &c[5] }));
                format!(
                  r#"<span class="ref">ऋ॰ {}.{}.{}{}</span><br>{}"#,
                  &c[1], &c[2], &c[4], &c[5],
                  text_to_html(rest, Opts::default())
                )
              }
              None => text_to_html(&body, Opts::default()),
            }
          })
          .collect();
        Rec { headword: h, senses, data: Some(json!({ "locations": locs })), ..Default::default() }
      })
      .collect(),
  )
}

fn parse_purana(dir: &Path) -> Result<Vec<Rec>> {
  let rows = read_tsv(dir, "Other/purANa-index-dev/purANa-index-dev.tsv")?;
  Ok(
    group(rows)
      .into_iter()
      .map(|(h, bodies)| {
        let senses: Vec<String> = bodies
          .iter()
          .map(|b| {
            text_to_html(
              &markup::unescape_tsv(b),
              Opts {
                cologne: true,
                ..Default::default()
              },
            )
            .replace("&lt;F&gt;", r#"<small>"#)
            .replace("&lt;/F&gt;", "</small>")
          })
          .collect();
        let gloss_en = Some(markup::english_gloss(&senses));
        Rec {
          headword: h,
          senses,
          gloss_en,
          ..Default::default()
        }
      })
      .collect(),
  )
}

// ---------------------------------------------------------------- word forms

pub struct Form {
  pub form: String,
  pub lemma: String,
  pub analysis: String,
  pub paradigm: Value,
}

/// JNU tiṅanta: key = conjugated form; body = `root (artha, gaṇa, pada, lakāra)\n 3×3 forms`.
pub fn parse_jnu_forms(dir: &Path) -> Result<Vec<Form>> {
  let rows = read_tsv(dir, "Grammar/jnu-tiNanta/jnu-tiNanta.tsv")?;
  static HEAD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*(\S+)\s*\(([^)]*)\)").unwrap());
  let mut out = Vec::new();
  for (form, body) in rows {
    let body = markup::unescape_tsv(&body);
    let mut lines = body.split('\n');
    let Some(c) = lines.next().and_then(|l| HEAD.captures(l)) else {
      continue;
    };
    let paradigm: Vec<Vec<String>> = lines
      .map(|l| l.split_whitespace().map(String::from).collect::<Vec<_>>())
      .filter(|r| !r.is_empty())
      .collect();
    out.push(Form {
      form,
      lemma: c[1].to_string(),
      analysis: c[2].trim().to_string(),
      paradigm: json!(paradigm),
    });
  }
  Ok(out)
}
