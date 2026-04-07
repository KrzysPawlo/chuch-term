use regex::RegexBuilder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchMatch {
    pub row: usize,
    pub start: usize,
    pub end: usize,
}

/// Find all literal occurrences of `query` in `lines`.
/// Returns byte ranges in the original strings, safe to reuse for cursor/highlight logic.
pub fn find_all(lines: &[String], query: &str, case_sensitive: bool) -> Vec<SearchMatch> {
    if query.is_empty() {
        return Vec::new();
    }

    let pattern = regex::escape(query);
    let regex = match RegexBuilder::new(&pattern)
        .case_insensitive(!case_sensitive)
        .unicode(true)
        .build()
    {
        Ok(regex) => regex,
        Err(_) => return Vec::new(),
    };

    let mut results = Vec::new();
    for (row, line) in lines.iter().enumerate() {
        for capture in regex.find_iter(line) {
            results.push(SearchMatch {
                row,
                start: capture.start(),
                end: capture.end(),
            });
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn finds_case_insensitive_literal_matches() {
        let results = find_all(&lines(&["Alpha alpha ALPHA"]), "alpha", false);
        assert_eq!(
            results,
            vec![
                SearchMatch {
                    row: 0,
                    start: 0,
                    end: 5,
                },
                SearchMatch {
                    row: 0,
                    start: 6,
                    end: 11,
                },
                SearchMatch {
                    row: 0,
                    start: 12,
                    end: 17,
                },
            ]
        );
    }

    #[test]
    fn escapes_regex_metacharacters() {
        let results = find_all(&lines(&["a+b a?b a+b"]), "a+b", true);
        assert_eq!(
            results,
            vec![
                SearchMatch {
                    row: 0,
                    start: 0,
                    end: 3,
                },
                SearchMatch {
                    row: 0,
                    start: 8,
                    end: 11,
                },
            ]
        );
    }

    #[test]
    fn preserves_unicode_offsets() {
        // "ZAŻÓŁĆ" is the correct uppercase of "zażółć" (z→Z, a→A, ż→Ż, ó→Ó, ł→Ł, ć→Ć).
        // "ŻAŻÓŁĆ" starts with Ż (uppercase ż ≠ z), so it would NOT match "zażółć".
        let results = find_all(&lines(&["zażółć ZAŻÓŁĆ zażółć"]), "zażółć", false);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].start, 0);
        assert_eq!(&"zażółć ZAŻÓŁĆ zażółć"[results[1].start..results[1].end], "ZAŻÓŁĆ");
    }

    #[test]
    fn finds_multiple_matches_in_one_line() {
        let results = find_all(&lines(&["abc abc abc"]), "abc", true);
        assert_eq!(
            results.iter().map(|item| item.start).collect::<Vec<_>>(),
            vec![0, 4, 8]
        );
    }
}
