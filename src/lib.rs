
use pyo3::prelude::*;
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

/// Checks if a pattern containing wildcards matches a given string.
///
/// This function checks if a pattern containing wildcards will match a given string.
/// The valid wildcards are:
/// `*`, which matches any number of characters, including none.
/// `?`, which matches exactly one of any characters.
///
/// # Arguments
///
/// * `pattern` - The pattern to check for.
/// * `text` - The text to check the pattern on.
///
/// # Examples
/// ```
/// use pyglob::is_wildcard_match;
/// let does_match = is_wildcard_match("aplbq", "a*b?");
/// assert_eq!(does_match, true);
/// let doesnt_match = is_wildcard_match("abc", "a*b");
/// assert_eq!(doesnt_match, false);
/// ```
#[pyfunction]
pub fn is_wildcard_match(text: &str, pattern: &str) -> bool {
    // Convert the pattern and text in to vectors of graphemes
    let pattern_graphemes = pattern.graphemes(true).collect::<Vec<&str>>();
    let text_graphemes = text.graphemes(true).collect::<Vec<&str>>();

    // Try to preprocess the pattern
    // let (pattern_graphemes, text_graphemes) = preprocessing(pattern_graphemes, text_graphemes);

    // Otherwise start our dynamic programming matching
    match_with_cache(&pattern_graphemes, &text_graphemes)
}

#[pymodule]
fn pyglob(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(is_wildcard_match, m)?)?;
    Ok(())
}

/// This applies preprocessing to the pattern to speed up matching
fn preprocessing<'a, 'b>(
    pattern: Vec<&'a str>,
    text: Vec<&'b str>,
) -> (Vec<&'a str>, Vec<&'b str>) {
    let pattern = remove_duplicate_stars(pattern);
    remove_matching_start_and_end(pattern, text)
}

/// Remove any duplicate stars, because they do not impact the matching
fn remove_duplicate_stars(mut pattern: Vec<&str>) -> Vec<&str> {
    let mut i: usize = 1;
    while i < pattern.len() {
        if pattern[i] == "*" && pattern[i - 1] == "*" {
            pattern.remove(i);
        } else {
            i += 1;
        }
    }
    pattern
}

/// If the start and end of two strings match, we can pre-emptively strip them
fn remove_matching_start_and_end<'a, 'b>(
    mut pattern: Vec<&'a str>,
    mut text: Vec<&'b str>,
) -> (Vec<&'a str>, Vec<&'b str>) {
    // Remove matching items from the start
    let mut i: usize = 0;
    while (
        i < pattern.len() && i < text.len()
        // Check that we're not at the end of the string
    ) && ((
        pattern[i] == text[i] && text[i] != "*"
        // Check if the characters match
    ) || (
        pattern[i] == "?"
        // Or check if the pattern is a question mark
    )) {
        pattern.remove(i);
        text.remove(i);
        i += 1;
    }
    if pattern.len() == 0 || text.len() == 0 {
        return (pattern, text);
    }

    // Remove matching items from the end
    let mut i: usize = pattern.len() - 1;
    let mut j: usize = text.len() - 1;
    while (i > 0 && j > 0)
        && ((
            pattern[i] == text[j] && text[j] != "*"
            // Check if the characters match
        ) || (
            pattern[i] == "?"
            // Or check if the pattern is a question mark
        ))
    {
        pattern.remove(i);
        text.remove(j);
        i -= 1;
        j -= 1;
    }
    (pattern, text)
}

fn match_with_cache(pattern: &Vec<&str>, text: &Vec<&str>) -> bool {
    // Create a cache
    let mut cache: HashMap<(usize, usize), bool> = HashMap::new();

    // Set the starting position where both strings are empty as `true`
    cache.insert((1, 1), true);

    set_cache(&mut cache, pattern, text, pattern.len() + 1, text.len() + 1);
    *cache.get(&(pattern.len() + 1, text.len() + 1)).unwrap()
}

/// A dynamic solution to the pattern matching, with the help of this video:
/// https://www.youtube.com/watch?v=3ZDZ-N0EPV0
///
/// `row` and `column` indexes are indexed by 1, so that we can use 0 as a "border"
fn set_cache(
    cache: &mut HashMap<(usize, usize), bool>,
    pattern: &Vec<&str>,
    text: &Vec<&str>,
    row: usize,
    column: usize,
) {
    // If we already have the item in the cache, return
    if cache.contains_key(&(row, column)) {
        return;
    }

    if row == 0 {
        cache.insert((row, column), false);
        return;
    }
    if column == 0 {
        cache.insert((row, column), false);
        return;
    }

    // Get character of the pattern at the current row
    let pattern_char = if row == 1 { "" } else { pattern[row - 2] };

    // Get the character of the text at the current column
    let text_char = if column == 1 { "" } else { text[column - 2] };

    // If the patter character matches the text character, take the value from the top left
    if (pattern_char == text_char && text_char != "*") || pattern_char == "?" {
        set_cache(cache, pattern, text, row - 1, column - 1);
        // Copy the value from the top left
        cache.insert(
            (row, column),
            *cache.get(&(row - 1, column - 1)).unwrap_or(&false),
        );
        return;
    }

    // If the pattern character is a star, then take a value from above or the left
    if pattern_char == "*" {
        set_cache(cache, pattern, text, row - 1, column);
        let left = cache.get(&(row - 1, column)).unwrap_or(&false);
        if *left == true {
            cache.insert((row, column), true);
            return;
        }

        set_cache(cache, pattern, text, row, column - 1);
        let right = cache.get(&(row, column - 1)).unwrap_or(&false);
        if *right == true {
            cache.insert((row, column), true);
            return;
        }
    }

    // If the strings don't match, and no wildcards matched, then this field is not a match.
    cache.insert((row, column), false);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_match() {
        assert_eq!(true, is_wildcard_match("alyib", "a?y*b"));
    }

    #[test]
    fn simple_non_match() {
        assert_eq!(false, is_wildcard_match("abcd", "a*b?"))
    }

    #[test]
    fn test_kanji_characters() {
        assert_eq!(true, is_wildcard_match("漢字", "??"))
    }

    #[test]
    fn empty_string_matches_star() {
        assert_eq!(true, is_wildcard_match("", "*"));
    }

    #[test]
    fn empty_string_doesnt_match_questionmark() {
        assert_eq!(false, is_wildcard_match("", "?"));
    }

    #[test]
    fn empty_string_matches_multiple_stars() {
        assert_eq!(true, is_wildcard_match("", "***"));
    }

    #[test]
    fn repeating_sequence_with_stars() {
        assert_eq!(true, is_wildcard_match("daaadabadmanda", "da*da*da"));
    }

    #[test]
    fn different_text_ending_doesnt_match() {
        assert_eq!(false, is_wildcard_match("testingmore", "testing"));
    }

    #[test]
    fn different_text_start_doesnt_match() {
        assert_eq!(false, is_wildcard_match("more, testing", "testing"));
    }

    #[test]
    fn star_and_questionmark_does_match() {
        assert_eq!(true, is_wildcard_match("xx", "*?"));
    }

    #[test]
    fn star_in_text_is_escaped() {
        assert_eq!(true, is_wildcard_match("a*", "*"));
    }

    #[test]
    fn empty_input_string() {
        assert_eq!(true, is_wildcard_match("", "*"))
    }

    #[test]
    fn empty_pattern_string() {
        assert_eq!(false, is_wildcard_match("test", ""))
    }

    #[test]
    fn both_strings_empty() {
        assert_eq!(true, is_wildcard_match("", ""))
    }

    #[test]
    fn long_test() {
        // assert_eq!(false, is_wildcard_match("**aa*****ba*a*bb**aa*ab****a*aaaaaa***a*aaaa**bbabb*b*b**aaaaaaaaa*a********ba*bbb***a*ba*bb*bb**a*b*bb", "abbabaaabbabbaababbabbbbbabbbabbbabaaaaababababbbabababaabbababaabbbbbbaaaabababbbaabbbbaabbbbababababbaabbaababaabbbababababbbbaaabbbbbabaaaabbababbbbaababaabbababbbbbababbbabaaaaaaaabbbbbaabaaababaaaabb"))
        assert_eq!(true, is_wildcard_match("abbabaaabbabbaababbabbbbbabbbabbbabaaaaababababbbabababaabbababaabbbbbbaaaabababbbaabbbbaabbbbababababbaabbaababaabbbababababbbbaaabbbbbabaaaabbababbbbaababaabbababbbbbababbbabaaaaaaaabbbbbaabaaababaaaabb", "abbabaaabbabbaababbabbbbbabbbabbbabaaaaababababbbabababaabbababaabbbbbbaaaabababbbaabbbbaabbbbababa*babbaabbaababaabbbababababbbbaaabbbbbabaaaabbababbbbaababaabbababbbbbababbbabaaaaaaaabbbbbaabaaababaaaabb"))
    }
}
