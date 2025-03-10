use super::common::run_test_with_highlights;

#[test]
fn test_kotlin() {
    run_test_with_highlights(
        "val test = 1",
        "kotlin",
        tree_sitter_kotlin::HIGHLIGHTS_QUERY,
        r#"keyword 0 3
variable 4 8
operator 9 10
number 11 12
"#,
    )
}

#[test]
fn test_php() {
    run_test_with_highlights(
        "<?php $test = 1;",
        "php",
        tree_sitter_php::HIGHLIGHTS_QUERY,
        r#"tag 0 5
variable 6 11
operator 6 7
number 14 15
"#,
    );
}

#[test]
fn test_bash() {
    run_test_with_highlights(
        "echo 'hi'",
        "bash",
        tree_sitter_bash::HIGHLIGHT_QUERY,
        r#"function 0 4
string 5 9
"#,
    )
}

#[test]
fn test_json() {
    run_test_with_highlights(
        "{\"test\": 1}",
        "json",
        tree_sitter_json::HIGHLIGHTS_QUERY,
        r#"string.special.key 1 7
string 1 7
number 9 10
"#,
    )
}

#[test]
fn test_dockerfile() {
    run_test_with_highlights(
        "FROM apache:latest",
        "dockerfile",
        tree_sitter_dockerfile::HIGHLIGHTS_QUERY,
        r#"keyword 0 4
operator 11 12
"#,
    )
}

#[test]
fn test_python() {
    run_test_with_highlights(
        "test = 1",
        "python",
        tree_sitter_python::HIGHLIGHTS_QUERY,
        r#"variable 0 4
operator 5 6
number 7 8
"#,
    )
}

#[test]
fn test_java() {
    run_test_with_highlights(
        "package test",
        "java",
        tree_sitter_java::HIGHLIGHTS_QUERY,
        r#"keyword 0 7
variable 8 12
"#,
    )
}

#[test]
fn test_rust() {
    run_test_with_highlights(
        "static TEST: i32 = 1;",
        "rust",
        tree_sitter_rust::HIGHLIGHTS_QUERY,
        r#"keyword 0 6
constructor 7 11
punctuation.delimiter 11 12
type.builtin 13 16
constant.builtin 19 20
punctuation.delimiter 20 21
"#,
    )
}

#[test]
fn test_lua() {
    run_test_with_highlights(
        "test = 1",
        "lua",
        tree_sitter_lua::HIGHLIGHTS_QUERY,
        r#"variable 0 4
operator 5 6
number 7 8
"#,
    )
}

#[test]
fn test_toml() {
    run_test_with_highlights(
        "[package]",
        "toml",
        tree_sitter_toml::HIGHLIGHT_QUERY,
        r#"punctuation.bracket 0 1
property 1 8
punctuation.bracket 8 9
"#,
    )
}

/// It returns "string 11 17", because the 😄 emoji is 4 bytes, the " character are in the string twice,
/// so 17 - 11 = 6 bytes for the string in total. This is important for compatibility with Kotlin Emacs.
#[test]
fn test_emojis() {
    run_test_with_highlights(
        "val test = \"😄\"\nval test = \"😄\"",
        "kotlin",
        tree_sitter_kotlin::HIGHLIGHTS_QUERY,
        r#"keyword 0 3
variable 4 8
operator 9 10
string 11 17
keyword 18 21
variable 22 26
operator 27 28
string 29 35
"#,
    )
}

/// Parity checks to Kotlin with the result: It is the same in Kotlin.
#[test]
fn test_string_byte_count() {
    assert_eq!("val test =\"😄😄😄\"".len(), 24);
    assert_eq!("val".len(), 3);
    assert_eq!("😄".len(), 4);
}

#[test]
fn test_groovy() {
    run_test_with_highlights(
        "apply plugin: 'java'",
        "groovy",
        tree_sitter_groovy::HIGHLIGHTS_QUERY,
        r#"variable 0 5
variable 6 12
string 14 20
"#,
    )
}

#[test]
fn test_css() {
    run_test_with_highlights(
        ".test { color: red; }",
        "css",
        tree_sitter_css::HIGHLIGHTS_QUERY,
        r#"property 1 5
property 8 13
punctuation.delimiter 13 14
"#,
    )
}

#[test]
fn test_html() {
    run_test_with_highlights(
        "<!DOCTYPE html><p>hi</p>",
        "html",
        tree_sitter_html::HIGHLIGHTS_QUERY,
        r#"constant 0 15
punctuation.bracket 14 15
punctuation.bracket 15 16
tag 16 17
punctuation.bracket 17 18
punctuation.bracket 20 22
tag 22 23
punctuation.bracket 23 24
"#,
    )
}

#[test]
fn test_javascript() {
    run_test_with_highlights(
        "const test = 1;",
        "javascript",
        tree_sitter_javascript::HIGHLIGHT_QUERY,
        r#"keyword 0 5
variable 6 10
operator 11 12
number 13 14
punctuation.delimiter 14 15
"#,
    )
}
