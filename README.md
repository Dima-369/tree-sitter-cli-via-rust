# Setup

Only tested on macOS. Requires `brew install tree-sitter`

Make sure to clone all git submodules for `Kotlin Emacs` since it requires the `highlights.scm` files for
highlighting code.

# CLI `--help` output

```
Outputs capture names with byte ranges or graphviz code using Tree-sitter for Kotlin Emacs.

Usage: tree-sitter-cli-via-rust [OPTIONS] --code <code> --language <language>

Options:
      --code <code>              The code to parse
      --language <language>      [possible values: kotlin, php, bash, json, dockerfile, python, java, rust]
      --highlights <highlights>  String of highlights like the content of queries/highlights.scm. This is required when not using --graphviz-only
      --graphviz-only            If passed, output only the graphviz dot graph
  -h, --help                     Print help
  -V, --version                  Print version
```

# Output

The format is one per line: `{captureName} {byteRangeStart} {byteRangeEnd}`

```
keyword 3 7
keyword 8 13
type 14 19
constructor 19 55
punctuation.bracket 19 20
keyword 25 28
property 29 30
variable 29 30
punctuation.delimiter 30 31
type 32 35
type.builtin 32 35
punctuation.delimiter 35 36
keyword 41 44
property 45 46
variable 45 46
punctuation.delimiter 46 47
type 48 51
type.builtin 48 51
punctuation.bracket 54 55
```

## `--graphviz-only`

Each node has the label: `{captureName} {byteStart} {byteEnd}\n{truncated content}`

Note that the IDs are randomly generated.

```
digraph name {
node_105553166401536[label="module 0 10
test = \"1\""];
node_105553166401536 -> node_105553128654720[label=""];
node_105553128654720[label="expression_statement 0 10
test = \"1\""];
node_105553128654720 -> node_105553128654624[label=""];
node_105553128654624[label="assignment 0 10
test = \"1\""];
node_105553128654624 -> node_105553128654144[label=""];
node_105553128654144[label="identifier 0 4
test"];
node_105553128654624 -> node_105553121312888[label=""];
node_105553121312888[label="= 5 6
="];
node_105553128654624 -> node_105553128654336[label=""];
node_105553128654336[label="string 7 10
\"1\""];
node_105553128654336 -> node_105553121312768[label=""];
node_105553121312768[label="string_start 7 8
\""];
node_105553128654336 -> node_105553121312776[label=""];
node_105553121312776[label="string_content 8 9
1"];
node_105553128654336 -> node_105553121312784[label=""];
node_105553121312784[label="string_end 9 10
\""];
}
```

See the generated graph here: https://is.gd/Xx7CKq