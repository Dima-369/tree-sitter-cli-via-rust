use std::io::Write;
use std::process::exit;
use tree_sitter::{Parser, Query, StreamingIterator, Tree};

pub static LANGUAGES: [&str; 14] = [
    "kotlin",
    "php",
    "bash",
    "json",
    "dockerfile",
    "python",
    "java",
    "rust",
    "lua",
    "toml",
    "groovy",
    "css",
    "html",
    "javascript",
];

pub enum Language {
    Kotlin,
    Php,
    Bash,
    Json,
    Dockerfile,
    Python,
    Java,
    Rust,
    Lua,
    Toml,
    Groovy,
    Css,
    Html,
    Javascript,
}

pub fn map_language_to_enum(language: &str) -> Language {
    match language {
        "kotlin" => Language::Kotlin,
        "php" => Language::Php,
        "bash" => Language::Bash,
        "json" => Language::Json,
        "dockerfile" => Language::Dockerfile,
        "python" => Language::Python,
        "java" => Language::Java,
        "rust" => Language::Rust,
        "lua" => Language::Lua,
        "toml" => Language::Toml,
        "groovy" => Language::Groovy,
        "css" => Language::Css,
        "html" => Language::Html,
        "javascript" => Language::Javascript,
        _ => panic!("Unsupported language: {}", language),
    }
}

pub fn set_parser_language(language: &&String, parser: &mut Parser, language_enum: Language) {
    match language_enum {
        Language::Kotlin => parser.set_language(&tree_sitter_kotlin::language()),
        Language::Php => parser.set_language(&tree_sitter_php::LANGUAGE_PHP.into()),
        Language::Bash => parser.set_language(&tree_sitter_bash::LANGUAGE.into()),
        Language::Json => parser.set_language(&tree_sitter_json::LANGUAGE.into()),
        Language::Dockerfile => parser.set_language(&tree_sitter_dockerfile::language()),
        Language::Python => parser.set_language(&tree_sitter_python::LANGUAGE.into()),
        Language::Java => parser.set_language(&tree_sitter_java::LANGUAGE.into()),
        Language::Rust => parser.set_language(&tree_sitter_rust::LANGUAGE.into()),
        Language::Lua => parser.set_language(&tree_sitter_lua::LANGUAGE.into()),
        Language::Toml => parser.set_language(&tree_sitter_toml::LANGUAGE.into()),
        Language::Groovy => parser.set_language(&tree_sitter_groovy::LANGUAGE.into()),
        Language::Css => parser.set_language(&tree_sitter_css::LANGUAGE.into()),
        Language::Html => parser.set_language(&tree_sitter_html::LANGUAGE.into()),
        Language::Javascript => parser.set_language(&tree_sitter_javascript::LANGUAGE.into()),
    }
        .unwrap_or_else(|_| panic!("Error loading {} grammar", language))
}

pub fn process_query<W>(parser: Parser, highlights: &str, tree: &Tree, code: &String, writer: &mut W)
where
    W: Write,
{
    let parser_language = parser.language().unwrap();
    let query = match Query::new(&parser_language, highlights) {
        Ok(query) => query,
        Err(_) => {
            eprintln!("Failed to create query for passed highlights");
            exit(1);
        }
    };
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let mut matches = query_cursor.matches(&query, tree.root_node(), code.as_bytes());
    while let Some(m) = matches.next() {
        for capture in m.captures {
            let node = capture.node;
            let capture_name = query.capture_names()[capture.index as usize];
            writeln!(
                writer,
                "{} {} {}",
                capture_name,
                node.byte_range().start,
                node.byte_range().end
            )
                .expect("write should succeed");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{get_command, handle_args};

    pub fn run_test_with_highlights<S: AsRef<str>>(
        code: S,
        language: S,
        highlights_query: &str,
        expected_output: &str,
    ) {
        let mut output = Vec::new();
        let args = get_command().get_matches_from(vec![
            "main",
            "--code",
            code.as_ref(),
            "--language",
            language.as_ref(),
            "--highlights",
            highlights_query,
        ]);
        handle_args(args, &mut output);
        let output = String::from_utf8(output).expect("Output array should be UTF-8");
        assert_eq!(expected_output, output);
    }


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

}