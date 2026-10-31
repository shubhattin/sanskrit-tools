//! Source text → sanitized HTML fragments (the `senses[]` stored in the DB).
//!
//! Plain-text sources are HTML-escaped first and only then decorated with the
//! small tag set documented in `docs/api.md`, so the output is safe by
//! construction. HTML sources go through `ammonia` with an allowlist.

use regex::Regex;
use std::sync::LazyLock;

pub fn escape(s: &str) -> String {
  let mut out = String::with_capacity(s.len() + 8);
  for c in s.chars() {
    match c {
      '&' => out.push_str("&amp;"),
      '<' => out.push_str("&lt;"),
      '>' => out.push_str("&gt;"),
      '"' => out.push_str("&quot;"),
      '\'' => out.push_str("&#39;"),
      c => out.push(c),
    }
  }
  out
}

/// TSV bodies encode newlines as a literal backslash-n.
pub fn unescape_tsv(s: &str) -> String {
  s.replace("\\n", "\n").replace('\t', " ")
}

static PAGE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[Page\s*([^\]]{1,24})\]").unwrap());
static BOLD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{@(.*?)@\}").unwrap());
static ITAL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{%(.*?)%\}").unwrap());
static HASH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{#(.*?)#\}").unwrap());
static MDBOLD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*\*(.+?)\*\*").unwrap());
/// Commentaries misuse the Vedic accent signs ॒ … ॑ (U+0952 / U+0951) as quotes.
static ACCENT_QUOTE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new("\u{0952}([^\u{0951}\u{0952}\n]{0,400}?)\u{0951}").unwrap());

#[derive(Clone, Copy, Default)]
pub struct Opts {
  /// Cologne markup: `{@b@}`, `{%i%}`, `{#..#}`, `[Page…]`
  pub cologne: bool,
  /// MW "Anglicized Sanskrit" digit encoding in Latin tokens (Sus3r. → Suśr.)
  pub mw_as: bool,
  /// ॒…॑ used as quotation marks
  pub accent_quotes: bool,
  /// `**bold**`
  pub md_bold: bool,
}

/// Escaped plain text → HTML fragment.
pub fn text_to_html(raw: &str, o: Opts) -> String {
  let mut s = raw.trim().to_string();
  if o.accent_quotes {
    s = ACCENT_QUOTE.replace_all(&s, "‘$1’").into_owned();
    // Unpaired leftovers are quotes too, never accents, in these works.
    s = s.replace('\u{0952}', "‘").replace('\u{0951}', "’");
  }
  if o.mw_as {
    s = mw_as_to_iast(&s);
  }
  let mut h = escape(&s);
  if o.cologne {
    h = BOLD.replace_all(&h, "<b>$1</b>").into_owned();
    h = ITAL.replace_all(&h, "<i>$1</i>").into_owned();
    h = HASH.replace_all(&h, "$1").into_owned();
    h = PAGE
      .replace_all(&h, r#"<span class="page">[p. $1]</span>"#)
      .into_owned();
  }
  if o.md_bold {
    h = MDBOLD.replace_all(&h, "<b>$1</b>").into_owned();
  }
  h = h.replace(
    '\u{FFFD}',
    r#"<span class="lost" title="glyph lost in source">◌</span>"#,
  );
  let h = h.trim_matches('\n');
  collapse_breaks(&h.replace('\n', "<br>"))
}

fn collapse_breaks(h: &str) -> String {
  static MANY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:\s*<br>\s*){3,}").unwrap());
  MANY.replace_all(h, "<br><br>").into_owned()
}

static AS_TOKEN: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"[A-Za-z]*[A-Za-z][1-9][A-Za-z1-9]*").unwrap());

/// Cologne MW "AS" encoding, verified against the mw-sa data:
/// `a1`→ā, `s3`→ś, `r2`→ṛ, `n2`→ṇ, `n3`→ṅ, `n5`→ñ, `n6`/`m2`→ṃ, `t2`→ṭ,
/// `d2`→ḍ, `h2`→ḥ; `4`/`7` mark sandhi-lengthened vowels (Siddha7nta).
/// Tokens like `Page123` and unknown pairs are left untouched.
pub fn mw_as_to_iast(s: &str) -> String {
  AS_TOKEN
    .replace_all(s, |c: &regex::Captures| {
      let tok = &c[0];
      if tok.starts_with("Page") {
        return tok.to_string();
      }
      let mut out = String::with_capacity(tok.len());
      let chars: Vec<char> = tok.chars().collect();
      let mut i = 0;
      while i < chars.len() {
        if i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
          let rep = match (chars[i], chars[i + 1]) {
            ('a', '1' | '4' | '7') => Some("ā"),
            ('A', '1' | '4' | '7') => Some("Ā"),
            ('i', '1' | '4' | '7') => Some("ī"),
            ('I', '1' | '4' | '7') => Some("Ī"),
            ('u', '1' | '4' | '7') => Some("ū"),
            ('U', '1' | '4' | '7') => Some("Ū"),
            ('e', '7') => Some("e"),
            ('o', '7') => Some("o"),
            ('r', '2') => Some("ṛ"),
            ('R', '2') => Some("Ṛ"),
            ('s', '3') => Some("ś"),
            ('S', '3') => Some("Ś"),
            ('n', '2') => Some("ṇ"),
            ('n', '3') => Some("ṅ"),
            ('n', '5') => Some("ñ"),
            ('n', '6') | ('m', '2') => Some("ṃ"),
            ('t', '2') => Some("ṭ"),
            ('T', '2') => Some("Ṭ"),
            ('d', '2') => Some("ḍ"),
            ('h', '2') => Some("ḥ"),
            _ => None,
          };
          if let Some(r) = rep {
            out.push_str(r);
            i += 2;
            continue;
          }
        }
        out.push(chars[i]);
        i += 1;
      }
      out
    })
    .into_owned()
}

/// Sanitize source HTML (BHSD, rAnADe). `bword://x` links become `a[data-q]`
/// handled by the frontend; other links must be http(s).
pub fn sanitize_html(raw: &str) -> String {
  static STYLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?is)<style.*?</style>").unwrap());
  static BWORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<a\s+href="bword://([^"]*)"\s*>"#).unwrap());
  let s = STYLE.replace_all(raw, "");
  let s = BWORD.replace_all(&s, r#"<a data-q="$1">"#);
  let mut b = ammonia::Builder::default();
  b.tags(
    [
      "b", "i", "br", "small", "sup", "sub", "a", "span", "em", "strong", "p",
    ]
    .into(),
  )
  .tag_attributes([("a", ["href", "data-q"].into()), ("span", ["class"].into())].into())
  .url_schemes(["http", "https"].into())
  .link_rel(Some("noopener noreferrer nofollow"));
  let out = b.clean(&s).to_string();
  collapse_breaks(out.trim())
}

/// Like [`sanitize_html`], but `bword://key` links are resolved to a
/// Devanagari query by `resolve` (unresolved links lose their target).
pub fn sanitize_html_resolving(raw: &str, resolve: &dyn Fn(&str) -> Option<String>) -> String {
  static BWORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<a\s+href="bword://([^"]*)"\s*>"#).unwrap());
  let s = BWORD.replace_all(raw, |c: &regex::Captures| match resolve(&c[1]) {
    Some(q) => format!(r#"<a href="bword://{}">"#, escape(&q)),
    None => "<a>".to_string(),
  });
  sanitize_html(&s)
}

static COLOGNE_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(
    r#"(?s)\{#(?P<sa1>.*?)#\}|<s>(?P<sa2>.*?)</s>|\{@(?P<b>.*?)@\}|\{%(?P<i>.*?)%\}|<(?P<tag>ab|lex|ls|i)>(?P<inner>.*?)</(?:ab|lex|ls|i)>|<div n="(?P<div>[^"]*)">|\[Page(?P<page>[^\]]*)\]|(?P<brk>[ⒶⒷⒺⓃ])|(?P<sp>[ⒸⒹ¦])|<[^>]*>"#,
  )
  .unwrap()
});

/// Cologne CDSL markup (English→Sanskrit dictionaries) → (HTML, English text).
/// Sanskrit (`{#..#}` / `<s>..</s>`, SLP1) becomes Devanagari `span.sa` and is
/// excluded from the English text used for full-text search.
pub fn cologne_to_html(raw: &str) -> (String, String) {
  // Editorial corrections `{{old->new||date|by|url|note}}`: keep the corrected text.
  static CORR: LazyLock<Regex> =
    // `old` / `new` may themselves contain braces (`{?}`, `{#..#}`): match lazily
    // up to the metadata separator `||` (or the closing `}}` when absent).
    LazyLock::new(|| Regex::new(r"\{\{.*?->(.*?)(?:\|\|.*?)?\}\}").unwrap());
  let raw = CORR.replace_all(raw, "$1");
  let raw = raw.as_ref();
  let mut html = String::new();
  let mut en = String::new();
  cologne_walk(raw, &mut html, &mut en);
  let html = collapse_breaks(html.trim().trim_start_matches("<br>").trim());
  static WS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());
  (html, WS.replace_all(en.trim(), " ").into_owned())
}

fn cologne_walk(raw: &str, html: &mut String, en: &mut String) {
  let mut last = 0;
  for c in COLOGNE_TOKEN.captures_iter(raw) {
    let m = c.get(0).unwrap();
    let text = &raw[last..m.start()];
    html.push_str(&escape(text));
    en.push_str(text);
    last = m.end();
    if let Some(sa) = c.name("sa1").or(c.name("sa2")) {
      let deva = crate::translit::slp1_to_deva(sa.as_str());
      html.push_str(&format!(r#"<span class="sa">{}</span>"#, escape(&deva)));
      en.push(' ');
    } else if let Some(b) = c.name("b") {
      html.push_str("<b>");
      cologne_walk(b.as_str(), html, en);
      html.push_str("</b>");
    } else if let Some(i) = c.name("i") {
      html.push_str("<i>");
      cologne_walk(i.as_str(), html, en);
      html.push_str("</i>");
    } else if let Some(inner) = c.name("inner") {
      html.push_str("<i>");
      cologne_walk(inner.as_str(), html, en);
      html.push_str("</i>");
    } else if let Some(d) = c.name("div") {
      let d = d.as_str();
      if d
        .chars()
        .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
      {
        html.push_str("<br>");
      } else {
        html.push(' ');
      }
      en.push(' ');
    } else if let Some(p) = c.name("page") {
      html.push_str(&format!(
        r#"<span class="page">[p. {}]</span>"#,
        escape(p.as_str().trim())
      ));
    } else if c.name("brk").is_some() {
      html.push_str("<br>");
      en.push(' ');
    } else {
      html.push(' ');
      en.push(' ');
    }
  }
  let text = &raw[last..];
  html.push_str(&escape(text));
  en.push_str(text);
}

/// Visible text of an HTML fragment.
pub fn strip_tags(h: &str) -> String {
  static TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]*>").unwrap());
  let t = TAG.replace_all(h, " ");
  t.replace("&nbsp;", " ")
    .replace("&amp;", "&")
    .replace("&lt;", "<")
    .replace("&gt;", ">")
    .replace("&quot;", "\"")
    .replace("&#39;", "'")
}

/// English-only text for full-text search: tags, Devanagari and page refs removed.
pub fn english_gloss(html_senses: &[String]) -> String {
  static DEVA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\u{0900}-\u{097F}]+").unwrap());
  static WS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());
  static EMPTY_BRACKETS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\(\s*[,.;:/\-]*\s*\)|\[\s*[,.;:/\-]*\s*\]").unwrap());
  static ORPHAN_PUNCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|\s)(?:[/\-–—˚°]+\s*)+(?:\s|$)").unwrap());
  static SPACE_BEFORE_PUNCT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+([,.;:)])").unwrap());
  let mut out = String::new();
  for s in html_senses {
    let t = strip_tags(s);
    let t = DEVA.replace_all(&t, " ");
    // Punctuation orphaned by removing Sanskrit: `/ -`, `( )`, `[ ]`, `, ,`.
    let t = EMPTY_BRACKETS.replace_all(&t, " ");
    let t = ORPHAN_PUNCT.replace_all(&t, " ");
    let t = WS.replace_all(&t, " ");
    let t = SPACE_BEFORE_PUNCT.replace_all(&t, "$1");
    out.push_str(t.trim());
    out.push('\n');
  }
  out.trim().to_string()
}

/// `<a data-q="देवनागरी">label</a>` cross reference. The query is the
/// Devanagari label itself: unambiguous for the search endpoint and readable
/// when it lands in the search box.
pub fn xref(label_deva: &str, _slp1: &str) -> String {
  format!(r#"<a data-q="{0}">{0}</a>"#, escape(label_deva))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mw_as() {
    assert_eq!(
      mw_as_to_iast("Sus3r. Su1ryas. Pa1n2. Page123"),
      "Suśr. Sūryas. Pāṇ. Page123"
    );
    assert_eq!(mw_as_to_iast("S3a1n3khGr2."), "ŚāṅkhGṛ.");
    assert_eq!(mw_as_to_iast("( अग्Un2. )"), "( अग्Uṇ. )");
  }

  #[test]
  fn escapes_then_marks() {
    let h = text_to_html(
      "{@अ@}¦ <x> [Page1-001-b+ 41]\nline2",
      Opts {
        cologne: true,
        ..Default::default()
      },
    );
    assert_eq!(
      h,
      r#"<b>अ</b>¦ &lt;x&gt; <span class="page">[p. 1-001-b+ 41]</span><br>line2"#
    );
  }

  #[test]
  fn quotes() {
    let h = text_to_html(
      "इति ॒सुपां सुलु॑ गिति",
      Opts {
        accent_quotes: true,
        ..Default::default()
      },
    );
    assert_eq!(h, "इति ‘सुपां सुलु’ गिति");
  }

  #[test]
  fn gloss() {
    let g = english_gloss(&[
      "अनग्नि/ अन्-अग्नि m. ( अन्-अग्निNir. )non-fire".into(),
      "अग्नि m. fire , sacrificial (गार्हपत्य, आहवनीय)".into(),
    ]);
    assert_eq!(g, "m. ( Nir.)non-fire\nm. fire, sacrificial");
  }

  #[test]
  fn cologne() {
    let (h, en) =
      cologne_to_html("{@Abandon@}¦,Ⓒ<lex>v. t.</lex>Ⓓ<s>tyaj</s> 1 <ab>P</ab>.Ⓝ{@[Abandon]ed@}");
    assert_eq!(
      h,
      r#"<b>Abandon</b> , <i>v. t.</i> <span class="sa">त्यज्</span> 1 <i>P</i>.<br><b>[Abandon]ed</b>"#
    );
    assert_eq!(en, "Abandon , v. t. 1 P. [Abandon]ed");
    let (h, _) = cologne_to_html(r#"BECOMINGNESS¦, {%s.%} {#yuktatA, OcityaM#}"#);
    assert_eq!(
      h,
      r#"BECOMINGNESS , <i>s.</i> <span class="sa">युक्तता, औचित्यं</span>"#
    );
  }

  #[test]
  fn sanitize() {
    let h = sanitize_html(
      r#"<style>x</style><b>a</b><script>bad()</script><a href="bword://agni">x</a><font color="red">r</font>"#,
    );
    assert_eq!(
      h,
      r#"<b>a</b><a data-q="agni" rel="noopener noreferrer nofollow">x</a>r"#
    );
  }
}
