use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use regex::Regex;

use super::Region;

/// Split a translated text into individual words, while filtering out function words
pub fn extract(string: &str, function_words: &HashSet<String>) -> HashSet<String> {
    static REGEX: OnceLock<Regex> = OnceLock::new();

    // split the string based on any whitespace, comma, full stop, colon, or semicolon or brackets
    let regex = REGEX.get_or_init(|| Regex::new(r#"[\s,\.:;\[\]\(\)\{\}]"#).unwrap());

    regex
        .split(string) // split the string every time teh regex is encountered
        .map(str::trim) // remove any leading or trailing whitespace
        .map(str::to_lowercase) // convert the strings to lowercase
        .filter(|s| !s.is_empty()) // remove any empty strings
        .filter(|s| s.chars().all(|c| c.is_alphabetic() || c == '\'')) // remove any words that contain anything other than text or apostrophe
        .filter(|s| !function_words.contains(s)) // remove any function words
        .collect()
}

/// A utility function that calls [`extract`] on every translation in a set of ChatGPT translations
pub fn extract_chatgpt(
    chatgpt: HashMap<String, (String, Region)>,
    function_words: &HashSet<String>,
) -> HashMap<String, (HashSet<String>, Region)> {
    chatgpt
        .into_iter()
        .map(|(prompt_name, (translation, region))| {
            let extracted = extract(&translation, function_words);
            (prompt_name, (extracted, region))
        })
        .collect()
}

#[test]
fn extract_words_works() {
    let function_words = HashSet::from_iter(["hello".to_string()]);
    let set = extract("hello WORld,foo123 234;345(a){b}[c]", &function_words);
    let expected: HashSet<_> = ["world", "foo123", "234", "345", "a", "b", "c"]
        .into_iter()
        .map(ToString::to_string)
        .collect();

    assert_eq!(set, expected);
}
