mod graphviz;
mod languages;

use crate::graphviz::generate_dot_graph;
use crate::languages::{map_language_to_enum, process_query, set_parser_language, LANGUAGES};
use clap::{Arg, ArgAction, ArgMatches};
use std::collections::HashMap;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::process::exit;
use tree_sitter::{Parser, Query, QueryCursor};

#[derive(serde::Deserialize)]
struct DaemonRequest {
    id: u64,
    code: String,
    language: String,
    highlights: String,
}

#[derive(serde::Serialize)]
struct DaemonResponse {
    id: u64,
    highlights: Vec<DaemonHighlight>,
    error: Option<String>,
}

#[derive(serde::Serialize)]
struct DaemonHighlight {
    #[serde(rename = "c")]
    capture: String,
    #[serde(rename = "s")]
    start_byte: usize,
    #[serde(rename = "e")]
    end_byte: usize,
}

pub fn get_command() -> clap::Command {
    clap::Command::new("Tree-sitter Syntax Highlighter")
        .version("1.0")
        .author("Dmytro Butemann <dbutemann@gmail.com>")
        .about("Outputs capture names with byte ranges or graphviz code using Tree-sitter for Kotlin Emacs.")
        .arg(
            Arg::new("daemon")
                .long("daemon")
                .action(ArgAction::SetTrue)
                .help("Read JSON-lines highlight requests from stdin and write JSON-lines responses to stdout"),
        )
        .arg(
            Arg::new("code")
                .long("code")
                .help("The code to parse")
        )
        .arg(
            Arg::new("language")
                .long("language")
                .value_parser(LANGUAGES)
        )
        .arg(
            Arg::new("highlights")
                .long("highlights")
                .help("String of highlights like the content of queries/highlights.scm. This is required when not using --graphviz-only")
        )
        .arg(
            Arg::new("highlights-file")
                .long("highlights-file")
                .help("Path to a highlights file (e.g., queries/highlights.scm). Alternative to --highlights.")
        )
        .arg(
            Arg::new("graphviz-only")
                .long("graphviz-only")
                .action(ArgAction::SetTrue)
                .help("If passed, output only the graphviz dot graph"),
        )
}

pub fn handle_args<W>(args: ArgMatches, mut writer: W)
where
    W: Write,
{
    if args.get_flag("daemon") {
        handle_daemon(io::stdin(), writer);
        return;
    }
    let Some(code) = args.get_one::<String>("code") else {
        eprintln!("--code is required when not using --daemon");
        exit(1);
    };
    let Some(language) = args.get_one::<String>("language") else {
        eprintln!("--language is required when not using --daemon");
        exit(1);
    };
    let graphviz_only = args.get_flag("graphviz-only");
    let highlights = args.get_one::<String>("highlights");
    let highlights_file = args.get_one::<String>("highlights-file");

    let highlights_content = if graphviz_only {
        String::new()
    } else if highlights.is_some() && highlights_file.is_some() {
        eprintln!("Error: Cannot use both --highlights and --highlights-file simultaneously");
        exit(1);
    } else if let Some(file_path) = highlights_file {
        match std::fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading highlights file '{}': {}", file_path, e);
                exit(1);
            }
        }
    } else if let Some(h) = highlights {
        h.clone()
    } else {
        eprintln!("--highlights or --highlights-file is required when not using --graphviz-only");
        exit(1);
    };

    let mut parser = Parser::new();
    let language_enum = map_language_to_enum(language);
    set_parser_language(language, &mut parser, language_enum);
    // Markdown grammar requires trailing newline to properly capture headings
    let code = if language == "markdown" && !code.ends_with('\n') {
        format!("{}\n", code)
    } else {
        code.to_string()
    };
    let tree = parser.parse(&code, None).unwrap();
    if graphviz_only {
        write!(writer, "{}", generate_dot_graph(&tree, &code))
            .expect("writing dot graph should succeed");
    } else {
        process_query(&parser, &highlights_content, &tree, &code, &mut writer);
    }
}

fn handle_daemon<R, W>(reader: R, writer: W)
where
    R: io::Read,
    W: Write,
{
    let reader = io::BufReader::new(reader);
    let mut writer = io::BufWriter::new(writer);
    let mut parser = Parser::new();
    let mut query_cursor = QueryCursor::new();
    let mut query_cache: HashMap<String, Query> = HashMap::new();
    for line in reader.lines() {
        let response = match line {
            Ok(line) => handle_daemon_line(&line, &mut parser, &mut query_cursor, &mut query_cache),
            Err(e) => DaemonResponse {
                id: 0,
                highlights: Vec::new(),
                error: Some(format!("Failed to read stdin: {}", e)),
            },
        };
        serde_json::to_writer(&mut writer, &response)
            .expect("writing JSON response should succeed");
        writeln!(writer).expect("writing newline should succeed");
        writer.flush().expect("flushing response should succeed");
    }
}

fn handle_daemon_line(
    line: &str,
    parser: &mut Parser,
    query_cursor: &mut QueryCursor,
    query_cache: &mut HashMap<String, Query>,
) -> DaemonResponse {
    let request: DaemonRequest = match serde_json::from_str(line) {
        Ok(request) => request,
        Err(e) => {
            return DaemonResponse {
                id: 0,
                highlights: Vec::new(),
                error: Some(format!("Invalid JSON request: {}", e)),
            };
        }
    };
    if !LANGUAGES.contains(&request.language.as_str()) {
        return DaemonResponse {
            id: request.id,
            highlights: Vec::new(),
            error: Some(format!("Unsupported language: {}", request.language)),
        };
    }
    let language_enum = map_language_to_enum(&request.language);
    set_parser_language(&request.language, parser, language_enum);
    let code = if request.language == "markdown" && !request.code.ends_with('\n') {
        format!("{}\n", request.code)
    } else {
        request.code
    };
    let Some(tree) = parser.parse(&code, None) else {
        return DaemonResponse {
            id: request.id,
            highlights: Vec::new(),
            error: Some("Failed to parse code".to_string()),
        };
    };
    let highlights = match crate::languages::query_highlights(
        parser,
        &request.highlights,
        &tree,
        &code,
        query_cursor,
        query_cache,
    ) {
        Ok(highlights) => highlights
            .into_iter()
            .map(|highlight| DaemonHighlight {
                capture: highlight.capture,
                start_byte: highlight.start_byte,
                end_byte: highlight.end_byte,
            })
            .collect(),
        Err(error) => {
            return DaemonResponse {
                id: request.id,
                highlights: Vec::new(),
                error: Some(error),
            };
        }
    };
    DaemonResponse {
        id: request.id,
        highlights,
        error: None,
    }
}

fn main() {
    let args = get_command().get_matches();
    handle_args(args, io::stdout());
}
