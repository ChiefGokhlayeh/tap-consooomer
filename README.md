# 📦 tap-rs

[![crate status: WIP](https://img.shields.io/badge/crate-WIP-blue)](https://github.com/ChiefGokhlayeh/tap-rs) [![pre-commit.ci status](https://results.pre-commit.ci/badge/github/ChiefGokhlayeh/tap-rs/main.svg)](https://results.pre-commit.ci/latest/github/ChiefGokhlayeh/tap-rs/main)

[Test Anything Protocol (TAP)](https://testanything.org/) Consumer for Rust. Capable of parsing [TAP14](https://testanything.org/tap-version-14-specification.html) files into [pest](https://github.com/pest-parser/pest) tokens.

## Usage

Use tap-rs to convert a TAP14 input file into JSON:

```sh
tap-rs <FILE>

ARGS:
    <FILE>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information
```

## Examples

See [examples](examples) directory for some example TAP logs. To conver them into JSON run:

```sh
tap-rs examples/common.log
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
<tr>
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
      "Test": {
        "result": true,
        "number": 1,
        "description": "i'm in root",
        "directive": null
      }
    },
    {
      "Subtest": {
        "name": "here begins sub-1",
        "plan": {
          "first": 2,
          "last": 2,
          "reason": "sub-1"
        },
        "body": [
          {
            "Test": {
              "result": true,
              "number": 2,
              "description": "i'm in sub-1",
              "directive": null
            }
          }
        ]
      }
    },
    {
      "Test": {
        "result": true,
        "number": 3,
        "description": null,
        "directive": null
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
