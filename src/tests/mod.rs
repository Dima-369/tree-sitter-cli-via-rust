use super::*;

pub mod test_graphviz;
pub mod test_languages;

pub fn run_test_with_highlights<S: AsRef<str>>(
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
