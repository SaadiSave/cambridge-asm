# cambridge-asm-cli

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/saadisave/cambridge-asm/Publish%20CLI?style=for-the-badge)](https://github.com/SaadiSave/cambridge-asm/actions/workflows/rust.yml) [![Crates.io](https://img.shields.io/crates/v/cambridge-asm?style=for-the-badge)](https://crates.io/crates/cambridge-asm)

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
casm-run 
Run compiled or plaintext pseudoassembly

USAGE:
    casm run [OPTIONS] <PATH>

ARGS:
    <PATH>    Path to the input file containing compiled or plaintext pseudoassembly

OPTIONS:
    -f, --format <FORMAT>    Format of input file [default: pasm] [possible values: pasm, json, ron,
                             yaml, bin]
    -h, --help               Print help information
    -t, --bench              Show execution time
    -v, --verbose            Increase logging level
```

### `casm help compile`

```text
casm-compile 
Compile pseudoassembly

USAGE:
    casm compile [OPTIONS] <INPUT>

ARGS:
    <INPUT>    Path to the input file containing pseudoassembly

OPTIONS:
    -f, --format <FORMAT>    Format of output file [default: json] [possible values: json, ron,
                             yaml, bin]
    -h, --help               Print help information
    -m, --minify             Minify output
    -o, --output <OUTPUT>    Path to output file
    -v, --verbose            Increase logging level
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
