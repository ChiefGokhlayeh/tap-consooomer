# 📦 tap-rs

[![crate status: WIP](https://img.shields.io/badge/crate-WIP-blue)](https://github.com/ChiefGokhlayeh/tap-rs) [![Build and Test](https://github.com/ChiefGokhlayeh/tap-rs/actions/workflows/build_and_test.yaml/badge.svg)](https://github.com/ChiefGokhlayeh/tap-rs/actions/workflows/build_and_test.yaml) [![pre-commit.ci status](https://results.pre-commit.ci/badge/github/ChiefGokhlayeh/tap-rs/main.svg)](https://results.pre-commit.ci/latest/github/ChiefGokhlayeh/tap-rs/main) [![codecov](https://codecov.io/gh/ChiefGokhlayeh/tap-rs/branch/main/graph/badge.svg?token=0WTJX09WD8)](https://codecov.io/gh/ChiefGokhlayeh/tap-rs)

[Test Anything Protocol (TAP)](https://testanything.org/) Consumer for Rust. Capable of parsing [TAP14](https://testanything.org/tap-version-14-specification.html) files into [pest](https://github.com/pest-parser/pest) tokens.

## Usage

```txt
Reads a given Test Anything Protocol (TAP) file and prints the JSON-formatted parser result to
stdout. If FILE is omitted, TAP input is read from stdin. Parsing only comences after encountering
an EOF. Only complete TAP files are supported.

USAGE:
    tap [FILE]

ARGS:
    <FILE>
            Path to TAP input file

OPTIONS:
    -h, --help
            Print help information

    -V, --version
            Print version information
```

## Examples

See [examples](examples) directory for some example TAP logs. To convert them into JSON run:

```sh
❯ tap-rs examples/cascading.tap
```

The TAP log should be transformed as follows:

<div align="center">
<table>
<thead>
<tr>
<th>Input</th>
<th></th>
<th>Output</th>
</tr>
</thead>
<tbody>
<tr align="left">
<td>

```tap
TAP version 14
1..3 # root
ok 1 - i'm in root
# subtest: here begins sub-1
  2..2 # sub-1
  ok 2 - i'm in sub-1
ok 3
```

</td>
<td>
<font size="50pt">➜</font>
</td>
<td>

```json
{
  "preamble": {
    "version": "14"
  },
  "plan": {
    "first": 1,
    "last": 3,
    "reason": "root"
  },
  "body": [
    {
      "test": {
        "result": true,
        "number": 1,
        "description": "i'm in root",
        "directive": null,
        "yaml": []
      }
    },
    {
      "subtest": {
        "name": "here begins sub-1",
        "plan": {
          "first": 2,
          "last": 2,
          "reason": "sub-1"
        },
        "body": [
          {
            "test": {
              "result": true,
              "number": 2,
              "description": "i'm in sub-1",
              "directive": null,
              "yaml": []
            }
          }
        ]
      }
    },
    {
      "test": {
        "result": true,
        "number": 3,
        "description": null,
        "directive": null,
        "yaml": []
      }
    }
  ]
}
```

</td>
</tr>
</tbody>
</table>
</div>

## License

Licensed under

- Apache License, Version 2.0
  ([LICENSE](LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>)

## Limitations

- Embedded YAML blocks are parsed into a list of individual `yaml` lines. These are treated as plain-text and **not** broken down any further. Use any of your favorite [YAML libraries](https://crates.io/search?q=yaml) (like [serde_yaml](https://crates.io/crates/serde_yaml)) to further parse the embedded YAML block. Any indentation preceding the first element is used as the _anchor_ for the entire YAML block _and trimmed off_. Any line separators (`<LF>` or `<CR><LF>`) at the end of any given `yaml` line are omitted. Empty or whitespace-only lines inside the embedded YAML block get removed.
