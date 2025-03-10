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

enum Language {
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