/// The most characters a claim or a source may hold. Counted in Unicode scalar
/// values, so `é` is one character.
pub const MAX_TEXT: usize = 600;

/// Claims a `use` returns when `--limit` is not given.
pub const DEFAULT_LIMIT: usize = 5;

/// The largest `--limit` a recall accepts.
pub const MAX_LIMIT: usize = 20;

/// `log` returns at most this many generations.
pub const LOG_CAP: usize = 50;

pub fn length(text: &str) -> usize {
    text.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_counts_characters_not_bytes() {
        assert_eq!(length("abc"), 3);
        assert_eq!(length("é€"), 2);
        assert_eq!("é€".len(), 5);
    }
}
