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

    let query_chars = (!case_sensitive).then(|| query.chars().collect::<Vec<_>>());
    let mut results = Vec::new();
    for (row, line) in lines.iter().enumerate() {
        if case_sensitive {
            for (start, matched) in line.match_indices(query) {
                results.push(SearchMatch {
                    row,
                    start,
                    end: start + matched.len(),
                });
            }
            continue;
        }

        let query_chars = query_chars.as_deref().unwrap_or(&[]);

        for (start, _) in line.char_indices() {
            if let Some(end) = match_literal_case_insensitive(line, start, query_chars) {
                results.push(SearchMatch { row, start, end });
            }
        }
    }
    results
}

fn match_literal_case_insensitive(line: &str, start: usize, query_chars: &[char]) -> Option<usize> {
    let mut end = start;
    let mut haystack = line[start..].chars();
    for expected in query_chars {
        let actual = haystack.next()?;
        if !chars_equal_case_insensitive(actual, *expected) {
            return None;
        }
        end += actual.len_utf8();
    }
    Some(end)
}

fn chars_equal_case_insensitive(left: char, right: char) -> bool {
    if left == right {
        return true;
    }
    left.to_lowercase().to_string() == right.to_lowercase().to_string()
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

    #[test]
    fn case_insensitive_search_handles_combining_sequences() {
        let results = find_all(&lines(&["e\u{301}x E\u{301}X"]), "E\u{301}X", false);
        assert_eq!(results.len(), 2);
    }
}
