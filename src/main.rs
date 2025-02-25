use clap::{Arg, Command};
use tree_sitter::Query;
use tree_sitter::{Parser, StreamingIterator};

static LANGUAGES: [&str; 7] = [
    "kotlin",
    "php",
    "bash",
    "json",
    "dockerfile",
    "python",
    "java",
];
enum Language {
    Kotlin,
    Php,
    Bash,
    Json,
    Dockerfile,
    Python,
    Java,
}
fn map_language_to_enum(language: &str) -> Language {
    match language {
        "kotlin" => Language::Kotlin,
        "php" => Language::Php,
        "bash" => Language::Bash,
        "json" => Language::Json,
        "dockerfile" => Language::Dockerfile,
        "python" => Language::Python,
        "java" => Language::Java,
        _ => panic!("Unsupported language: {}", language),
    }
}
fn set_parser_language(language: &&String, parser: &mut Parser, language_enum: Language) {
    match language_enum {
        Language::Kotlin => parser.set_language(&tree_sitter_kotlin::language()),
        Language::Php => parser.set_language(&tree_sitter_php::LANGUAGE_PHP.into()),
        Language::Bash => parser.set_language(&tree_sitter_bash::LANGUAGE.into()),
        Language::Json => parser.set_language(&tree_sitter_json::LANGUAGE.into()),
        Language::Dockerfile => parser.set_language(&tree_sitter_dockerfile::language().into()),
        Language::Python => parser.set_language(&tree_sitter_python::LANGUAGE.into()),
        Language::Java => parser.set_language(&tree_sitter_java::LANGUAGE.into()),
    }
        .expect(&format!("Error loading {} grammar", language));
}

fn get_command() -> Command {
    Command::new("Tree-sitter Syntax Highlighter")
        .version("1.0")
        .author("Dmytro Butemann <dbutemann@gmail.com>")
        .about("Outputs capture names with byte range using Tree-sitter for Kotlin Emacs.")
        .arg(
            Arg::new("code")
                .long("code")
                .help("The code to parse")
                .required(true),
        )
        .arg(
            Arg::new("language")
                .long("language")
                .value_parser(LANGUAGES)
                .required(true),
        )
        .arg(
            Arg::new("highlights")
                .long("highlights")
                .help("String of highlights like the content of queries/highlights.scm")
                .required(true),
        )
}

fn main() {
    let args = get_command().get_matches();
    let code = args.get_one::<String>("code").unwrap();
    let language = args.get_one::<String>("language").unwrap();
    let highlights = args.get_one::<String>("highlights").unwrap();
    let mut parser = Parser::new();
    let language_enum = map_language_to_enum(language);
    set_parser_language(&language, &mut parser, language_enum);
    let parsed = parser.parse(code, None);
    let tree = parsed.unwrap();
    let root_node = tree.root_node();
    let parser_language = parser.language().unwrap();
    let query = Query::new(&parser_language, highlights)
        .expect("Failed to create query for passed highlights");
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let mut matches = query_cursor.matches(&query, root_node, code.as_bytes());
    while let Some(m) = matches.next() {
        for capture in m.captures {
            let node = capture.node;
            let capture_name = query.capture_names()[capture.index as usize];
            println!(
                "{} {} {}",
                capture_name,
                node.byte_range().start,
                node.byte_range().end
            );
        }
    }
}