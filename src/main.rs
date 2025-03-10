mod graphviz;
mod languages;

use crate::graphviz::generate_dot_graph;
use crate::languages::{map_language_to_enum, process_query, set_parser_language, LANGUAGES};
use clap::{Arg, ArgAction, ArgMatches};
use std::io;
use std::io::Write;
use std::process::exit;
use tree_sitter::{Parser, StreamingIterator};

pub fn get_command() -> clap::Command {
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

pub fn handle_args<W>(args: ArgMatches, mut writer: W)
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

fn main() {
    let args = get_command().get_matches();
    handle_args(args, io::stdout());
}
