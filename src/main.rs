use clap::{Arg, Command};
use std::fs;
use tree_sitter::Parser;
use tree_sitter::Query;

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

fn create_parser(language: &str) -> Parser {
    let mut parser = Parser::new();
    let result = match language {
        "kotlin" => parser.set_language(&tree_sitter_kotlin::language()),
        "php" => {
            let language = tree_sitter_php::LANGUAGE_PHP;
            parser.set_language(&language.into())
        }
        _ => panic!("Unsupported language: {}", language),
    };
    // TODO show parsed error
    result.expect(&format!(
        "Error loading specified {} language grammar",
        language
    ))
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
    let language = "kotlin";
    let highlights = fs::read_to_string(
        "/Users/dima/Developer/java-tree-sitter/tree-sitter-kotlin/queries/highlights.scm",
    )
    .unwrap();
    let parser = create_parser(language);
    let parsed = parser.parse(code, None);
    let tree = parsed.expect(&format!(
        "Failed to parse passed code with language {}",
        language
    ));
    let root_node = tree.root_node();
    let query = Query::new(&tree_sitter_language, &*highlights).expect("Failed to create query");
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let matches = query_cursor.matches(&query, root_node, code.as_bytes());
    for m in matches {
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
