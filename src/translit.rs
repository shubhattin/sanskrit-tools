//! Transliteration + search-key normalization.
//!
//! - **SLP1** (one ASCII char per phoneme, case-sensitive) is the canonical
//!   search key. Conversions go through `vidyut-lipi`.
//! - **Loose key**: lowercase ASCII fold of SLP1 that forgives vowel length,
//!   aspiration spelled as `h`, retroflex/dental, sibilants, nasals, ṛ/ḷ,
//!   visarga and doubled consonants. Used for trigram "did you mean".
//! - Query interpretation: Devanagari → SLP1; IAST (diacritics) → SLP1; plain
//!   ASCII is read as lipilekhika "Normal", Harvard-Kyoto, ITRANS and SLP1.

use std::cell::RefCell;
use unicode_normalization::UnicodeNormalization;
use vidyut_lipi::{Lipika, Scheme};

thread_local! {
  static LIPIKA: RefCell<Lipika> = RefCell::new(Lipika::new());
}

pub fn convert(text: &str, from: Scheme, to: Scheme) -> String {
  LIPIKA.with(|l| l.borrow_mut().transliterate(text, from, to))
}

pub fn nfc(s: &str) -> String {
  s.nfc().collect()
}

pub fn has_devanagari(s: &str) -> bool {
  s.chars().any(|c| ('\u{0900}'..='\u{097F}').contains(&c))
}

const IAST_MARKS: &str = "āīūṛṝḷḹṃḥṅñṭḍṇśṣĀĪŪṚṜḶṂḤṄÑṬḌṆŚṢṁ";

pub fn has_iast(s: &str) -> bool {
  s.chars().any(|c| IAST_MARKS.contains(c))
}

/// Keep only SLP1 phoneme letters (drops spaces, punctuation, accents,
/// avagraha, digits, candrabindu).
pub fn clean_slp1(s: &str) -> String {
  s.chars().filter(|c| c.is_ascii_alphabetic()).collect()
}

/// Devanagari (or mixed) text → cleaned SLP1 key.
pub fn deva_key(deva: &str) -> String {
  clean_slp1(&convert(&nfc(deva), Scheme::Devanagari, Scheme::Slp1))
}

pub fn slp1_to_deva(slp1: &str) -> String {
  convert(slp1, Scheme::Slp1, Scheme::Devanagari)
}

pub fn iast_key(iast: &str) -> String {
  clean_slp1(&convert(
    &nfc(&iast.to_lowercase()),
    Scheme::Iast,
    Scheme::Slp1,
  ))
}

pub fn hk_key(hk: &str) -> String {
  clean_slp1(&convert(hk, Scheme::HarvardKyoto, Scheme::Slp1))
}

/// Loose fold of an SLP1 key (see module docs). Output is lowercase ASCII.
pub fn loose_key(slp1: &str) -> String {
  let mut out = String::with_capacity(slp1.len() + 4);
  for c in slp1.chars() {
    let rep: &str = match c {
      'A' => "a",
      'I' => "i",
      'U' => "u",
      'E' => "e",
      'O' => "o",
      // ṛ is commonly typed `ri` (krishna, rishi)
      'f' | 'F' => "ri",
      'x' | 'X' => "l",
      'K' => "kh",
      'G' => "gh",
      'C' => "ch",
      'J' => "jh",
      'w' => "t",
      'W' => "th",
      'q' => "d",
      'Q' => "dh",
      'T' => "th",
      'D' => "dh",
      'P' => "ph",
      'B' => "bh",
      'N' | 'Y' | 'R' | 'M' | 'm' => "n",
      'S' | 'z' => "s",
      'H' | '~' | '\'' => "",
      'L' => "l",
      'V' | 'Z' => "",
      c if c.is_ascii_lowercase() => {
        out.push(c);
        continue;
      }
      _ => "",
    };
    out.push_str(rep);
  }
  // Collapse doubled letters (tattva/tatva, archaic सर्व्व).
  let mut collapsed = String::with_capacity(out.len());
  let mut prev = '\0';
  for c in out.chars() {
    if c != prev {
      collapsed.push(c);
    }
    prev = c;
  }
  collapsed
}

fn push_unique(v: &mut Vec<String>, s: String) {
  if !s.is_empty() && !v.contains(&s) {
    v.push(s);
  }
}

/// lipilekhika "Normal" (the user's typing scheme) → SLP1.
pub fn normal_key(ascii: &str) -> String {
  let deva = lipilekhika::transliterate(
    ascii,
    lipilekhika::Script::Normal,
    lipilekhika::Script::Devanagari,
    None,
  );
  deva_key(&deva)
}

/// All SLP1 interpretations of a user query, most likely first.
pub fn query_keys(q: &str) -> Vec<String> {
  let q = nfc(q.trim());
  let mut keys = Vec::new();
  if q.is_empty() {
    return keys;
  }
  if has_devanagari(&q) {
    push_unique(&mut keys, deva_key(&q));
    return keys;
  }
  if has_iast(&q) {
    push_unique(&mut keys, iast_key(&q));
    return keys;
  }
  let lower_only = !q.chars().any(|c| c.is_ascii_uppercase());
  push_unique(&mut keys, normal_key(&q));
  push_unique(&mut keys, hk_key(&q));
  push_unique(
    &mut keys,
    clean_slp1(&convert(&q, Scheme::Itrans, Scheme::Slp1)),
  );
  // Raw SLP1 only when it uses SLP1-specific capitals (aBi, kfzRa).
  if !lower_only {
    push_unique(&mut keys, clean_slp1(&q));
  } else {
    // All-lowercase input is also valid SLP1 (no long vowels / aspirates).
    push_unique(&mut keys, clean_slp1(&q));
  }
  keys
}

/// Headword-ending variants used as a fallback (Apte lists nominals declined:
/// ज्ञानम् vs MW ज्ञान; visarga/stem variants).
pub fn ending_variants(key: &str) -> Vec<String> {
  let mut v = Vec::new();
  if let Some(stem) = key.strip_suffix('m').or_else(|| key.strip_suffix('H')) {
    push_unique(&mut v, stem.to_string());
  }
  if key.ends_with('a') || key.ends_with('A') || key.ends_with('i') || key.ends_with('u') {
    push_unique(&mut v, format!("{key}m"));
    push_unique(&mut v, format!("{key}H"));
  }
  v
}

/// Lookup keys for a Dhātupāṭha root: the clean root (`mfd`), the upadeśa
/// (`mrada` from म्रदँ) and the root with an added `a` when it ends in a
/// consonant (works cite म्रद for म्रद्).
pub fn root_variants(root: &str, upadesha: &str) -> Vec<String> {
  let mut v = Vec::new();
  push_unique(&mut v, root.to_string());
  push_unique(&mut v, upadesha.to_string());
  if root
    .chars()
    .last()
    .is_some_and(|c| !"aAiIuUfFxXeEoO".contains(c))
  {
    push_unique(&mut v, format!("{root}a"));
  }
  v
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn devanagari_keys() {
    assert_eq!(deva_key("अग्नि"), "agni");
    assert_eq!(deva_key("कृष्णः"), "kfzRaH");
    assert_eq!(deva_key("पलित-दर्शन"), "palitadarSana");
    assert_eq!(deva_key("ज्ञानम्"), "jYAnam");
  }

  #[test]
  fn loose() {
    assert_eq!(loose_key("kfzRaH"), "krisna");
    assert_eq!(loose_key("kfzRa"), loose_key(&query_keys("krishna")[0]));
    assert_eq!(loose_key("jYAnam"), "jnanan");
    assert_eq!(loose_key("tattva"), loose_key("tatva"));
    assert_eq!(loose_key("Siva"), loose_key("SivaH"));
    assert_eq!(loose_key("sarvva"), loose_key("sarva"));
  }

  #[test]
  fn queries() {
    assert_eq!(query_keys("अग्नि"), vec!["agni"]);
    assert_eq!(query_keys("kṛṣṇa"), vec!["kfzRa"]);
    assert!(
      query_keys("kRSNa").contains(&"kfzRa".to_string()),
      "{:?}",
      query_keys("kRSNa")
    );
    assert!(
      query_keys("kRShNa").contains(&"kfzRa".to_string()),
      "{:?}",
      query_keys("kRShNa")
    );
    assert!(
      query_keys("shiva").contains(&"Siva".to_string()),
      "{:?}",
      query_keys("shiva")
    );
    assert!(
      query_keys("dharma").contains(&"Darma".to_string()),
      "{:?}",
      query_keys("dharma")
    );
    assert!(query_keys("bhavati").contains(&"Bavati".to_string()));
    // jñ typed as "jn" is resolved by the loose key, not by exact SLP1.
    assert_eq!(loose_key(&query_keys("jnAna")[0]), loose_key("jYAna"));
  }

  #[test]
  fn endings() {
    assert!(ending_variants("jYAnam").contains(&"jYAna".to_string()));
    assert!(ending_variants("jYAna").contains(&"jYAnam".to_string()));
  }
}
