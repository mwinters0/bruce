//! The selectors a file writes (D33): a list in one string, each one checked
//! against what `lol_html` parses.

/// Each selector of `list` on its own, a string holding a selector list
/// split at its commas, and each checked against what `lol_html` parses:
/// a combinator or pseudo-class it does not stream (`+`, `~`, `:has`) is
/// refused with its reason (D33).
pub(super) fn selectors(list: &[String]) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for written in list {
        for part in split_list(written) {
            let part = part.trim();
            if part.is_empty() {
                return Err(format!("{written:?} holds an empty selector"));
            }
            part.parse::<lol_html::Selector>()
                .map_err(|e| format!("{part:?}: {e}"))?;
            out.push(part.to_owned());
        }
    }
    Ok(out)
}

/// `s` split at each comma outside quotes, brackets and parentheses, a
/// backslash escaping the character after it.
fn split_list(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let (mut depth, mut quote, mut escaped, mut start) = (0u32, None, false, 0);
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match (c, quote) {
            ('\\', _) => escaped = true,
            (q, Some(open)) if q == open => quote = None,
            (_, Some(_)) => {}
            ('"' | '\'', None) => quote = Some(c),
            ('[' | '(', None) => depth += 1,
            (']' | ')', None) => depth = depth.saturating_sub(1),
            (',', None) if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);
    parts
}
