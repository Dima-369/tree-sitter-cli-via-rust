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

The IDs are stable.

```
digraph name {
node_1[label="source_file 0 72
fn main() {\n                let test = \"\\\"1\\\"\"; // comment\n ..."];
node_1 -> node_2[label=""];
node_2[label="function_item 0 72
fn main() {\n                let test = \"\\\"1\\\"\"; // comment\n ..."];
node_2 -> node_3[label=""];
node_3[label="fn 0 2
fn"];
node_2 -> node_4[label=""];
node_4[label="identifier 3 7
main"];
node_2 -> node_5[label=""];
node_5[label="parameters 7 9
()"];
node_5 -> node_6[label=""];
node_6[label="( 7 8
("];
node_5 -> node_7[label=""];
node_7[label=") 8 9
)"];
node_2 -> node_8[label=""];
node_8[label="block 10 72
{\n                let test = \"\\\"1\\\"\"; // comment\n           ..."];
node_8 -> node_9[label=""];
node_9[label="{ 10 11
{"];
node_8 -> node_10[label=""];
node_10[label="let_declaration 28 47
let test = \"\\\"1\\\"\";"];
node_10 -> node_11[label=""];
node_11[label="let 28 31
let"];
node_10 -> node_12[label=""];
node_12[label="identifier 32 36
test"];
node_10 -> node_13[label=""];
node_13[label="= 37 38
="];
node_10 -> node_14[label=""];
node_14[label="string_literal 39 46
\"\\\"1\\\"\""];
node_14 -> node_15[label=""];
node_15[label="\" 39 40
\""];
node_14 -> node_16[label=""];
node_16[label="escape_sequence 40 42
\\\""];
node_14 -> node_17[label=""];
node_17[label="string_content 42 43
1"];
node_14 -> node_18[label=""];
node_18[label="escape_sequence 43 45
\\\""];
node_14 -> node_19[label=""];
node_19[label="\" 45 46
\""];
node_10 -> node_20[label=""];
node_20[label="; 46 47
;"];
node_8 -> node_21[label=""];
node_21[label="line_comment 48 58
// comment"];
node_21 -> node_22[label=""];
node_22[label="// 48 50
//"];
node_8 -> node_23[label=""];
node_23[label="} 71 72
}"];
}
```

See the generated graph here:
https://dreampuf.github.io/GraphvizOnline/?engine=dot#digraph%20name%20%7B%0Anode_1%5Blabel%3D%22source_file%200%2072%0Afn%20main()%20%7B%5Cn%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20let%20test%20%3D%20%5C%22%5C%5C%5C%221%5C%5C%5C%22%5C%22%3B%20%2F%2F%20comment%5Cn%20...%22%5D%3B%0Anode_1%20-%3E%20node_2%5Blabel%3D%22%22%5D%3B%0Anode_2%5Blabel%3D%22function_item%200%2072%0Afn%20main()%20%7B%5Cn%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20let%20test%20%3D%20%5C%22%5C%5C%5C%221%5C%5C%5C%22%5C%22%3B%20%2F%2F%20comment%5Cn%20...%22%5D%3B%0Anode_2%20-%3E%20node_3%5Blabel%3D%22%22%5D%3B%0Anode_3%5Blabel%3D%22fn%200%202%0Afn%22%5D%3B%0Anode_2%20-%3E%20node_4%5Blabel%3D%22%22%5D%3B%0Anode_4%5Blabel%3D%22identifier%203%207%0Amain%22%5D%3B%0Anode_2%20-%3E%20node_5%5Blabel%3D%22%22%5D%3B%0Anode_5%5Blabel%3D%22parameters%207%209%0A()%22%5D%3B%0Anode_5%20-%3E%20node_6%5Blabel%3D%22%22%5D%3B%0Anode_6%5Blabel%3D%22(%207%208%0A(%22%5D%3B%0Anode_5%20-%3E%20node_7%5Blabel%3D%22%22%5D%3B%0Anode_7%5Blabel%3D%22)%208%209%0A)
%22%5D%3B%0Anode_2%20-%3E%20node_8%5Blabel%3D%22%22%5D%3B%0Anode_8%5Blabel%3D%22block%2010%2072%0A%7B%5Cn%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20let%20test%20%3D%20%5C%22%5C%5C%5C%221%5C%5C%5C%22%5C%22%3B%20%2F%2F%20comment%5Cn%20%20%20%20%20%20%20%20%20%20%20...%22%5D%3B%0Anode_8%20-%3E%20node_9%5Blabel%3D%22%22%5D%3B%0Anode_9%5Blabel%3D%22%7B%2010%2011%0A%7B%22%5D%3B%0Anode_8%20-%3E%20node_10%5Blabel%3D%22%22%5D%3B%0Anode_10%5Blabel%3D%22let_declaration%2028%2047%0Alet%20test%20%3D%20%5C%22%5C%5C%5C%221%5C%5C%5C%22%5C%22%3B%22%5D%3B%0Anode_10%20-%3E%20node_11%5Blabel%3D%22%22%5D%3B%0Anode_11%5Blabel%3D%22let%2028%2031%0Alet%22%5D%3B%0Anode_10%20-%3E%20node_12%5Blabel%3D%22%22%5D%3B%0Anode_12%5Blabel%3D%22identifier%2032%2036%0Atest%22%5D%3B%0Anode_10%20-%3E%20node_13%5Blabel%3D%22%22%5D%3B%0Anode_13%5Blabel%3D%22%3D%2037%2038%0A%3D%22%5D%3B%0Anode_10%20-%3E%20node_14%5Blabel%3D%22%22%5D%3B%0Anode_14%5Blabel%3D%22string_literal%2039%2046%0A%5C%22%5C%5C%5C%221%5C%5C%5C%22%5C%22%22%5D%3B%0Anode_14%20-%3E%20node_15%5Blabel%3D%22%22%5D%3B%0Anode_15%5Blabel%3D%22%5C%22%2039%2040%0A%5C%22%22%5D%3B%0Anode_14%20-%3E%20node_16%5Blabel%3D%22%22%5D%3B%0Anode_16%5Blabel%3D%22escape_sequence%2040%2042%0A%5C%5C%5C%22%22%5D%3B%0Anode_14%20-%3E%20node_17%5Blabel%3D%22%22%5D%3B%0Anode_17%5Blabel%3D%22string_content%2042%2043%0A1%22%5D%3B%0Anode_14%20-%3E%20node_18%5Blabel%3D%22%22%5D%3B%0Anode_18%5Blabel%3D%22escape_sequence%2043%2045%0A%5C%5C%5C%22%22%5D%3B%0Anode_14%20-%3E%20node_19%5Blabel%3D%22%22%5D%3B%0Anode_19%5Blabel%3D%22%5C%22%2045%2046%0A%5C%22%22%5D%3B%0Anode_10%20-%3E%20node_20%5Blabel%3D%22%22%5D%3B%0Anode_20%5Blabel%3D%22%3B%2046%2047%0A%3B%22%5D%3B%0Anode_8%20-%3E%20node_21%5Blabel%3D%22%22%5D%3B%0Anode_21%5Blabel%3D%22line_comment%2048%2058%0A%2F%2F%20comment%22%5D%3B%0Anode_21%20-%3E%20node_22%5Blabel%3D%22%22%5D%3B%0Anode_22%5Blabel%3D%22%2F%2F%2048%2050%0A%2F%2F%22%5D%3B%0Anode_8%20-%3E%20node_23%5Blabel%3D%22%22%5D%3B%0Anode_23%5Blabel%3D%22%7D%2071%2072%0A%7D%22%5D%3B%0A%7D