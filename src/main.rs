use std::fs;
use tree_sitter::Parser;
use tree_sitter::Query;

// TODO use click and allow setting language enum for kotlin and php
// TODO use click and pass highlights string

fn main() {
    let code = r#"
  data class Point(
    val x: Int,
    val y: Int
  )
"#;
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_kotlin::language())
        .expect("Error loading Kotlin grammar");
    let parsed = parser.parse(code, None);
    let tree = parsed.unwrap();
    let root_node = tree.root_node();

    println!("{:#?}", root_node);
    println!("{:#?}", root_node.kind());
    println!("{:#?}", root_node.start_position());
    println!("{:#?}", root_node.end_position());

    let query_source = fs::read_to_string(
        "/Users/dima/Developer/java-tree-sitter/tree-sitter-kotlin/queries/highlights.scm",
    )
    .expect("Failed to read highlights.scm file");

    let query =
        Query::new(&tree_sitter_kotlin::language(), &*query_source).expect("Failed to create query");

    let mut query_cursor = tree_sitter::QueryCursor::new();
    let matches = query_cursor.matches(&query, root_node, code.as_bytes());

    for m in matches {
        for capture in m.captures {
            let node = capture.node;
            let capture_name = query.capture_names()[capture.index as usize];
            println!(
                "Capture name: {}, text: {}",
                capture_name,
                node.utf8_text(code.as_bytes()).unwrap()
            );
        }
    }
}
