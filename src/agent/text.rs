//! Entry HTML → plain text for the model (tags dropped, entities decoded, length capped).

/// Plain text of a sanitized entry fragment (the tag set in `docs/api.md`): `<br>` becomes a line
/// break, every other tag is dropped; the five entities `ammonia` emits are decoded.
pub fn html_to_text(html: &str) -> String {
  let mut out = String::with_capacity(html.len());
  let mut rest = html;
  while let Some(lt) = rest.find('<') {
    out.push_str(&rest[..lt]);
    let after = &rest[lt..];
    let Some(gt) = after.find('>') else {
      out.push_str(after);
      rest = "";
      break;
    };
    let name: String = after[1..gt]
      .trim_start_matches('/')
      .chars()
      .take_while(|c| c.is_ascii_alphanumeric())
      .collect();
    if name.eq_ignore_ascii_case("br") {
      out.push('\n');
    }
    rest = &after[gt + 1..];
  }
  out.push_str(rest);
  decode_entities(&out)
}

fn decode_entities(s: &str) -> String {
  if !s.contains('&') {
    return s.to_string();
  }
  s.replace("&lt;", "<")
    .replace("&gt;", ">")
    .replace("&quot;", "\"")
    .replace("&#39;", "'")
    .replace("&nbsp;", " ")
    .replace("&amp;", "&")
}

/// Senses joined one per line.
pub fn senses_text(senses: &[String]) -> String {
  senses
    .iter()
    .map(|s| html_to_text(s).trim().to_string())
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join("\n")
}

/// At most `max` characters, cut at a char boundary with `…`; returns whether it was cut.
pub fn truncate(text: String, max: usize) -> (String, bool) {
  match text.char_indices().nth(max) {
    None => (text, false),
    Some((end, _)) => {
      let mut cut = text[..end].trim_end().to_string();
      cut.push('…');
      (cut, true)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn strips_tags_and_keeps_line_breaks() {
    assert_eq!(
      html_to_text("<b>धर्म</b> m. law<br>duty &amp; <span class=\"ref\">RV.</span>"),
      "धर्म m. law\nduty & RV."
    );
    assert_eq!(html_to_text("a<br/>b"), "a\nb");
    assert_eq!(html_to_text("x &lt;y&gt;"), "x <y>");
  }

  #[test]
  fn truncates_on_char_boundaries() {
    assert_eq!(truncate("धर्मः".into(), 3), ("धर्…".into(), true));
    assert_eq!(truncate("abc".into(), 3), ("abc".into(), false));
  }
}
