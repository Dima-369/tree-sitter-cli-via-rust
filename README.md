# Setup

Only tested on macOS. Requires `brew install tree-sitter`.

Make sure to clone all git submodules for `Kotlin Emacs` since it requires the `highlights.scm` files for
highlighting code.

# CLI `--help` output

```
Outputs capture names with byte range using Tree-sitter for Kotlin Emacs.

Usage: tree-sitter-cli-via-rust --code <code> --language <language> --highlights <highlights>

Options:
      --code <code>              The code to parse
      --language <language>      [possible values: kotlin, php, bash, json, dockerfile, python, java]
      --highlights <highlights>  String of highlights like the content of queries/highlights.scm
  -h, --help                     Print help
  -V, --version                  Print version
```

# Output

The format is one per line: `{captureName} {byteRangeStart} {byteRangeEnd}`.

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