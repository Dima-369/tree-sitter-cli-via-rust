use clap::{Arg, ArgAction, ArgMatches};
use std::io;
use std::io::Write;
use std::process::exit;
use tree_sitter::{Node, Parser, Query, StreamingIterator, Tree};

static LANGUAGES: [&str; 10] = [
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
    }
    .unwrap_or_else(|_| panic!("Error loading {} grammar", language));
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
        string.replace("\\", "\\\\")
            .replace("\"", "\\\"")
            .replace("\n", "\\n")
            .replace("\r", "\\r")
            .replace("\t", "\\t")
            .replace("\x08", "\\b")
            .replace("\x0c", "\\f")
    }

    fn process_node(node: Node, graph_string: &mut String, code: &String) {
        let node_id = format!("node_{}", node.id());
        let node_content = node
            .utf8_text(code.as_ref())
            .expect("Converting to UTF8 with the node range should succeed");
        let max_node_content_length = 60;
        let truncated_node_content = if node_content.len() > max_node_content_length {
            format!("{}...", &node_content[..max_node_content_length])
        } else {
            node_content.to_string()
        };
        // try to escape arbitrary input
        // https://forum.graphviz.org/t/how-do-i-properly-escape-arbitrary-text-for-use-in-labels/1762/9
        let escaped_node_content = escape_string(&*truncated_node_content);
        graph_string.push_str(&format!(
            "{}[label=\"{} {} {}\n{}\"];\n",
            node_id,
            node.kind().replace("\"", "\\\""),
            node.byte_range().start,
            node.byte_range().end,
            escaped_node_content
        ));
        for child in node.children(&mut node.walk()) {
            let child_id = format!("node_{}", child.id());
            graph_string.push_str(&format!("{} -> {}[label=\"\"];\n", node_id, child_id));
            process_node(child, graph_string, code);
        }
    }

    let mut graph_string = "".to_string();
    let root_node = tree.root_node();
    process_node(root_node, &mut graph_string, code);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn run_test_with_highlights<S: AsRef<str>>(
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

    /// Note that the resulting graphviz code is printed to stdout and needs to be externally validated, like
    /// with https://dreampuf.github.io/GraphvizOnline/?engine=dot#digraph
    ///
    /// See test_dot_graph_creation_via_dot_process() which validates it automatically.
    #[test]
    fn test_dot_graph_simple() {
        let mut output = Vec::new();
        let args = get_command().get_matches_from(vec![
            "main",
            "--graphviz-only",
            "--code",
            // test with quotes for correct escaping
            "test = \"1\"",
            "--language",
            "python",
        ]);
        handle_args(args, &mut output);
        let output = String::from_utf8(output).expect("Output array should be UTF-8");
        println!("{}", output);
        // not testing node IDs since they are random on every invocation
        assert!(output.starts_with("digraph name {\n"));
        assert!(output.ends_with("\n}"));
        assert_eq!(output.lines().count(), 28);
    }

    /// Validate if the generated graph code has valid syntax via the dot process which should be on the PATH.
    #[test]
    fn test_dot_graph_creation_via_dot_process() {
        let mut output = Vec::new();
        let args = get_command().get_matches_from(vec![
            "main",
            "--graphviz-only",
            "--code",
            // test with brackets and escaped quotes for correct escaping
            r#"fn main() {
                let test = "\"1\""; // comment
            }"#,
            "--language",
            "rust",
        ]);
        handle_args(args, &mut output);
        let graphviz_code = String::from_utf8(output).expect("Output array should be UTF-8");
        let mut dot_process = std::process::Command::new("dot")
            // If this is used to convert to PNG, the process will never terminate. Maybe it keeps waiting on stdin?
            // Or some other issue with the argument escaping?
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("dot application should be available on the PATH");
        if let Some(ref mut stdin) = dot_process.stdin {
            stdin
                .write_all(graphviz_code.as_bytes())
                .expect("stdin should be writable");
        }
        let result = dot_process
            .wait()
            .expect("Process should exit, but did not?");
        if !result.success() {
            let mut stdout = String::new();
            let mut stderr = String::new();
            if let Some(ref mut stdout_pipe) = dot_process.stdout {
                stdout_pipe
                    .read_to_string(&mut stdout)
                    .expect("stdout should be readable");
            }
            if let Some(ref mut stderr_pipe) = dot_process.stderr {
                stderr_pipe
                    .read_to_string(&mut stderr)
                    .expect("stderr should be readable");
            }
            eprintln!("{}", graphviz_code);
            panic!(
                "dot failed creating PNG from the generated graphviz code with exit code {}: {} {}",
                result.code().unwrap(),
                stdout,
                stderr
            );
        }
    }

    #[test]
    fn test_kotlin() {
        run_test_with_highlights(
            "let test = 1",
            "kotlin",
            tree_sitter_kotlin::HIGHLIGHTS_QUERY,
            r#"variable 0 3
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

}
