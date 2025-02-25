use clap::{Arg, Command};
use std::fs;
use tree_sitter::{Parser, StreamingIterator};
use tree_sitter::{Query, Tree};

static LANGUAGES: [&str; 2] = ["kotlin", "php"];

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

fn get_parser_language(language: &str) -> tree_sitter_language::LanguageFn {
    match language {
        "kotlin" => tree_sitter_kotlin_ng::LANGUAGE,
        "php" => tree_sitter_php::LANGUAGE_PHP,
        _ => panic!("Unsupported language: {}", language),
    }
}

fn create_parser(language: &str, parser_language: tree_sitter_language::LanguageFn) -> Parser {
    let mut parser = Parser::new();
    match parser.set_language(&parser_language.into()) {
        Ok(_) => parser,
        Err(err) => panic!(
            "Error loading specified {} language grammar: {:?}",
            language, err
        ),
    }
}

fn main() {
    /*    let matches = get_command().get_matches();
        let code = matches.get_one::<String>("code").unwrap();
        let language = matches.get_one::<String>("language").unwrap();
        let highlights = matches.get_one::<String>("highlights").unwrap();
    */
    // TODO remove debug code
    let code = r#"
  data class Point(
    val x: Int,
    val y: Int
  )
"#;
    let language = "php";
    let highlights = fs::read_to_string(
        "/Users/dima/Developer/java-tree-sitter/tree-sitter-kotlin/queries/highlights-backup.scm",
    ).unwrap();
    let parser_language = get_parser_language(language);
    let mut parser = create_parser(language, parser_language);
    let old_tree: Option<&Tree> = None;
    let parsed = parser.parse(code, old_tree);
    let tree = parsed.expect(&format!(
        "Failed to parse passed code with language {}",
        language
    ));
    let root_node = tree.root_node();
    // let query = Query::new(&tree_sitter_kotlin_ng::LANGUAGE.into(), &*highlights).expect("Failed to create query");
    // TODO why does this crash?
    let query = Query::new(&parser_language.into(), "(simple_identifier) @variable")
        .expect("Failed to create query");
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
