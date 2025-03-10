use std::collections::HashMap;
use tree_sitter::{Node, Tree};

pub fn generate_dot_graph(tree: &Tree, code: &String) -> String {
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

#[cfg(test)]
mod tests {
    use crate::{get_command, handle_args};
    use std::io::{Read, Write};

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

    /// Verifies that:
    /// 1. The graphviz output is deterministic (same input produces identical output)
    /// 2. Node IDs in the generated graph are sequential starting from 1
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