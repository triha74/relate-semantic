/// A slug is a lowercase letter, then lowercase letters, digits, and hyphens.
pub fn is_slug(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// The slug an agent most likely meant, or `None` when nothing sensible is left.
/// `Widget Price` is `widget-price`. `NW 2024` is `nw-2024`. `2024` is `None`.
pub fn suggest(text: &str) -> Option<String> {
    let mut out = String::new();
    let mut gap = false;
    for c in text.chars() {
        let c = c.to_ascii_lowercase();
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            if gap && !out.is_empty() {
                out.push('-');
            }
            gap = false;
            out.push(c);
        } else {
            gap = true;
        }
    }
    if is_slug(&out) {
        Some(out)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs() {
        assert!(is_slug("widget-price"));
        assert!(is_slug("nw-2024"));
        assert!(is_slug("a"));
        assert!(!is_slug(""));
        assert!(!is_slug("Widget"));
        assert!(!is_slug("2024"));
        assert!(!is_slug("-a"));
        assert!(!is_slug("a_b"));
        assert!(!is_slug("a b"));
    }

    #[test]
    fn suggestions() {
        assert_eq!(suggest("Widget Price").as_deref(), Some("widget-price"));
        assert_eq!(suggest("NW 2024").as_deref(), Some("nw-2024"));
        assert_eq!(suggest("  nw__2024--x ").as_deref(), Some("nw-2024-x"));
        assert_eq!(suggest("2024"), None);
        assert_eq!(suggest("!!"), None);
        assert_eq!(suggest("Ünïcode"), Some("n-code".into()));
    }
}
