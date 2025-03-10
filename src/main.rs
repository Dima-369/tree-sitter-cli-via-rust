mod tests;

use clap::{Arg, ArgAction, ArgMatches};
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::process::exit;
use tree_sitter::{Node, Parser, Query, StreamingIterator, Tree};

static LANGUAGES: [&str; 14] = [
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
fn map_language_to_enum(language: &str) -> Language {
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
fn set_parser_language(language: &&String, parser: &mut Parser, language_enum: Language) {
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

fn get_command() -> clap::Command {
    clap::Command::new("Tree-sitter Syntax Highlighter")
        .version("1.0")
        .author("Dmytro Butemann <dbutemann@gmail.com>")
        .about("Outputs capture names with byte ranges or graphviz code using Tree-sitter for Kotlin Emacs.")
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
                .help("String of highlights like the content of queries/highlights.scm. This is required when not using --graphviz-only")
        )
        .arg(
            Arg::new("graphviz-only")
                .long("graphviz-only")
                .action(ArgAction::SetTrue)
                .help("If passed, output only the graphviz dot graph"),
        )
}

fn generate_dot_graph(tree: &Tree, code: &String) -> String {
    fn escape_string(string: &str) -> String {
        string
            .replace("\\", "\\\\")
            .replace("\"", "\\\"")
            .replace("\n", "\\n")
            .replace("\r", "\\r")
            .replace("\t", "\\t")
            .replace("\x08", "\\b") // backspace
            .replace("\x0c", "\\f") // form feed
    }

    fn process_node(
        node: Node,
        graph_string: &mut String,
        code: &String,
        id_map: &mut HashMap<usize, usize>,
    ) {
        let next_id = id_map.len() + 1;
        let stable_id = *id_map.entry(node.id()).or_insert(next_id);
        let node_id = format!("node_{}", stable_id);

        let node_content = node
            .utf8_text(code.as_ref())
            .expect("Converting to UTF8 with the node range should succeed");
        let max_node_content_length = 60;
        let truncated_node_content = if node_content.len() > max_node_content_length {
            format!("{}...", &node_content[..max_node_content_length])
        } else {
            node_content.to_string()
        };

        let escaped_node_content = escape_string(&truncated_node_content);
        graph_string.push_str(&format!(
            "{}[label=\"{} {} {}\n{}\"];\n",
            node_id,
            node.kind().replace("\"", "\\\""),
            node.byte_range().start,
            node.byte_range().end,
            escaped_node_content
        ));

        for child in node.children(&mut node.walk()) {
            let next_child_id = id_map.len() + 1;
            let child_stable_id = *id_map.entry(child.id()).or_insert(next_child_id);
            let child_id = format!("node_{}", child_stable_id);
            graph_string.push_str(&format!("{} -> {}[label=\"\"];\n", node_id, child_id));
            process_node(child, graph_string, code, id_map);
        }
    }

    let mut graph_string = String::new();
    let mut id_map = HashMap::new();
    let root_node = tree.root_node();
    process_node(root_node, &mut graph_string, code, &mut id_map);
    format!("digraph name {{\n{}}}", graph_string)
}

fn handle_args<W>(args: ArgMatches, mut writer: W)
where
    W: Write,
{
    let code = args.get_one::<String>("code").unwrap();
    let language = args.get_one::<String>("language").unwrap();
    let graphviz_only = args.get_one::<bool>("graphviz-only").unwrap();
    let highlights = args.get_one::<String>("highlights");
    if !graphviz_only && highlights.is_none() {
        eprintln!("--highlights is required when not using --graphviz-only");
        exit(1);
    }
    let mut parser = Parser::new();
    let language_enum = map_language_to_enum(language);
    set_parser_language(&language, &mut parser, language_enum);
    let tree = parser.parse(code, None).unwrap();
    if *graphviz_only {
        write!(writer, "{}", generate_dot_graph(&tree, code))
            .expect("writing dot graph should succeed");
    } else {
        process_query(parser, highlights.unwrap(), &tree, code, &mut writer);
    }
}

fn process_query<W>(parser: Parser, highlights: &str, tree: &Tree, code: &String, writer: &mut W)
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

fn main() {
    let args = get_command().get_matches();
    handle_args(args, io::stdout());
}