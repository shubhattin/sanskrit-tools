//! Sandhi between words (vidyut-sandhi): the rule table and phonetic split candidates.
//!
//! Joining words is done by the grammar instead ([`super::prakriya::join_words`]), which knows
//! more than the phonetic table (e.g. वाक् + ईशः → वागीशः).
//!
//! Splitting is purely phonetic: every way a chunk could have come from two words. It
//! over-generates on purpose; the API marks the parts that are words in the dictionaries.

use super::{Result, err, to_deva, to_slp1};
use serde::Serialize;
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

#[derive(Serialize, ToSchema)]
pub struct ChunkSplits {
  pub chunk: String,
  pub splits: Vec<SplitCandidate>,
}

/// A word-final `s` / `r` (how the table writes visarga) reads as visarga.
fn pada_final(slp1: &str) -> String {
  match slp1.strip_suffix(['s', 'r']) {
    Some(x) if !x.is_empty() => format!("{x}H"),
    _ => slp1.to_string(),
  }
}

/// Two-word split candidates for each space-separated chunk of the text (at most `limit` per
/// chunk), in order of the split point.
pub fn split(text: &str, limit: usize) -> Result<Vec<ChunkSplits>> {
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
  if chunks.len() > 8 {
    return err("Enter at most eight words at a time");
  }
  Ok(
    chunks
      .iter()
      .map(|chunk| {
        let mut splits: Vec<SplitCandidate> = Vec::new();
        for i in 0..chunk.len() {
          for s in SPLITTER.split_at(chunk, i) {
            // One-letter parts (ऋ + आमोऽस्ति) are noise, never the intended reading.
            if s.kind() != Kind::Standard
              || !s.is_valid()
              || s.first().len() < 2
              || s.second().len() < 2
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
        ChunkSplits {
          chunk: to_deva(chunk),
          splits,
        }
      })
      .collect(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn splits_and_rules() {
    let s = split("तथैव", 50).unwrap();
    let pairs: Vec<_> = s[0]
      .splits
      .iter()
      .map(|c| format!("{}+{}", c.left, c.right))
      .collect();
    assert!(pairs.contains(&"तथा+एव".to_string()), "{pairs:?}");
    let s = split("रामोऽस्ति", 50).unwrap();
    let pairs: Vec<_> = s[0]
      .splits
      .iter()
      .map(|c| format!("{}+{}", c.left, c.right))
      .collect();
    assert!(pairs.contains(&"रामः+अस्ति".to_string()), "{pairs:?}");
    assert!(rules().len() > 1000);
  }
}
