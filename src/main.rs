use clap::{Arg, ArgAction, ArgMatches};
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::process::exit;
use tree_sitter::{Node, Parser, Query, StreamingIterator, Tree};

static LANGUAGES: [&str; 12] = [
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

    /// Generate the same graph for the same code
    #[test]
    fn test_dot_graph_stable_ids() {
        let code = r#"let x = 1;"#;
        let mut output1 = Vec::new();
        let mut output2 = Vec::new();
        let args = get_command().get_matches_from(vec![
            "main",
            "--graphviz-only",
            "--code",
            code,
            "--language",
            "rust",
        ]);
        handle_args(args.clone(), &mut output1);
        handle_args(args, &mut output2);
        let output1 = String::from_utf8(output1).expect("Output array should be UTF-8");
        let output2 = String::from_utf8(output2).expect("Output array should be UTF-8");
        assert_eq!(output1, output2);

        // Verify node IDs are sequential
        let node_ids: Vec<_> = output1
            .lines()
            // filter out node connections which have ->
            .filter(|line| line.starts_with("node_") && !line.contains("->"))
            .map(|line| {
                let id = line
                    .split('[')
                    .next()
                    .unwrap()
                    .trim()
                    .strip_prefix("node_")
                    .expect("Node ID should start with 'node_'")
                    .parse::<usize>()
                    .unwrap_or_else(|_| {
                        eprintln!("Failed to parse node ID from string: {}", line);
                        panic!("Node ID should be a valid number");
                    });
                id
            })
            .collect();

        // Check that IDs start at 1 and are sequential
        let mut expected_id = 1;
        for &id in node_ids.iter() {
            assert_eq!(
                id, expected_id,
                "Node IDs should be sequential starting from 1"
            );
            expected_id += 1;
        }
    }
}
