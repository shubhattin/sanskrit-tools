//! Meter identification (vidyut-chandas): syllable scansion, the best-matching meter, every
//! compatible vṛtta, and the meter catalogue.
//!
//! vidyut's catalogue (`data/chandas/meters.tsv`, from the Vṛttaratnākara) has the vṛttas and
//! seven jātis, but not anuṣṭubh. Since most verses are ślokas, anuṣṭubh is checked here.

use super::{Result, err, to_deva, to_slp1};
use serde::Serialize;
use std::sync::LazyLock;
use utoipa::ToSchema;
use vidyut_chandas::{Akshara, Chandas, MatchType, VrttaWeight, Weight};

static CHANDAS: LazyLock<Chandas> = LazyLock::new(|| {
  Chandas::from_text(include_str!("data/chandas/meters.tsv")).expect("embedded meters.tsv parses")
});

const ANUSHTUBH: &str = "अनुष्टुप्";

#[derive(Serialize, ToSchema, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MeterKind {
  /// Fixed syllable pattern.
  Vrtta,
  /// Counted by mātrās.
  Jati,
  /// The śloka: eight syllables a pāda, weights fixed only at the 5th–7th.
  Anushtubh,
}

#[derive(Serialize, ToSchema, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Fit {
  /// The whole text fits the meter.
  Full,
  /// Whole pādas fit.
  Pada,
  /// The text fits the start of the meter.
  Prefix,
}

#[derive(Serialize, ToSchema)]
pub struct MeterMatch {
  pub name: String,
  pub kind: MeterKind,
  pub fit: Fit,
  /// e.g. "vipulā" for a śloka whose odd pāda leaves the common (pathyā) pattern.
  pub note: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct Syllable {
  pub text: String,
  /// Guru (heavy, 2 mātrās); otherwise laghu.
  pub guru: bool,
}

#[derive(Serialize, ToSchema)]
pub struct ScannedLine {
  pub syllables: Vec<Syllable>,
  /// `L` / `G` per syllable.
  pub pattern: String,
  /// The pattern in gaṇas (य म त र ज भ न स, then ल / ग for the rest).
  pub ganas: Vec<String>,
  pub matras: i32,
}

#[derive(Serialize, ToSchema)]
pub struct Scansion {
  pub lines: Vec<ScannedLine>,
  pub best: Option<MeterMatch>,
  /// Other vṛttas the text is compatible with.
  pub others: Vec<MeterMatch>,
}

/// Verse text → SLP1 lines. Daṇḍas end lines; verse numbers and blank lines are dropped.
fn lines(text: &str) -> Result<Vec<String>> {
  let text: String = text
    .chars()
    .map(|c| match c {
      '।' | '॥' | '|' => '\n',
      '०'..='९' | '0'..='9' => ' ',
      c => c,
    })
    .collect();
  let mut out = Vec::new();
  for l in text.lines() {
    let s = to_slp1(l)?;
    if s.chars().any(|c| c.is_ascii_alphabetic()) {
      out.push(s);
    }
  }
  if out.is_empty() {
    return err("Enter a verse");
  }
  Ok(out)
}

fn weight_char(w: Weight) -> char {
  match w {
    Weight::G => 'G',
    Weight::L => 'L',
  }
}

/// `GGLGLLLGLLGLGG` → `त भ ज ज ग ग`.
pub fn ganas(pattern: &str) -> Vec<String> {
  let chars: Vec<char> = pattern.chars().collect();
  let mut out = Vec::new();
  let mut chunks = chars.chunks_exact(3);
  for c in chunks.by_ref() {
    let s: String = c.iter().collect();
    out.push(
      match s.as_str() {
        "LGG" => "य",
        "GGG" => "म",
        "GGL" => "त",
        "GLG" => "र",
        "LGL" => "ज",
        "GLL" => "भ",
        "LLL" => "न",
        "LLG" => "स",
        _ => "?",
      }
      .to_string(),
    );
  }
  for c in chunks.remainder() {
    out.push(if *c == 'G' { "ग" } else { "ल" }.to_string());
  }
  out
}

fn scanned(row: &[Akshara]) -> ScannedLine {
  let pattern: String = row.iter().map(|a| weight_char(a.weight())).collect();
  ScannedLine {
    syllables: row
      .iter()
      .map(|a| Syllable {
        text: to_deva(a.text()),
        guru: a.weight() == Weight::G,
      })
      .collect(),
    ganas: ganas(&pattern),
    matras: row.iter().map(|a| a.num_matras()).sum(),
    pattern,
  }
}

fn fit(m: MatchType) -> Option<Fit> {
  match m {
    MatchType::Full => Some(Fit::Full),
    MatchType::Pada => Some(Fit::Pada),
    MatchType::Prefix => Some(Fit::Prefix),
    MatchType::None => None,
  }
}

/// (vidyut's `Padya` type is not exported; its name tells jātis from vṛttas.)
fn padya_match(name: &str, m: MatchType) -> Option<MeterMatch> {
  let kind = if CHANDAS.jatis().iter().any(|j| j.name() == name) {
    MeterKind::Jati
  } else {
    MeterKind::Vrtta
  };
  Some(MeterMatch {
    name: to_deva(name),
    kind,
    fit: fit(m)?,
    note: None,
  })
}

/// Anuṣṭubh: whole half-verses of 16 syllables (or full pādas of 8). Even pādas must be pathyā
/// (5th laghu, 6th guru, 7th laghu); odd pādas are pathyā (5 L, 6 G, 7 G) or else a vipulā.
fn anushtubh(rows: &[Vec<Akshara>]) -> Option<MeterMatch> {
  let flat: Vec<Weight> = rows.iter().flatten().map(|a| a.weight()).collect();
  if flat.is_empty() || !flat.len().is_multiple_of(8) {
    return None;
  }
  // A lone pāda can be odd or even: only the shared 5th laghu / 6th guru is checked.
  if flat.len() == 8 {
    return (flat[4] == Weight::L && flat[5] == Weight::G).then(|| MeterMatch {
      name: ANUSHTUBH.into(),
      kind: MeterKind::Anushtubh,
      fit: Fit::Pada,
      note: None,
    });
  }
  if !flat.len().is_multiple_of(16) {
    return None;
  }
  let mut vipula = false;
  for (i, pada) in flat.chunks(8).enumerate() {
    let (l5, g6, g7) = (
      pada[4] == Weight::L,
      pada[5] == Weight::G,
      pada[6] == Weight::G,
    );
    if i % 2 == 1 {
      if !(l5 && g6 && !g7) {
        return None;
      }
    } else if !(l5 && g6 && g7) {
      vipula = true;
    }
  }
  Some(MeterMatch {
    name: ANUSHTUBH.into(),
    kind: MeterKind::Anushtubh,
    fit: if flat.len() >= 32 {
      Fit::Full
    } else {
      Fit::Pada
    },
    note: vipula.then(|| "vipulā".into()),
  })
}

/// Scans a verse (Devanagari; a line per pāda or half-verse, daṇḍas also end lines) and names
/// its meter.
pub fn identify(text: &str) -> Result<Scansion> {
  let c = &*CHANDAS;
  let lines = lines(text)?;
  let joined = lines.join("\n");
  let m = c.classify(&joined);
  let rows = m.aksharas();
  let mut best = m
    .padya()
    .as_ref()
    .and_then(|p| padya_match(p.name(), m.match_type()));
  let all = c.classify_all(&joined);
  let mut others = all
    .padyas()
    .iter()
    .zip(all.match_types())
    .filter_map(|(p, &t)| padya_match(p.name(), t))
    .filter(|o| best.as_ref().is_none_or(|b| b.name != o.name))
    .fold(Vec::<MeterMatch>::new(), |mut acc, o| {
      // vidyut's catalogue has a few names twice (variants); list each once.
      if !acc.iter().any(|a| a.name == o.name) {
        acc.push(o);
      }
      acc
    });
  // Anuṣṭubh is checked here: it beats a partial vṛtta match (short vṛttas match the start of
  // anything), and otherwise stays a candidate.
  if let Some(a) = anushtubh(rows) {
    if best.as_ref().is_none_or(|b| b.fit != Fit::Full) {
      if let Some(b) = best.take() {
        others.insert(0, b);
      }
      best = Some(a);
    } else {
      others.insert(0, a);
    }
  }
  Ok(Scansion {
    lines: rows.iter().map(|r| scanned(r)).collect(),
    best,
    others,
  })
}

#[derive(Serialize, ToSchema)]
pub struct MeterPada {
  /// `L` / `G`, `.` for "either" (pāda-final syllables).
  pub pattern: String,
  pub ganas: Vec<String>,
  /// Caesura (yati) after these syllable counts.
  pub yati: Vec<usize>,
}

#[derive(Serialize, ToSchema)]
pub struct MeterInfo {
  pub name: String,
  pub kind: MeterKind,
  /// Vṛtta: one entry per distinct pāda (samavṛttas have one).
  pub padas: Vec<MeterPada>,
  /// Jāti: mātrās per pāda.
  pub matras: Vec<i32>,
  /// Syllables per pāda (vṛtta, anuṣṭubh).
  pub syllables: Vec<usize>,
}

/// The meter catalogue: anuṣṭubh, the vṛttas (shortest first), then the jātis.
pub fn meters() -> Vec<MeterInfo> {
  let c = &*CHANDAS;
  let mut out = vec![MeterInfo {
    name: ANUSHTUBH.into(),
    kind: MeterKind::Anushtubh,
    padas: vec![
      MeterPada {
        pattern: "....LGG.".into(),
        ganas: vec![],
        yati: vec![],
      },
      MeterPada {
        pattern: "....LGL.".into(),
        ganas: vec![],
        yati: vec![],
      },
    ],
    matras: vec![],
    syllables: vec![8, 8, 8, 8],
  }];
  let mut vrttas: Vec<MeterInfo> = c
    .vrttas()
    .iter()
    .map(|v| {
      let mut padas: Vec<MeterPada> = Vec::new();
      for p in v.padas() {
        let pattern: String = p
          .weights()
          .iter()
          .map(|w| match w {
            VrttaWeight::G => 'G',
            VrttaWeight::L => 'L',
            VrttaWeight::Any => '.',
          })
          .collect();
        if padas.iter().any(|x| x.pattern == pattern) {
          continue;
        }
        padas.push(MeterPada {
          ganas: ganas(&pattern.replace('.', "G")),
          yati: p.yati().to_vec(),
          pattern,
        });
      }
      MeterInfo {
        name: to_deva(v.name()),
        kind: MeterKind::Vrtta,
        syllables: v.padas().iter().map(|p| p.weights().len()).collect(),
        padas,
        matras: vec![],
      }
    })
    .collect();
  vrttas.sort_by_key(|m| m.syllables.first().copied().unwrap_or(0));
  out.extend(vrttas);
  out.extend(c.jatis().iter().map(|j| MeterInfo {
    name: to_deva(j.name()),
    kind: MeterKind::Jati,
    padas: vec![],
    matras: j.matras().to_vec(),
    syllables: vec![],
  }));
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn identifies_meters() {
    let s = identify("मातः समस्तजगतां मधुकैटभारेः").unwrap();
    assert_eq!(s.lines[0].pattern, "GGLGLLLGLLGLGG");
    assert_eq!(s.lines[0].ganas.join(" "), "त भ ज ज ग ग");
    assert_eq!(s.best.unwrap().name, "वसन्ततिलका");
    let gita = "धर्मक्षेत्रे कुरुक्षेत्रे समवेता युयुत्सवः।\nमामकाः पाण्डवाश्चैव किमकुर्वत सञ्जय॥१॥";
    let s = identify(gita).unwrap();
    assert_eq!(s.best.unwrap().name, "अनुष्टुप्");
    assert!(identify("abc").is_err());
    let pada = identify("वागर्थाविव सम्पृक्तौ").unwrap();
    assert_eq!(pada.best.unwrap().name, "अनुष्टुप्");
    assert!(!pada.others.is_empty());
    assert!(meters().len() > 140);
  }
}
