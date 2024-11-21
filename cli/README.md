# cambridge-asm-cli

[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/SaadiSave/cambridge-asm/publish-cli.yml?style=for-the-badge)](https://github.com/SaadiSave/cambridge-asm/actions/workflows/publish-cli.yml)
[![Crates.io](https://img.shields.io/crates/v/cambridge-asm?style=for-the-badge)](https://crates.io/crates/cambridge-asm)

Command line interface to execute pseudoassembly programs

## Usage

### `casm -h`

```text
Cambridge Pseudoassembly Interpreter x.y.z
Saadi Save <github.com/SaadiSave>
Run pseudoassembly from Cambridge International syllabus 9618 (2021)

USAGE:
    casm <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    compile    Compile pseudoassembly
    help       Print this message or the help of the given subcommand(s)
    run        Run compiled or plaintext pseudoassembly
```

### `casm help run`

```text
Run compiled or plaintext pseudoassembly

Usage: casm run [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to the input file containing compiled or plaintext pseudoassembly

Options:
  -v, --verbose...       Increase logging level
  -t, --bench            Show execution time
  -f, --format <FORMAT>  Format of input file [default: pasm] [possible values: pasm, json, ron, yaml, cbor]
  -h, --help             Print help
```

### `casm help compile`

```text
Compile pseudoassembly

Usage: casm compile [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Path to the input file containing pseudoassembly

Options:
  -o, --output <OUTPUT>  Path to output file
  -v, --verbose...       Increase logging level
  -f, --format <FORMAT>  Format of output file [default: json] [possible values: json, ron, yaml, cbor]
  -m, --minify           Minify output
  -d, --debug            Include debuginfo
  -h, --help             Print help
```

## Log levels

* `OFF` by default
* `-v` = `WARN`: Enable warnings
* `-vv` = `INFO`: Enable info logs
* `-vvv` = `DEBUG`: Enable debugging logs
* `-vvvv` = `TRACE`: Trace execution line by line

### `WARN`

Arithmetic overflows are logged as warnings.

### `INFO`

General status is logged as info.

### `DEBUG`

Steps in the parsing procedure and internal structs created are shown in debug logs

### `TRACE`

Line-by-line execution is logged
