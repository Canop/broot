//! This checks some edge cases of pattern searches, especially composite patterns.
//! Don't hesitate to suggest more tests for clarification or to prevent regressions.
use {
    broot::{
        command::CommandParts,
        pattern::*,
    },
};

fn build_pattern(s: &str) -> Pattern {
    let cp = CommandParts::from(s);
    let search_modes = SearchModeMap::default();
    cp.pattern.print_tree();
    Pattern::new(
        &cp.pattern,
        &search_modes,
        0, // we don't do content search here
    ).unwrap()
}

fn check(
    pattern: &str,
    haystack: &str,
    expected: bool,
) {
    println!("applying pattern {:?} on {:?}", pattern, haystack);
    let pattern = build_pattern(pattern);
    //dbg!(&pattern);
    let found = pattern.search_string(haystack).is_some();
    assert_eq!(found, expected);
}

#[test]
fn simple_fuzzy() {
    check("toto", "toto", true);
    check("toto", "Toto", true);
    check("toto", "ToTuTo", true);
    check("tota", "ToTuTo", false);
}

#[test]
fn simple_exact() {
    check("e/toto", "toto", true);
    check("e/toto", "Toto", false);
    check("e/toto", "Tototo", true);
}

#[test]
fn simple_regex() {
    check("/toto", "toto", true);
    check("/to{3,5}to", "toto", false);
    check("/to{3,5}to", "tooooto", true);
    check("/to{3,5}to", "toooooooto", false);
    check("/to{3,5}to", "tooOoto", false);
    check("/to{3,5}to/i", "tooOoto", true);
}

#[test]
fn one_operator() {
    check("a&b", "a", false);
    check("a&b", "ab", true);
    check("a|b", "ab", true);
    check("a|b", "b", true);
    check("a|b", "c", false);
}

#[test]
fn negation() {
    check("!ab", "a", true);
    check("!ab", "aB", false);
    check("a&!b", "aB", false);
    check("a&!b", "aA", true);
    check("!a&!b", "ccc", true);
    check("!a|!b", "ccc", true);
    check("!a|!b", "cac", true);
    check("!a|!b", "cbc", true);
    check("!a|!b", "cbac", false);
}

// remember: it's left to right
#[test]
fn multiple_operators_no_parenthesis() {
    check("ab|ac|ad", "ab", true);
    check("ab|ac|ad", "ac", true);
    check("ab|ac|ad", "ad", true);
    check("ab|ac|ad|af|ag|er", "ad", true);
    check("ab&ac&ad", "ad", false);
    check("ab&ac&ad", "abcd", true);
    check("ab|ac|ad|ae", "ad", true);
    check("ab|ac|ad&ae", "ad", false);
    check("ab|ac|ad&ae", "axcd", false);
    check("ab|ac|ad&ae", "abe", true);
    check("ab|ac&ad|ae", "abd", true);
    check("ab|ac&ad|ae", "abc", false);
}

#[test]
fn multiple_operators_with_parenthesis() {
    check("ab|(ac|ad)", "ab", true);
    check("(ab|ac)|ad", "ac", true);
    check("ab|(ac|ad)|ae", "ad", true);
    check("ab|ac|(ad&ae)", "ac", true);
    check("ab|ac|(ad&ae)", "ad", false);
    check("(ab|ac)&(ad|ae)", "ad", false);
    check("!(ab|ac)&(ad|ae)", "ad", true);
    check("ab|(ac&ad|ae)", "abc", true);
    check("(ab|ac)&(ad|ae)", "abd", true);
}
