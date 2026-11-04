//! Sandhi between words (vidyut-sandhi): the rule table and phonetic split candidates.
//!
//! Joining words is done by the grammar instead ([`super::prakriya::join_words`]), which knows
//! more than the phonetic table (e.g. वाक् + ईशः → वागीशः).
//!
//! Splitting is purely phonetic: every way a chunk could have come from two words. It
//! over-generates on purpose; the API marks the parts that are words in the dictionaries.

use super::kosha::{self, PadaInfo};
use super::{Result, err, to_deva, to_slp1};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use utoipa::ToSchema;
use vidyut_sandhi::{Kind, SplitsMap, Splitter};

const RULES_CSV: &str = include_str!("data/sandhi/rules.csv");

/// (first, second, result) in SLP1.
static RULES: LazyLock<Vec<(&'static str, &'static str, &'static str)>> = LazyLock::new(|| {
  RULES_CSV
    .lines()
    .skip(1)
    .filter_map(|l| {
      let mut it = l.splitn(3, ',');
      Some((it.next()?, it.next()?, it.next()?))
    })
    .collect()
});

/// Same as `Splitter::from_csv` (which only reads files): each result maps back to its pair, and a
/// result with a space also matches without it.
static SPLITTER: LazyLock<Splitter> = LazyLock::new(|| {
  let mut map = SplitsMap::new();
  for &(first, second, result) in RULES.iter() {
    map.insert(result.to_string(), (first.to_string(), second.to_string()));
    let joined = result.replace(' ', "");
    if joined != result {
      map.insert(joined, (first.to_string(), second.to_string()));
    }
  }
  Splitter::from_map(map)
});

/// The splitter over the embedded rule table (shared with cheda).
pub(crate) fn splitter() -> &'static Splitter {
  &SPLITTER
}

#[derive(Serialize, ToSchema)]
pub struct SandhiRule {
  /// End of the first word.
  pub first: String,
  /// Start of the second word.
  pub second: String,
  pub result: String,
}

/// The whole rule table (vowels, visarga, final m / n and stops), in Devanagari.
pub fn rules() -> Vec<SandhiRule> {
  RULES
    .iter()
    .map(|&(f, s, r)| SandhiRule {
      first: sound(f),
      second: sound(s),
      result: to_deva(r),
    })
    .collect()
}

/// A rule side in Devanagari: lone vowels as independent letters (अ, इ), consonants with virāma.
fn sound(slp1: &str) -> String {
  to_deva(slp1)
}

#[derive(Serialize, ToSchema, Clone)]
pub struct SplitCandidate {
  pub left: String,
  pub right: String,
}

/// A word-final `s` / `r` (how the table writes visarga) reads as visarga.
pub fn pada_final(slp1: &str) -> String {
  match slp1.strip_suffix(['s', 'r']) {
    Some(x) if !x.is_empty() => format!("{x}H"),
    _ => slp1.to_string(),
  }
}

/// Most chunks split at once (a śloka).
pub const MAX_CHUNKS: usize = 16;

/// The text's space-separated chunks in SLP1 (letters and avagraha only).
pub fn chunks(text: &str) -> Result<Vec<String>> {
  let slp = to_slp1(text)?;
  let chunks: Vec<String> = slp
    .split_whitespace()
    .map(|c| {
      c.chars()
        .filter(|&c| c.is_ascii_alphabetic() || c == '\'')
        .collect::<String>()
    })
    .filter(|c| !c.is_empty())
    .collect();
  if chunks.is_empty() {
    return err("Enter a word or phrase");
  }
  if chunks.len() > MAX_CHUNKS {
    return err("Enter at most sixteen words (a verse) at a time");
  }
  Ok(chunks)
}

/// Dictionary key of a part: its letters (no avagraha).
pub fn word_key(slp1: &str) -> String {
  slp1.chars().filter(|c| c.is_ascii_alphabetic()).collect()
}

const VOWELS: &str = "aAiIuUfFxXeEoO";

fn is_vowel(c: char) -> bool {
  VOWELS.contains(c)
}

/// Every two-word split of a chunk that undoes a sandhi rule, in order of the split point.
pub fn two_word(chunk: &str, limit: usize) -> Vec<SplitCandidate> {
  let mut splits: Vec<SplitCandidate> = Vec::new();
  for i in 0..chunk.len() {
    for s in SPLITTER.split_at(chunk, i) {
      // One-letter parts (ऋ + आमोऽस्ति) are noise, never the intended reading.
      if s.kind() != Kind::Standard || !s.is_valid() || s.first().len() < 2 || s.second().len() < 2
      {
        continue;
      }
      let c = SplitCandidate {
        left: to_deva(&pada_final(s.first())),
        right: to_deva(s.second()),
      };
      if !splits
        .iter()
        .any(|x| x.left == c.left && x.right == c.right)
      {
        splits.push(c);
      }
    }
  }
  splits.truncate(limit);
  splits
}

const VOICELESS: &str = "kKcCwWtTpPSzs";

/// Two words can simply touch (no sandhi change) only if the first ends the way a word can
/// before that sound: a vowel before a consonant; visarga before k/p/sibilants; anusvāra before
/// a consonant; or one final consonant (no cluster) that agrees in voicing with the next sound.
fn plain_cut(f: &str, r: &str) -> bool {
  let mut tail = f.chars().rev();
  let (Some(a), Some(b)) = (tail.next(), r.chars().next()) else {
    return false;
  };
  let before = tail.next();
  let voiced_next = is_vowel(b) || !(VOICELESS.contains(b) || b == 'H');
  match a {
    a if is_vowel(a) => !is_vowel(b) && b != '\'',
    'H' => "kKpPSzs".contains(b),
    'M' => !is_vowel(b),
    _ if !before.is_some_and(is_vowel) => false,
    'k' | 'w' | 't' | 'p' => VOICELESS.contains(b),
    'g' | 'q' | 'd' | 'b' | 'r' => voiced_next,
    'N' | 'R' | 'n' | 'm' => true,
    _ => false,
  }
}

/// Ways the text can split at one place: the sound at `at` is where the words meet (a sandhi
/// rule produced it), or the words simply touch just before it. `(first word, rest)` in SLP1.
pub fn splits_at(chunk: &str, at: usize) -> Vec<(String, String)> {
  let mut out: Vec<(String, String)> = Vec::new();
  let mut push = |f: String, r: String| {
    if !out.contains(&(f.clone(), r.clone())) {
      out.push((f, r));
    }
  };
  if at >= chunk.len() || !chunk.is_char_boundary(at) {
    return out;
  }
  if at >= 1 {
    let (f, r) = chunk.split_at(at);
    if plain_cut(f, r) && word_key(f).len() >= 2 && !word_key(r).is_empty() {
      // Pada form of the first word: anusvāra is a final m; final r is visarga.
      let f = match f.strip_suffix('M') {
        Some(x) => format!("{x}m"),
        None => pada_final(f),
      };
      push(f, r.to_string());
    }
  }
  for s in SPLITTER.split_at(chunk, at) {
    if s.kind() == Kind::Standard
      && s.is_valid()
      && word_key(s.first()).len() >= 2
      && !word_key(s.second()).is_empty()
    {
      push(pada_final(s.first()), s.second().to_string());
    }
  }
  out
}

/// A chunk's last word as it was before sandhi with the next chunk changed its final sound: ओ →
/// अः (रामो वनं), ं → म् (वनं गच्छति), a voiced stop → voiceless (वाग् देवी), र् → ः (पुनर् आगतः).
pub fn final_variants(word: &str) -> Vec<String> {
  let Some(last) = word.chars().last() else {
    return vec![];
  };
  let stem = &word[..word.len() - last.len_utf8()];
  if stem.is_empty() {
    return vec![];
  }
  match last {
    'o' => vec![format!("{stem}aH")],
    'M' => vec![format!("{stem}m")],
    'r' => vec![format!("{stem}H")],
    'g' => vec![format!("{stem}k")],
    'q' => vec![format!("{stem}w")],
    'd' => vec![format!("{stem}t")],
    'b' => vec![format!("{stem}p")],
    _ => vec![],
  }
}

/// Longest chunk the optimizer segments (SLP1 letters); longer ones only get two-word splits.
const LATTICE_MAX_LEN: usize = 48;
const LATTICE_MAX_NODES: usize = 600;

/// All the ways a chunk can be cut into words: from each remaining text, the possible
/// (first word, rest) splits. Built once per chunk; [`readings`] then scores paths through it.
pub struct Lattice {
  root: String,
  edges: HashMap<String, Vec<(String, String)>>,
}

pub fn lattice(chunk: &str) -> Lattice {
  let mut edges: HashMap<String, Vec<(String, String)>> = HashMap::new();
  let mut todo = vec![chunk.to_string()];
  while let Some(rest) = todo.pop() {
    if edges.contains_key(&rest) || edges.len() >= LATTICE_MAX_NODES {
      continue;
    }
    let mut out = Vec::new();
    if rest.len() <= LATTICE_MAX_LEN {
      for at in 1..rest.len() {
        for (first, second) in splits_at(&rest, at) {
          // Progress: the rest must get shorter, so paths end.
          if second.len() < rest.len() && word_key(&second).len() >= 2 {
            todo.push(second.clone());
            out.push((first, second));
          }
        }
      }
    }
    edges.insert(rest, out);
  }
  Lattice {
    root: chunk.to_string(),
    edges,
  }
}

impl Lattice {
  /// Every word that can appear in a reading (dictionary keys), for one lookup.
  pub fn words(&self) -> HashSet<String> {
    let mut w = HashSet::new();
    for (rest, out) in &self.edges {
      w.insert(word_key(rest));
      w.extend(final_variants(rest).iter().map(|v| word_key(v)));
      for (first, _) in out {
        w.insert(word_key(first));
      }
    }
    w
  }
}

/// Dictionary lookup for a key (SLP1 letters): a headword or inflected form in the site's
/// dictionaries. The splitter adds vidyut's kosha (every form the grammar generates) itself.
pub type InDictionary<'a> = &'a dyn Fn(&str) -> bool;

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct Part {
  pub text: String,
  /// A word: in the dictionaries or generated by the grammar (vidyut's kosha).
  pub known: bool,
  /// A headword or inflected form in the dictionaries.
  pub dictionary: bool,
  /// What the form can be (vidyut's kosha), likeliest first; empty when the grammar has no such
  /// form.
  pub analyses: Vec<PadaInfo>,
}

impl Part {
  pub fn new(slp1: &str, dict: InDictionary) -> Self {
    let k = word_key(slp1);
    let analyses = kosha::analyses(&k);
    let dictionary = dict(&k);
    Part {
      text: to_deva(slp1),
      known: dictionary || !analyses.is_empty() || kosha::contains(&k),
      dictionary,
      analyses,
    }
  }
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct Reading {
  pub parts: Vec<Part>,
  /// `तथा✓ + एव✓` (✓ = known): how the reading is shown to Jev, and its option label.
  pub label: String,
  /// This is cheda's segmentation (vidyut's statistical segmenter).
  pub cheda: bool,
}

impl Reading {
  pub fn new(parts: Vec<Part>, cheda: bool) -> Self {
    let label = parts
      .iter()
      .map(|p| format!("{}{}", p.text, if p.known { "✓" } else { "" }))
      .collect::<Vec<_>>()
      .join(" + ");
    Reading {
      parts,
      label,
      cheda,
    }
  }

  fn words(&self) -> Vec<&str> {
    self.parts.iter().map(|p| p.text.as_str()).collect()
  }
}

/// Cost of one word: dictionary words are cheapest, other forms the grammar generates a little
/// dearer, very short words dearer still (one-syllable words match almost anywhere); unknown
/// stretches are expensive.
fn part_cost(slp1: &str, dict: InDictionary) -> f32 {
  let k = word_key(slp1);
  let short = if k.len() <= 2 { 1.2 } else { 0.0 };
  if dict(&k) {
    1.0 + short
  } else if kosha::contains(&k) {
    1.2 + short
  } else {
    3.0
  }
}

type Paths = Vec<(f32, Vec<String>)>;

fn best_paths(
  lat: &Lattice,
  rest: &str,
  dict: InDictionary,
  k: usize,
  memo: &mut HashMap<String, Paths>,
) -> Paths {
  if let Some(p) = memo.get(rest) {
    return p.clone();
  }
  // Every rest runs to the chunk's end, so it can be the last word as is or with its final
  // sound restored (रामो → रामः before a space).
  let mut paths: Paths = std::iter::once(rest.to_string())
    .chain(final_variants(rest))
    .map(|w| (part_cost(&w, dict), vec![w]))
    .collect();
  for (first, second) in lat.edges.get(rest).into_iter().flatten() {
    let c = part_cost(first, dict);
    for (tail_cost, tail) in best_paths(lat, second, dict, k, memo) {
      let mut words = vec![first.clone()];
      words.extend(tail);
      paths.push((c + tail_cost, words));
    }
  }
  paths.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.len().cmp(&b.1.len())));
  let mut seen = HashSet::new();
  paths.retain(|(_, w)| seen.insert(w.join(" ")));
  paths.truncate(k);
  memo.insert(rest.to_string(), paths.clone());
  paths
}

/// The `k` best ways to read a chunk as words (fewest unknown parts, then fewest parts).
pub fn readings(lat: &Lattice, dict: InDictionary, k: usize) -> Vec<Reading> {
  readings_of(lat, &lat.root, dict, k)
}

/// Same, for a rest of the chunk that is in the lattice (or a fresh one).
pub fn readings_of(lat: &Lattice, rest: &str, dict: InDictionary, k: usize) -> Vec<Reading> {
  let mut memo = HashMap::new();
  best_paths(lat, rest, dict, k, &mut memo)
    .into_iter()
    .map(|(_, words)| Reading::new(words.iter().map(|w| Part::new(w, dict)).collect(), false))
    .collect()
}

/// Adds cheda's segmentation of the chunk to the optimizer's readings: marks the same reading, or
/// puts it where its cost ranks it, at most second (kept even past `k`). Each of its words leads with the analysis
/// cheda chose.
pub fn with_cheda(
  mut readings: Vec<Reading>,
  chunk: &str,
  dict: InDictionary,
  k: usize,
) -> Vec<Reading> {
  // As written and with the final sound restored (रामो → रामः, but विष्णो stays): the
  // segmentation with fewer words wins, ties go to the text as written.
  let Some(words) = std::iter::once(chunk.to_string())
    .chain(final_variants(chunk))
    .filter_map(|c| kosha::segment(&c))
    .min_by_key(|w| w.len())
  else {
    return readings;
  };
  let parts: Vec<Part> = words
    .iter()
    .map(|w| {
      let mut p = Part::new(&pada_final(&w.text), dict);
      if let Some(chosen) = &w.pada {
        p.analyses.retain(|a| a != chosen);
        p.analyses.insert(0, chosen.clone());
      }
      p
    })
    .collect();
  let cheda = Reading::new(parts, true);
  // cheda's reading of known words beats an optimizer reading with an unknown stretch
  // (पाण्डवाश्चैव whole vs पाण्डवाः + च + एव, short words being dear).
  let all_known = cheda.parts.iter().all(|p| p.known);
  let first_has_unknown = readings
    .first()
    .is_some_and(|r| r.parts.iter().any(|p| !p.known));
  if let Some(i) = readings.iter().position(|r| r.words() == cheda.words()) {
    readings.remove(i);
    let at = if all_known && first_has_unknown { 0 } else { i };
    readings.insert(at, cheda);
    return readings;
  }
  let cost: f32 = words
    .iter()
    .map(|w| part_cost(&pada_final(&w.text), dict))
    .sum();
  let at = readings
    .iter()
    .position(|r| {
      r.parts
        .iter()
        .map(|p| part_cost(&to_slp1(&p.text).unwrap_or_default(), dict))
        .sum::<f32>()
        > cost
    })
    .unwrap_or(readings.len())
    // cheda weighs how common words and their sequences are, which the optimizer cannot: its
    // reading always shows next to the optimizer's best, or first (see above).
    .min(if all_known && first_has_unknown { 0 } else { 1 });
  readings.insert(at, cheda);
  if readings.len() > k {
    let drop = (0..readings.len()).rev().find(|&i| !readings[i].cheda);
    if let Some(i) = drop {
      readings.remove(i);
    }
  }
  readings
}

#[derive(Serialize, ToSchema, Clone)]
pub struct Varna {
  /// The sound in Devanagari (त्, अ, ः).
  pub text: String,
  /// Position to pass to the split-at API.
  pub at: usize,
}

#[derive(Serialize, ToSchema, Clone)]
pub struct Akshara {
  pub varnas: Vec<Varna>,
}

/// The chunk as syllables (akṣaras), each broken into its sounds (varṇas).
pub fn aksharas(chunk: &str) -> Vec<Akshara> {
  let mut out: Vec<Akshara> = Vec::new();
  let mut cur: Vec<Varna> = Vec::new();
  for (at, c) in chunk.char_indices() {
    let text = match c {
      '\'' => "ऽ".to_string(),
      c => to_deva(&c.to_string()),
    };
    let v = Varna { text, at };
    if is_vowel(c) {
      cur.push(v);
      out.push(Akshara {
        varnas: std::mem::take(&mut cur),
      });
    } else if matches!(c, 'H' | 'M' | '~' | '\'') && cur.is_empty() && !out.is_empty() {
      out.last_mut().expect("checked").varnas.push(v);
    } else {
      cur.push(v);
    }
  }
  if !cur.is_empty() {
    match out.last_mut() {
      Some(last) => last.varnas.extend(cur),
      None => out.push(Akshara { varnas: cur }),
    }
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  fn pairs(text: &str) -> Vec<String> {
    two_word(&chunks(text).unwrap()[0], 50)
      .iter()
      .map(|c| format!("{}+{}", c.left, c.right))
      .collect()
  }

  #[test]
  fn splits_and_rules() {
    assert!(pairs("तथैव").contains(&"तथा+एव".to_string()));
    assert!(pairs("रामोऽस्ति").contains(&"रामः+अस्ति".to_string()));
    assert!(rules().len() > 1000);
  }

  #[test]
  fn optimizer_reads_known_words() {
    let known: HashSet<&str> = ["taTA", "eva", "Darma", "kzetre", "kuru", "rAmaH", "asti"]
      .into_iter()
      .collect();
    let is_known = |k: &str| known.contains(k);
    let read = |t: &str| {
      let c = chunks(t).unwrap().remove(0);
      readings(&lattice(&c), &is_known, 3)[0]
        .parts
        .iter()
        .map(|p| p.text.clone())
        .collect::<Vec<_>>()
        .join(" + ")
    };
    assert_eq!(read("तथैव"), "तथा + एव");
    // The kosha knows the compound as one word.
    assert!(["धर्मक्षेत्रे", "धर्म + क्षेत्रे"].contains(&read("धर्मक्षेत्रे").as_str()));
    assert_eq!(read("रामोऽस्ति"), "रामः + अस्ति");
    assert_eq!(read("रामो"), "रामः");
    let syl = aksharas(&chunks("रामः").unwrap()[0]);
    let s: Vec<String> = syl
      .iter()
      .map(|s| s.varnas.iter().map(|v| v.text.clone()).collect::<String>())
      .collect();
    assert_eq!(s, vec!["र्आ", "म्अः"]);
  }

  #[test]
  fn kosha_reads_inflected_forms() {
    let is_known = |_: &str| false;
    let c = chunks("दत्तोद्भवनम्").unwrap().remove(0);
    let r = with_cheda(readings(&lattice(&c), &is_known, 5), &c, &is_known, 5);
    let labels: Vec<&str> = r.iter().map(|r| r.label.as_str()).collect();
    assert!(r.iter().any(|r| r.cheda), "{labels:?}");
    assert_eq!(r[0].parts.last().unwrap().text, "उद्भवनम्", "{labels:?}");
    assert!(
      r[0]
        .parts
        .last()
        .unwrap()
        .analyses
        .iter()
        .any(|a| a.lemma == "उद्भवन")
    );
  }
}
