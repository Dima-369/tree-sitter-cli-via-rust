use super::*;
use std::io::Read;

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
