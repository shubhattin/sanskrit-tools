//! Pāṇinian word generation (vidyut-prakriya): verb tables, declensions, kṛdantas, taddhitas,
//! feminine stems, compounds and sandhi between words, each with its step-by-step derivation.
//!
//! Requests name grammatical categories with the small snake_case ids below ([`LakaraId`] …);
//! kṛt / taddhita suffixes use vidyut's own SLP1 labels (`kta`, `Satf`, `aR`) because there are
//! hundreds of them. Words, stems and outputs are Devanagari.

use super::{Result, dedup, err, to_deva, word_slp1};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::LazyLock;
use utoipa::ToSchema;
use vidyut_prakriya::args::{
  BaseKrt, Dhatu, DhatuPada, Krdanta, Lakara, Linga, Pada, Pratipadika, Prayoga, Purusha, Samasa,
  SamasaType, Sanadi, Slp1String, Subanta, Taddhita, Taddhitanta, Tinanta, Vacana, Vibhakti,
};
use vidyut_prakriya::dhatupatha::Dhatupatha;
use vidyut_prakriya::{Decision, Prakriya, Rule, Vyakarana};

// ---------------------------------------------------------------------------------------------
// Data + engine
// ---------------------------------------------------------------------------------------------

const DHATUPATHA: &str = include_str!("data/prakriya/dhatupatha.tsv");

struct Root {
  code: String,
  gana: u8,
  number: u16,
  dhatu: Dhatu,
  aupadeshika: String,
  artha: String,
  /// Roots after it-lopa (SLP1), e.g. `eDa~\` → `eD`.
  clean: Vec<String>,
}

struct Engine {
  /// Logs every step (derivations).
  full: Vyakarana,
  /// No step log (tables: only the final words are needed).
  fast: Vyakarana,
  roots: Vec<Root>,
  by_code: HashMap<String, usize>,
  texts: RuleTexts,
}

static ENGINE: LazyLock<Engine> = LazyLock::new(|| {
  let fast = Vyakarana::builder().log_steps(false).build();
  let dp = Dhatupatha::from_text(DHATUPATHA).expect("embedded dhatupatha.tsv parses");
  let roots: Vec<Root> = dp
    .iter()
    .map(|e| {
      let clean = dedup(fast.derive_dhatus(e.dhatu()).iter().map(|p| p.text()));
      let gana = e.code().get(..2).and_then(|g| g.parse().ok()).unwrap_or(0);
      Root {
        code: e.code().to_string(),
        gana,
        number: e.number(),
        aupadeshika: e.dhatu().aupadeshika().unwrap_or_default().to_string(),
        dhatu: e.dhatu().clone(),
        artha: e.artha().to_string(),
        clean,
      }
    })
    .collect();
  let by_code = roots
    .iter()
    .enumerate()
    .map(|(i, r)| (r.code.clone(), i))
    .collect();
  Engine {
    full: Vyakarana::new(),
    fast,
    roots,
    by_code,
    texts: RuleTexts::load(),
  }
});

/// Text of the rules a derivation cites, from the shipped TSVs (`code<TAB>text`, SLP1).
struct RuleTexts {
  ashtadhyayi: HashMap<&'static str, &'static str>,
  varttika: HashMap<&'static str, &'static str>,
  unadi: HashMap<&'static str, &'static str>,
  dhatupatha: HashMap<&'static str, &'static str>,
  linganushasana: HashMap<&'static str, &'static str>,
  phit: HashMap<&'static str, &'static str>,
  kashika: HashMap<&'static str, &'static str>,
  kaumudi: HashMap<&'static str, &'static str>,
}

fn tsv(text: &'static str) -> HashMap<&'static str, &'static str> {
  text
    .lines()
    .skip(1)
    .filter_map(|l| l.split_once('\t'))
    .collect()
}

impl RuleTexts {
  fn load() -> Self {
    RuleTexts {
      ashtadhyayi: tsv(include_str!("data/prakriya/sutrapatha.tsv")),
      varttika: tsv(include_str!("data/prakriya/varttikas.tsv")),
      unadi: tsv(include_str!("data/prakriya/unadipatha.tsv")),
      dhatupatha: tsv(include_str!("data/prakriya/dhatupatha-ganasutras.tsv")),
      linganushasana: tsv(include_str!("data/prakriya/linganushasanam.tsv")),
      phit: tsv(include_str!("data/prakriya/phit-sutras.tsv")),
      kashika: tsv(include_str!("data/prakriya/kashika.tsv")),
      kaumudi: tsv(include_str!("data/prakriya/kaumudi.tsv")),
    }
  }
}

fn engine() -> &'static Engine {
  &ENGINE
}

/// A suffix's name as grammars write it: aupadeśika without the nasalization mark of its
/// it-vowel (`Satf~` → शतृ, `Ramu~l` → णमुल्).
fn suffix_label(slp1: &str) -> String {
  to_deva(&slp1.replace('~', ""))
}

/// Final texts of a set of prakriyās, in Devanagari, without duplicates. A pause form with a
/// voiced final stop is dropped when its voiceless twin is there (8.4.56 वाऽवसाने gives both
/// रामात् and रामाद्): tables show one, the derivation still has both.
fn texts(ps: &[Prakriya]) -> Vec<String> {
  let all = dedup(ps.iter().map(|p| p.text()));
  let twin = |s: &str| -> Option<String> {
    let (stem, last) = s.split_at(s.len().checked_sub(1)?);
    let voiceless = match last {
      "d" => "t",
      "g" => "k",
      "q" => "w",
      "b" => "p",
      _ => return None,
    };
    Some(format!("{stem}{voiceless}"))
  };
  all
    .iter()
    .filter(|s| twin(s).is_none_or(|t| !all.contains(&t)))
    .map(|s| to_deva(s))
    .collect()
}

// ---------------------------------------------------------------------------------------------
// Category ids
// ---------------------------------------------------------------------------------------------

macro_rules! id_enum {
  ($(#[$m:meta])* $name:ident => $target:ty { $($v:ident => $t:expr),* $(,)? }) => {
    $(#[$m])*
    #[derive(
      Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema, schemars::JsonSchema,
    )]
    #[serde(rename_all = "snake_case")]
    pub enum $name { $($v),* }
    impl $name {
      pub const ALL: &'static [$name] = &[$($name::$v),*];
    }
    impl From<$name> for $target {
      fn from(v: $name) -> $target {
        match v { $($name::$v => $t),* }
      }
    }
  };
}

id_enum!(
  /// Lakāra (tense / mood). The Vedic leṭ is left out.
  LakaraId => Lakara {
    Lat => Lakara::Lat, Lit => Lakara::Lit, Lut => Lakara::Lut, Lrt => Lakara::Lrt,
    Lot => Lakara::Lot, Lan => Lakara::Lan, VidhiLin => Lakara::VidhiLin,
    AshirLin => Lakara::AshirLin, Lun => Lakara::Lun, Lrn => Lakara::Lrn,
  }
);
id_enum!(PrayogaId => Prayoga {
  Kartari => Prayoga::Kartari, Karmani => Prayoga::Karmani, Bhave => Prayoga::Bhave,
});
id_enum!(PadaId => DhatuPada {
  Parasmaipada => DhatuPada::Parasmaipada, Atmanepada => DhatuPada::Atmanepada,
});
id_enum!(PurushaId => Purusha {
  Prathama => Purusha::Prathama, Madhyama => Purusha::Madhyama, Uttama => Purusha::Uttama,
});
id_enum!(VacanaId => Vacana { Eka => Vacana::Eka, Dvi => Vacana::Dvi, Bahu => Vacana::Bahu });
id_enum!(LingaId => Linga { Pum => Linga::Pum, Stri => Linga::Stri, Napumsaka => Linga::Napumsaka });
id_enum!(VibhaktiId => Vibhakti {
  Prathama => Vibhakti::Prathama, Dvitiya => Vibhakti::Dvitiya, Trtiya => Vibhakti::Trtiya,
  Caturthi => Vibhakti::Caturthi, Panchami => Vibhakti::Panchami, Sasthi => Vibhakti::Sasthi,
  Saptami => Vibhakti::Saptami, Sambodhana => Vibhakti::Sambodhana,
});
id_enum!(
  /// Sanādi pratyayas that make a new root from a root (the nāmadhātu ones are not exposed).
  SanadiId => Sanadi { Nic => Sanadi::Ric, San => Sanadi::san, Yan => Sanadi::yaN, YanLuk => Sanadi::yaNluk }
);
id_enum!(SamasaKind => SamasaType {
  Tatpurusha => SamasaType::Tatpurusha, Karmadharaya => SamasaType::Karmadharaya,
  Dvigu => SamasaType::Dvigu, Bahuvrihi => SamasaType::Bahuvrihi, Dvandva => SamasaType::Dvandva,
  SamaharaDvandva => SamasaType::SamaharaDvandva, Avyayibhava => SamasaType::Avyayibhava,
});

fn linga_id(l: Linga) -> LingaId {
  match l {
    Linga::Pum => LingaId::Pum,
    Linga::Stri => LingaId::Stri,
    Linga::Napumsaka => LingaId::Napumsaka,
  }
}

// ---------------------------------------------------------------------------------------------
// Dhātupāṭha
// ---------------------------------------------------------------------------------------------

/// A root of vidyut's Dhātupāṭha.
#[derive(Serialize, ToSchema, Clone)]
pub struct RootInfo {
  /// Dhātupāṭha code `GG.NNNN` (e.g. `01.0001`).
  pub code: String,
  pub gana: u8,
  pub number: u16,
  /// Aupadeśika form with it-markers and accents (e.g. `एधँ॒`).
  pub aupadeshika: String,
  /// The root after it-lopa (e.g. `एध्`); several when it-lopa allows options.
  pub roots: Vec<String>,
  pub artha: String,
}

impl RootInfo {
  fn of(r: &Root) -> Self {
    RootInfo {
      code: r.code.clone(),
      gana: r.gana,
      number: r.number,
      aupadeshika: to_deva(&r.aupadeshika),
      roots: r.clean.iter().map(|c| to_deva(c)).collect(),
      artha: to_deva(&r.artha),
    }
  }
}

/// `01.0001`, `1.1`, `10.252` → `GG.NNNN` (the site's own Dhātupāṭha numbers use `G.N`).
pub fn normalize_code(code: &str) -> Option<String> {
  let (g, n) = code.trim().split_once('.')?;
  let g: u8 = g.parse().ok()?;
  let n: u16 = n.parse().ok()?;
  Some(format!("{g:02}.{n:04}"))
}

fn root_by_code(code: &str) -> Option<&'static Root> {
  let e = engine();
  normalize_code(code)
    .and_then(|c| e.by_code.get(&c))
    .map(|&i| &e.roots[i])
}

pub fn root(code: &str) -> Option<RootInfo> {
  root_by_code(code).map(RootInfo::of)
}

/// Roots matching a Devanagari root (`भू`, `एधँ`), part of one, a meaning word, or a code
/// (`1.1`). Exact roots first; `gana` narrows to one gaṇa. Empty query lists the gaṇa in order.
pub fn search_roots(q: &str, gana: Option<u8>, limit: usize) -> Result<Vec<RootInfo>> {
  let e = engine();
  let in_gana = |r: &&Root| gana.is_none_or(|g| r.gana == g);
  let q = q.trim();
  if q.is_empty() {
    return Ok(
      e.roots
        .iter()
        .filter(in_gana)
        .take(limit)
        .map(RootInfo::of)
        .collect(),
    );
  }
  if q.chars().next().is_some_and(|c| c.is_ascii_digit()) {
    return Ok(root_by_code(q).map(RootInfo::of).into_iter().collect());
  }
  let key = word_slp1(q, "root")?;
  let mut hits: Vec<(u8, usize)> = e
    .roots
    .iter()
    .enumerate()
    .filter(|(_, r)| in_gana(r))
    .filter_map(|(i, r)| {
      let up = letters(&r.aupadeshika);
      let rank = if r.clean.iter().any(|c| *c == key) {
        0
      } else if up == key {
        1
      } else if r.clean.iter().any(|c| c.starts_with(&key)) || up.starts_with(&key) {
        2
      } else if key.len() >= 3 && r.artha.contains(&key) {
        3
      } else {
        return None;
      };
      Some((rank, i))
    })
    .collect();
  hits.sort();
  Ok(
    hits
      .into_iter()
      .take(limit)
      .map(|(_, i)| RootInfo::of(&e.roots[i]))
      .collect(),
  )
}

/// A root of the site's own Dhātupāṭha (another edition: its numbering drifts from vidyut's).
pub struct SiteRoot<'a> {
  pub gana: u8,
  pub number: u16,
  /// Devanagari upadeśa (`डुकृञ्`).
  pub upadesha: &'a str,
  /// Plain root in SLP1 (`kf`).
  pub root_slp1: &'a str,
}

fn letters(s: &str) -> String {
  s.chars().filter(|c| c.is_ascii_alphabetic()).collect()
}

/// Same root in both editions: same gaṇa, and the upadeśa or the plain root agrees.
fn same_root(r: &Root, site: &SiteRoot) -> bool {
  if r.gana != site.gana {
    return false;
  }
  let up = super::to_slp1(site.upadesha)
    .map(|s| letters(&s))
    .unwrap_or_default();
  (!up.is_empty() && letters(&r.aupadeshika) == up) || r.clean.iter().any(|c| c == site.root_slp1)
}

/// How far apart two roots of a gaṇa are (0 = same number): ranks candidates.
fn distance(a: u16, b: u16) -> u16 {
  a.abs_diff(b)
}

/// vidyut's root for a root of the site's Dhātupāṭha: the same number if it is the same root,
/// else the nearest agreeing root of the gaṇa. `None` rather than a wrong root.
pub fn match_site_root(site: &SiteRoot) -> Option<RootInfo> {
  engine()
    .roots
    .iter()
    .filter(|r| same_root(r, site))
    .min_by_key(|r| distance(r.number, site.number))
    .map(RootInfo::of)
}

/// The reverse: which of the site's roots (of the root's gaṇa) is this vidyut root.
pub fn pick_site_root<'a, T>(
  code: &str,
  candidates: impl IntoIterator<Item = (T, SiteRoot<'a>)>,
) -> Option<T> {
  let r = root_by_code(code)?;
  candidates
    .into_iter()
    .filter(|(_, s)| same_root(r, s))
    .min_by_key(|(_, s)| distance(r.number, s.number))
    .map(|(t, _)| t)
}

// ---------------------------------------------------------------------------------------------
// Arguments
// ---------------------------------------------------------------------------------------------

/// The 22 upasargas in SLP1 (आङ् is `AN`; its it-marker matters to the grammar).
const UPASARGAS: &[&str] = &[
  "pra", "parA", "apa", "sam", "anu", "ava", "nis", "nir", "dus", "dur", "vi", "AN", "ni", "aDi",
  "api", "ati", "su", "ud", "aBi", "prati", "pari", "upa",
];

/// A root from the Dhātupāṭha, optionally with sanādi suffixes (in order: `[nic, san]` is the
/// desiderative of the causative) and upasargas (Devanagari: `प्र`, `सम्`, `आ`).
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct DhatuArgs {
  pub code: String,
  #[serde(default)]
  pub sanadi: Vec<SanadiId>,
  #[serde(default)]
  pub upasargas: Vec<String>,
}

impl DhatuArgs {
  fn build(&self) -> Result<Dhatu> {
    let Some(root) = root_by_code(&self.code) else {
      return err(format!("Unknown Dhātupāṭha code {}", self.code));
    };
    let mut prefixes = Vec::new();
    for u in &self.upasargas {
      let s = word_slp1(u, "upasarga")?;
      let s = if s == "A" { "AN".to_string() } else { s };
      if !UPASARGAS.contains(&s.as_str()) {
        return err(format!("{u} is not an upasarga"));
      }
      prefixes.push(s);
    }
    let sanadi: Vec<Sanadi> = self.sanadi.iter().map(|&s| s.into()).collect();
    Ok(
      root
        .dhatu
        .clone()
        .with_sanadi(&sanadi)
        .with_prefixes(&prefixes),
    )
  }
}

fn basic(stem: &str) -> Result<Pratipadika> {
  let s = word_slp1(stem, "stem")?;
  Ok(Pratipadika::basic(
    Slp1String::from(s).map_err(|e| super::InputError(e.to_string()))?,
  ))
}

fn avyaya(stem: &str) -> Result<Pratipadika> {
  let s = word_slp1(stem, "word")?;
  Ok(Pratipadika::avyaya(
    Slp1String::from(s).map_err(|e| super::InputError(e.to_string()))?,
  ))
}

fn parse_krt(s: &str) -> Result<BaseKrt> {
  BaseKrt::from_str(s).or_else(|_| err(format!("Unknown kṛt suffix {s}")))
}

fn parse_taddhita(s: &str) -> Result<Taddhita> {
  Taddhita::from_str(s).or_else(|_| err(format!("Unknown taddhita suffix {s}")))
}

/// One member of a compound.
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct SamasaPada {
  /// Devanagari stem (`राजन्`), or an indeclinable for the first member of an avyayībhāva.
  pub stem: String,
  #[serde(default = "default_vibhakti")]
  pub vibhakti: VibhaktiId,
  #[serde(default = "default_linga")]
  pub linga: LingaId,
}

fn default_vibhakti() -> VibhaktiId {
  VibhaktiId::Prathama
}
fn default_linga() -> LingaId {
  LingaId::Pum
}

/// A nominal stem (prātipadika): typed as is, or derived (kṛdanta, taddhitānta, samāsa).
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StemArgs {
  Basic {
    stem: String,
  },
  Krdanta {
    dhatu: DhatuArgs,
    /// vidyut kṛt label (SLP1), e.g. `kta`, `Satf~`.
    krt: String,
    /// Only for śatṛ / śānac (laṭ or lṛṭ).
    #[serde(default)]
    lakara: Option<LakaraId>,
    /// Only for śatṛ / śānac (karmaṇi śānac = present passive participle).
    #[serde(default)]
    prayoga: Option<PrayogaId>,
  },
  Taddhitanta {
    stem: String,
    /// vidyut taddhita label (SLP1), e.g. `aR`, `tva`.
    taddhita: String,
  },
  Samasa {
    padas: Vec<SamasaPada>,
    samasa_type: SamasaKind,
  },
}

fn krdanta(
  dhatu: &DhatuArgs,
  krt: &str,
  lakara: Option<LakaraId>,
  prayoga: Option<PrayogaId>,
) -> Result<Krdanta> {
  let mut b = Krdanta::builder()
    .dhatu(dhatu.build()?)
    .krt(parse_krt(krt)?);
  if let Some(l) = lakara {
    b = b.lakara(l.into());
  }
  if let Some(p) = prayoga {
    b = b.prayoga(p.into());
  }
  b.build().map_err(|e| super::InputError(e.to_string()))
}

fn samasa(padas: &[SamasaPada], kind: SamasaKind) -> Result<Samasa> {
  if padas.len() < 2 {
    return err("A compound needs at least two members");
  }
  let mut subs = Vec::new();
  for (i, p) in padas.iter().enumerate() {
    if i == 0 && kind == SamasaKind::Avyayibhava {
      subs.push(Subanta::avyaya(avyaya(&p.stem)?));
    } else {
      subs.push(Subanta::new(
        basic(&p.stem)?,
        p.linga.into(),
        p.vibhakti.into(),
        Vacana::Eka,
      ));
    }
  }
  Samasa::builder()
    .padas(subs)
    .samasa_type(kind.into())
    .build()
    .map_err(|e| super::InputError(e.to_string()))
}

impl StemArgs {
  fn build(&self) -> Result<Pratipadika> {
    Ok(match self {
      StemArgs::Basic { stem } => basic(stem)?,
      StemArgs::Krdanta {
        dhatu,
        krt,
        lakara,
        prayoga,
      } => krdanta(dhatu, krt, *lakara, *prayoga)?.into(),
      StemArgs::Taddhitanta { stem, taddhita } => {
        Taddhitanta::new(basic(stem)?, parse_taddhita(taddhita)?).into()
      }
      StemArgs::Samasa { padas, samasa_type } => samasa(padas, *samasa_type)?.into(),
    })
  }
}

/// The stem's own forms: the typed stem, or what the derivation produces (`कृ + क्त` → `कृत`).
fn stem_texts(stem: &StemArgs) -> Result<Vec<String>> {
  let e = engine();
  let p = stem.build()?;
  Ok(match &p {
    Pratipadika::Basic(_) => vec![to_deva(&word_slp1(
      match stem {
        StemArgs::Basic { stem } => stem,
        _ => "",
      },
      "stem",
    )?)],
    Pratipadika::Krdanta(k) => texts(&e.fast.derive_krdantas(k)),
    Pratipadika::Taddhitanta(t) => texts(&e.fast.derive_taddhitantas(t)),
    Pratipadika::Samasa(s) => texts(&e.fast.derive_samasas(s)),
  })
}

// ---------------------------------------------------------------------------------------------
// Tiṅanta
// ---------------------------------------------------------------------------------------------

/// `forms[purusha][vacana]` = the variants for that cell (puruṣa: prathama, madhyama, uttama;
/// vacana: eka, dvi, bahu).
#[derive(Serialize, ToSchema)]
pub struct LakaraTable {
  pub lakara: LakaraId,
  pub forms: Vec<Vec<Vec<String>>>,
}

#[derive(Serialize, ToSchema)]
pub struct PadaTables {
  pub pada: PadaId,
  pub lakaras: Vec<LakaraTable>,
}

#[derive(Serialize, ToSchema)]
pub struct Conjugation {
  pub root: RootInfo,
  /// The root after sanādi / upasarga (e.g. भावि for भू + णिच्).
  pub stems: Vec<String>,
  pub prayoga: PrayogaId,
  /// One set of tables per pada the root takes in this prayoga (ubhayapadī roots get both).
  pub tables: Vec<PadaTables>,
}

/// Every lakāra of a root in one prayoga.
pub fn conjugate(args: &DhatuArgs, prayoga: PrayogaId) -> Result<Conjugation> {
  let e = engine();
  let dhatu = args.build()?;
  let root = RootInfo::of(root_by_code(&args.code).expect("checked by build"));
  let stems = texts(&e.fast.derive_dhatus(&dhatu));
  let mut tables = Vec::new();
  for &pada in PadaId::ALL {
    let mut lakaras = Vec::new();
    for &lakara in LakaraId::ALL {
      let forms: Vec<Vec<Vec<String>>> = PurushaId::ALL
        .iter()
        .map(|&purusha| {
          VacanaId::ALL
            .iter()
            .map(|&vacana| {
              let t = Tinanta::builder()
                .dhatu(dhatu.clone())
                .prayoga(prayoga.into())
                .lakara(lakara.into())
                .purusha(purusha.into())
                .vacana(vacana.into())
                .pada(pada.into())
                .build()
                .expect("all tinanta fields set");
              texts(&e.fast.derive_tinantas(&t))
            })
            .collect()
        })
        .collect();
      if forms.iter().flatten().any(|c| !c.is_empty()) {
        lakaras.push(LakaraTable { lakara, forms });
      }
    }
    if !lakaras.is_empty() {
      tables.push(PadaTables { pada, lakaras });
    }
  }
  if tables.is_empty() {
    return err("vidyut has no forms for this root in this prayoga");
  }
  Ok(Conjugation {
    root,
    stems,
    prayoga,
    tables,
  })
}

// ---------------------------------------------------------------------------------------------
// Subanta
// ---------------------------------------------------------------------------------------------

#[derive(Serialize, ToSchema)]
pub struct Declension {
  /// The stem the table is built on (derived stems spelled out).
  pub stems: Vec<String>,
  pub linga: LingaId,
  /// `forms[vibhakti][vacana]`, vibhaktis prathamā … saptamī then sambodhana.
  pub forms: Vec<Vec<Vec<String>>>,
}

pub fn decline(stem: &StemArgs, linga: LingaId) -> Result<Declension> {
  let e = engine();
  let p = stem.build()?;
  let forms: Vec<Vec<Vec<String>>> = VibhaktiId::ALL
    .iter()
    .map(|&vibhakti| {
      VacanaId::ALL
        .iter()
        .map(|&vacana| {
          let s = Subanta::new(p.clone(), linga.into(), vibhakti.into(), vacana.into());
          texts(&e.fast.derive_subantas(&s))
        })
        .collect()
    })
    .collect();
  if forms.iter().flatten().all(|c| c.is_empty()) {
    return err("vidyut could not decline this stem in this gender");
  }
  Ok(Declension {
    stems: stem_texts(stem)?,
    linga,
    forms,
  })
}

/// Feminine stems of a (masculine) stem: अज → अजा, गौर → गौरी, राजन् → राज्ञी.
pub fn stryantas(stem: &StemArgs) -> Result<Vec<String>> {
  let e = engine();
  Ok(texts(&e.fast.derive_stryantas(&stem.build()?)))
}

// ---------------------------------------------------------------------------------------------
// Kṛdanta
// ---------------------------------------------------------------------------------------------

#[derive(Serialize, ToSchema)]
pub struct KrdantaForm {
  /// vidyut kṛt label (SLP1) — pass back as `krt` in [`StemArgs::Krdanta`].
  pub krt: String,
  /// The suffix in Devanagari (`क्त`, `शतृ`).
  pub label: String,
  pub lakara: Option<LakaraId>,
  pub prayoga: Option<PrayogaId>,
  /// Short English description (curated suffixes only).
  pub gloss: Option<String>,
  pub forms: Vec<String>,
  /// Genders the stem is declined in (empty for indeclinables).
  pub lingas: Vec<LingaId>,
  pub avyaya: bool,
}

#[derive(Serialize, ToSchema)]
pub struct KrtGroup {
  pub key: String,
  pub label: String,
  pub items: Vec<KrdantaForm>,
}

type KrtSpec = (BaseKrt, Option<Lakara>, Option<Prayoga>, &'static str);

const PARTICIPLES: &[KrtSpec] = &[
  (BaseKrt::kta, None, None, "past passive participle"),
  (BaseKrt::ktavatu, None, None, "past active participle"),
  (
    BaseKrt::Satf,
    Some(Lakara::Lat),
    None,
    "present active participle",
  ),
  (
    BaseKrt::SAnac,
    Some(Lakara::Lat),
    None,
    "present middle participle",
  ),
  (
    BaseKrt::SAnac,
    Some(Lakara::Lat),
    Some(Prayoga::Karmani),
    "present passive participle",
  ),
  (
    BaseKrt::Satf,
    Some(Lakara::Lrt),
    None,
    "future active participle",
  ),
  (
    BaseKrt::SAnac,
    Some(Lakara::Lrt),
    None,
    "future middle participle",
  ),
  (BaseKrt::kvasu, None, None, "perfect active participle"),
];
const INDECLINABLES: &[KrtSpec] = &[
  (BaseKrt::ktvA, None, None, "gerund (ल्यप् after an upasarga)"),
  (BaseKrt::tumun, None, None, "infinitive"),
  (BaseKrt::Ramul, None, None, "repeated action"),
];
const GERUNDIVES: &[KrtSpec] = &[
  (BaseKrt::tavya, None, None, "to be done"),
  (BaseKrt::anIyar, None, None, "to be done"),
  (BaseKrt::yat, None, None, "to be done"),
  (BaseKrt::Ryat, None, None, "to be done"),
  (BaseKrt::kyap, None, None, "to be done"),
];
const NOUNS: &[KrtSpec] = &[
  (BaseKrt::tfc, None, None, "agent"),
  (BaseKrt::Rvul, None, None, "agent"),
  (BaseKrt::Rini, None, None, "agent, habitual"),
  (BaseKrt::lyuw, None, None, "action, instrument"),
  (BaseKrt::GaY, None, None, "action, state"),
  (BaseKrt::ktin, None, None, "action (feminine)"),
  (BaseKrt::ac, None, None, "agent"),
  (BaseKrt::ap, None, None, "action"),
  (BaseKrt::aR, None, None, "agent (with upapada)"),
];

fn krdanta_form(
  dhatu: &Dhatu,
  (krt, lakara, prayoga, gloss): KrtSpec,
  gloss_opt: bool,
) -> Option<KrdantaForm> {
  let e = engine();
  let mut b = Krdanta::builder().dhatu(dhatu.clone()).krt(krt);
  if let Some(l) = lakara {
    b = b.lakara(l);
  }
  if let Some(p) = prayoga {
    b = b.prayoga(p);
  }
  let k = b.build().ok()?;
  let forms = texts(&e.fast.derive_krdantas(&k));
  if forms.is_empty() {
    return None;
  }
  let lakara_id = lakara.and_then(|l| {
    LakaraId::ALL
      .iter()
      .copied()
      .find(|&x| Lakara::from(x) == l)
  });
  let prayoga_id = prayoga.and_then(|p| {
    PrayogaId::ALL
      .iter()
      .copied()
      .find(|&x| Prayoga::from(x) == p)
  });
  Some(KrdantaForm {
    krt: krt.as_str().to_string(),
    label: suffix_label(krt.as_str()),
    lakara: lakara_id,
    prayoga: prayoga_id,
    gloss: gloss_opt.then(|| gloss.to_string()),
    forms,
    lingas: krt_lingas(krt),
    avyaya: krt.is_avyaya(),
  })
}

/// Genders a kṛdanta is declined in. vidyut lists them only for some suffixes; the rest are
/// adjectival (all three) except action nouns with a fixed gender.
fn krt_lingas(krt: BaseKrt) -> Vec<LingaId> {
  use BaseKrt::*;
  if krt.is_avyaya() {
    return vec![];
  }
  let known: Vec<LingaId> = krt.lingas().iter().map(|&l| linga_id(l)).collect();
  if !known.is_empty() {
    return known;
  }
  match krt {
    lyuw => vec![LingaId::Napumsaka],
    GaY | ap | ac | aR | Ga | Kal => vec![LingaId::Pum],
    _ => LingaId::ALL.to_vec(),
  }
}

/// Kṛdantas of a root: the common ones grouped (participles, indeclinables, gerundives, nouns),
/// then every other (non-Vedic) kṛt suffix that applies.
pub fn krdantas(args: &DhatuArgs) -> Result<Vec<KrtGroup>> {
  let dhatu = args.build()?;
  let groups: [(&str, &str, &[KrtSpec]); 4] = [
    ("participles", "Participles", PARTICIPLES),
    ("avyaya", "Indeclinables", INDECLINABLES),
    ("gerundives", "Gerundives", GERUNDIVES),
    ("nouns", "Agent & action nouns", NOUNS),
  ];
  let mut out: Vec<KrtGroup> = groups
    .iter()
    .map(|(key, label, specs)| KrtGroup {
      key: key.to_string(),
      label: label.to_string(),
      items: specs
        .iter()
        .filter_map(|&s| krdanta_form(&dhatu, s, true))
        .collect(),
    })
    .collect();
  let curated: Vec<BaseKrt> = groups
    .iter()
    .flat_map(|(_, _, s)| s.iter().map(|x| x.0))
    .collect();
  let others: Vec<KrdantaForm> = BaseKrt::iter()
    .filter(|k| !curated.contains(k) && !k.is_chandasa() && !k.is_duplicate())
    .filter_map(|k| krdanta_form(&dhatu, (k, None, None, ""), false))
    .collect();
  out.push(KrtGroup {
    key: "other".into(),
    label: "Other kṛt suffixes".into(),
    items: others,
  });
  out.retain(|g| !g.items.is_empty());
  Ok(out)
}

// ---------------------------------------------------------------------------------------------
// Taddhita
// ---------------------------------------------------------------------------------------------

#[derive(Serialize, ToSchema)]
pub struct TaddhitaForm {
  /// vidyut taddhita label (SLP1) — pass back as `taddhita` in [`StemArgs::Taddhitanta`].
  pub taddhita: String,
  pub label: String,
  pub gloss: Option<String>,
  pub forms: Vec<String>,
  /// The gender to decline it in first (तल् → strī, त्व / ष्यञ् → napuṃsaka).
  pub linga: LingaId,
}

const COMMON_TADDHITAS: &[(Taddhita, &str)] = &[
  (Taddhita::aR, "descendant of, related to"),
  (Taddhita::iY, "patronymic"),
  (Taddhita::Qak, "descendant (of a feminine)"),
  (Taddhita::yat, "fit for, related to"),
  (Taddhita::Wak, "dealing with, made of"),
  (Taddhita::matup, "possessing"),
  (Taddhita::vini, "possessing"),
  (Taddhita::ini, "possessing"),
  (Taddhita::tva, "-ness (neuter)"),
  (Taddhita::tal, "-ness (feminine)"),
  (Taddhita::imanic, "-ness (masculine)"),
  (Taddhita::zyaY, "-ness, state"),
  (Taddhita::tarap, "comparative"),
  (Taddhita::tamap, "superlative"),
  (Taddhita::Iyasun, "comparative"),
  (Taddhita::izWan, "superlative"),
  (Taddhita::kan, "diminutive, 'a little'"),
  (Taddhita::mayaw, "made of, full of"),
  (Taddhita::vati, "like"),
  (Taddhita::tasil, "from (adverb)"),
];

/// Taddhitāntas of a stem: the common suffixes first (with a gloss), then every other one that
/// applies.
pub fn taddhitas(stem: &str) -> Result<Vec<TaddhitaForm>> {
  let e = engine();
  let p = basic(stem)?;
  let make = |t: Taddhita, gloss: Option<&str>| {
    let forms = texts(&e.fast.derive_taddhitantas(&Taddhitanta::new(p.clone(), t)));
    (!forms.is_empty()).then(|| TaddhitaForm {
      taddhita: t.as_str().to_string(),
      label: suffix_label(t.as_str()),
      gloss: gloss.map(str::to_string),
      forms,
      linga: match t {
        Taddhita::tal => LingaId::Stri,
        Taddhita::tva | Taddhita::zyaY => LingaId::Napumsaka,
        _ => LingaId::Pum,
      },
    })
  };
  let mut out: Vec<TaddhitaForm> = COMMON_TADDHITAS
    .iter()
    .filter_map(|&(t, g)| make(t, Some(g)))
    .collect();
  out.extend(
    Taddhita::iter()
      .filter(|t| !COMMON_TADDHITAS.iter().any(|(c, _)| c == t))
      .filter_map(|t| make(t, None)),
  );
  Ok(out)
}

// ---------------------------------------------------------------------------------------------
// Samāsa
// ---------------------------------------------------------------------------------------------

#[derive(Serialize, ToSchema)]
pub struct SamasaWord {
  pub linga: LingaId,
  pub forms: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct SamasaResult {
  /// The compound stem(s).
  pub stems: Vec<String>,
  /// Nominative in each gender the compound takes (dual for itaretara-dvandva; the indeclinable
  /// for avyayībhāva).
  pub words: Vec<SamasaWord>,
}

pub fn compound(padas: &[SamasaPada], kind: SamasaKind) -> Result<SamasaResult> {
  let e = engine();
  let s = samasa(padas, kind)?;
  let stems = texts(&e.fast.derive_samasas(&s));
  if stems.is_empty() {
    return err("vidyut could not form this compound");
  }
  let lingas: &[LingaId] = match kind {
    SamasaKind::Avyayibhava | SamasaKind::SamaharaDvandva => &[LingaId::Napumsaka],
    _ => LingaId::ALL,
  };
  let vacana = if kind == SamasaKind::Dvandva {
    Vacana::Dvi
  } else {
    Vacana::Eka
  };
  let words = lingas
    .iter()
    .map(|&l| {
      let sub = Subanta::new(s.clone(), l.into(), Vibhakti::Prathama, vacana);
      SamasaWord {
        linga: l,
        forms: texts(&e.fast.derive_subantas(&sub)),
      }
    })
    .filter(|w| !w.forms.is_empty())
    .collect();
  Ok(SamasaResult { stems, words })
}

// ---------------------------------------------------------------------------------------------
// Sandhi between words (vākya)
// ---------------------------------------------------------------------------------------------

/// Joins words with the grammar's sandhi rules (`रामः` `गच्छति` → `रामो गच्छति`).
pub fn join_words(words: &[String]) -> Result<Vec<String>> {
  let words = vakya_words(words)?;
  let ps = engine().full.derive_vakyas(&padas(&words));
  Ok(dedup(ps.iter().map(|p| spaced(p, &words))))
}

/// The words in SLP1, with pada-final visarga / anusvāra written as the s / m they come from
/// (the sandhi rules start from those).
fn vakya_words(words: &[String]) -> Result<Vec<String>> {
  let words: Vec<&String> = words.iter().filter(|w| !w.trim().is_empty()).collect();
  if words.len() < 2 {
    return err("Enter at least two words");
  }
  words
    .iter()
    .map(|w| {
      let s = word_slp1(w, "word")?;
      Ok(if let Some(x) = s.strip_suffix('H') {
        format!("{x}s")
      } else if let Some(x) = s.strip_suffix('M') {
        format!("{x}m")
      } else {
        s
      })
    })
    .collect()
}

fn padas(words: &[String]) -> Vec<Pada> {
  words.iter().map(Pada::from_text).collect()
}

/// The joined phrase as it is usually printed: words stay apart after a vowel, visarga or
/// anusvāra and run on after a consonant (तच्च, इत्यपि, देवेन्द्रः); an elided अ is written as
/// avagraha (रामोऽस्ति). Relies on the last step keeping one term per word.
fn spaced(p: &Prakriya, words: &[String]) -> String {
  let terms: Vec<&str> = match p.history().last() {
    Some(step) => step
      .result()
      .iter()
      .map(|t| t.text())
      .filter(|t| !t.is_empty())
      .collect(),
    None => vec![],
  };
  if terms.len() != words.len() {
    return to_deva(&p.text());
  }
  let mut out = String::new();
  for (i, t) in terms.iter().enumerate() {
    if i > 0 {
      let prev = terms[i - 1].chars().last();
      if words[i].starts_with('a') && !t.starts_with('a') && matches!(prev, Some('e' | 'o')) {
        out.push('\'');
      } else if prev.is_some_and(|c| "aAiIuUfFxXeEoOHM".contains(c)) {
        out.push(' ');
      }
    }
    out.push_str(t);
  }
  to_deva(&out)
}

// ---------------------------------------------------------------------------------------------
// Derivations
// ---------------------------------------------------------------------------------------------

/// A word (or stem / root / phrase) to derive step by step.
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WordSpec {
  Tinanta {
    dhatu: DhatuArgs,
    prayoga: PrayogaId,
    lakara: LakaraId,
    purusha: PurushaId,
    vacana: VacanaId,
    #[serde(default)]
    pada: Option<PadaId>,
  },
  Subanta {
    stem: StemArgs,
    linga: LingaId,
    vibhakti: VibhaktiId,
    vacana: VacanaId,
  },
  /// The prātipadika itself (kṛdanta, taddhitānta, samāsa).
  Stem { stem: StemArgs },
  /// Feminine stem.
  Stryanta { stem: StemArgs },
  /// The root with its sanādi / upasargas.
  Dhatu { dhatu: DhatuArgs },
  /// Words joined by sandhi.
  Vakya { words: Vec<String> },
}

#[derive(Serialize, ToSchema, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuleSource {
  Ashtadhyayi,
  Varttika,
  Dhatupatha,
  Unadipatha,
  Linganushasana,
  Phit,
  Kashika,
  Kaumudi,
  Other,
}

#[derive(Serialize, ToSchema)]
pub struct RuleRef {
  pub source: RuleSource,
  /// e.g. `3.2.123`, `1.1.33.1` (vārttika), `2473` (Kaumudī).
  pub code: String,
  pub text: Option<String>,
  /// This site's sūtra id (`3.2.123` → `3.2.123`, `1.1.1` → `1.1.001`) for Aṣṭādhyāyī rules and
  /// for vārttikas / Kāśikā notes on one.
  pub sutra_id: Option<String>,
  /// Link for references outside the shipped texts.
  pub url: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct StepTerm {
  pub text: String,
  /// Changed by this step.
  pub changed: bool,
}

#[derive(Serialize, ToSchema)]
pub struct Step {
  pub rule: RuleRef,
  pub terms: Vec<StepTerm>,
}

#[derive(Serialize, ToSchema)]
pub struct OptionalRule {
  pub rule: RuleRef,
  /// Whether this derivation applied the optional rule.
  pub applied: bool,
}

#[derive(Serialize, ToSchema)]
pub struct Derivation {
  pub text: String,
  pub steps: Vec<Step>,
  /// The optional rules this result depends on (why a cell has several forms).
  pub optional_rules: Vec<OptionalRule>,
}

/// `1.1.1` → `1.1.001` (the site's sūtra ids); `1.1.33.1` → `1.1.033`.
fn sutra_id(code: &str) -> Option<String> {
  let mut it = code.split('.');
  let a: u8 = it.next()?.parse().ok()?;
  let p: u8 = it.next()?.parse().ok()?;
  let s: u16 = it.next()?.parse().ok()?;
  Some(format!("{a}.{p}.{s:03}"))
}

fn rule_ref(rule: Rule) -> RuleRef {
  let t = &engine().texts;
  let code = rule.code();
  let (source, map) = match rule {
    Rule::Ashtadhyayi(_) => (RuleSource::Ashtadhyayi, Some(&t.ashtadhyayi)),
    Rule::Varttika(_) => (RuleSource::Varttika, Some(&t.varttika)),
    Rule::Dhatupatha(_) => (RuleSource::Dhatupatha, Some(&t.dhatupatha)),
    Rule::Unadipatha(_) => (RuleSource::Unadipatha, Some(&t.unadi)),
    Rule::Linganushasana(_) => (RuleSource::Linganushasana, Some(&t.linganushasana)),
    Rule::Phit(_) => (RuleSource::Phit, Some(&t.phit)),
    Rule::Kashika(_) => (RuleSource::Kashika, Some(&t.kashika)),
    Rule::Kaumudi(_) => (RuleSource::Kaumudi, Some(&t.kaumudi)),
    Rule::Anyatra(_) => (RuleSource::Other, None),
  };
  if source == RuleSource::Other {
    let (text, url) = match code.rsplit_once(", ") {
      Some((d, u)) if u.starts_with("http") => (d, Some(u.to_string())),
      _ => (code, None),
    };
    return RuleRef {
      source,
      code: String::new(),
      text: Some(text.to_string()),
      sutra_id: None,
      url,
    };
  }
  let links_sutra = matches!(
    source,
    RuleSource::Ashtadhyayi | RuleSource::Varttika | RuleSource::Kashika
  );
  RuleRef {
    source,
    code: code.to_string(),
    text: map.and_then(|m| m.get(code)).map(|s| to_deva(s)),
    sutra_id: if links_sutra { sutra_id(code) } else { None },
    url: None,
  }
}

fn derivation(p: &Prakriya) -> Derivation {
  Derivation {
    text: to_deva(&p.text()),
    steps: p
      .history()
      .iter()
      .map(|s| Step {
        rule: rule_ref(s.rule()),
        terms: s
          .result()
          .iter()
          .filter(|t| !t.text().is_empty())
          .map(|t| StepTerm {
            text: to_deva(t.text()),
            changed: t.was_changed(),
          })
          .collect(),
      })
      .collect(),
    optional_rules: p
      .rule_choices()
      .iter()
      .map(|c| OptionalRule {
        rule: rule_ref(c.rule()),
        applied: c.decision() == Decision::Accept,
      })
      .collect(),
  }
}

/// Every derivation of a word (one per result; optional rules can give several).
pub fn derive(spec: &WordSpec) -> Result<Vec<Derivation>> {
  let v = &engine().full;
  let ps = match spec {
    WordSpec::Tinanta {
      dhatu,
      prayoga,
      lakara,
      purusha,
      vacana,
      pada,
    } => {
      let mut b = Tinanta::builder()
        .dhatu(dhatu.build()?)
        .prayoga((*prayoga).into())
        .lakara((*lakara).into())
        .purusha((*purusha).into())
        .vacana((*vacana).into());
      if let Some(p) = pada {
        b = b.pada((*p).into());
      }
      v.derive_tinantas(&b.build().map_err(|e| super::InputError(e.to_string()))?)
    }
    WordSpec::Subanta {
      stem,
      linga,
      vibhakti,
      vacana,
    } => v.derive_subantas(&Subanta::new(
      stem.build()?,
      (*linga).into(),
      (*vibhakti).into(),
      (*vacana).into(),
    )),
    WordSpec::Stem { stem } => v.derive_pratipadikas(&stem.build()?),
    WordSpec::Stryanta { stem } => v.derive_stryantas(&stem.build()?),
    WordSpec::Dhatu { dhatu } => v.derive_dhatus(&dhatu.build()?),
    WordSpec::Vakya { words } => {
      let words = vakya_words(words)?;
      let ps = v.derive_vakyas(&padas(&words));
      return Ok(
        ps.iter()
          .map(|p| Derivation {
            text: spaced(p, &words),
            ..derivation(p)
          })
          .collect(),
      );
    }
  };
  Ok(ps.iter().map(derivation).collect())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn slp(deva: &str) -> String {
    crate::vidyut::to_slp1(deva).unwrap()
  }

  fn args(code: &str) -> DhatuArgs {
    DhatuArgs {
      code: code.into(),
      sanadi: vec![],
      upasargas: vec![],
    }
  }

  #[test]
  fn search_finds_roots() {
    let r = search_roots("भू", None, 5).unwrap();
    assert_eq!(r[0].code, "01.0001");
    assert_eq!(search_roots("10.252", None, 5).unwrap()[0].code, "10.0252");
    assert!(search_roots("bhU", None, 5).is_err());
  }

  #[test]
  fn conjugates_ubhayapadi() {
    let c = conjugate(&args("08.0010"), PrayogaId::Kartari).unwrap();
    assert_eq!(c.tables.len(), 2);
    assert_eq!(c.tables[0].lakaras[0].forms[0][0], vec!["करोति"]);
    assert_eq!(c.tables[1].lakaras[0].forms[0][0], vec!["कुरुते"]);
    let p = conjugate(
      &DhatuArgs {
        upasargas: vec!["प्र".into()],
        sanadi: vec![SanadiId::Nic],
        ..args("01.0001")
      },
      PrayogaId::Kartari,
    )
    .unwrap();
    assert_eq!(p.tables[0].lakaras[0].forms[0][0], vec!["प्रभावयति"]);
  }

  #[test]
  fn declines_and_derives() {
    let d = decline(
      &StemArgs::Basic {
        stem: "राम".into()
      },
      LingaId::Pum,
    )
    .unwrap();
    assert_eq!(d.forms[2][0], vec!["रामेण"]);
    assert_eq!(d.forms[4][0], vec!["रामात्"]);
    let kta = StemArgs::Krdanta {
      dhatu: args("08.0010"),
      krt: "kta".into(),
      lakara: None,
      prayoga: None,
    };
    assert_eq!(decline(&kta, LingaId::Stri).unwrap().stems, vec!["कृत"]);
    let steps = derive(&WordSpec::Tinanta {
      dhatu: args("01.0001"),
      prayoga: PrayogaId::Kartari,
      lakara: LakaraId::Lat,
      purusha: PurushaId::Prathama,
      vacana: VacanaId::Eka,
      pada: None,
    })
    .unwrap();
    assert_eq!(steps[0].text, "भवति");
    let r = steps[0]
      .steps
      .iter()
      .find(|s| s.rule.code == "3.2.123")
      .unwrap();
    assert_eq!(r.rule.sutra_id.as_deref(), Some("3.2.123"));
    assert_eq!(r.rule.text.as_deref(), Some("वर्तमाने लट्"));
    assert_eq!(sutra_id("1.1.1").as_deref(), Some("1.1.001"));
  }

  #[test]
  fn joins_words() {
    let j = |a: &str, b: &str| join_words(&[a.into(), b.into()]).unwrap();
    assert_eq!(j("रामः", "गच्छति"), vec!["रामो गच्छति"]);
    assert_eq!(j("रामः", "अस्ति"), vec!["रामोऽस्ति"]);
    assert_eq!(j("देव", "इन्द्रः"), vec!["देवेन्द्रः"]);
    assert_eq!(j("तत्", "च"), vec!["तच्च"]);
    assert_eq!(j("ततः", "एव"), vec!["तत एव"]);
    assert_eq!(slp("देवेन्द्रः"), "devendraH");
  }
}
