# cambridge-asm

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/saadisave/cambridge-asm/Build?label=Build&logo=github)](https://github.com/SaadiSave/cambridge-asm/actions/workflows/rust.yml) [![Crates.io version](https://img.shields.io/crates/v/cambridge-asm)](https://crates.io/crates/cambridge-asm) [![Crates.io version](https://img.shields.io/crates/v/cambridge-asm?label=lib.rs)](https://lib.rs/crates/cambridge-asm)

## **Disclaimer**

### **This software is not related to Cambridge International, Cambridge University, or any of their sister institutions**

## Purpose

This is an interpreter for the pseudoassembly defined in syllabus 9618 - Computer Science of Cambridge Assesment International Education.

## Usage

```text
Cambridge Pseudoassembly Interpreter 0.6.0
Saadi Save <github.com/SaadiSave>
Run pseudoassembly from Cambridge International syllabus 9618 (2021)

USAGE:
    cambridge-asm [FLAGS] <INPUT>

FLAGS:
    -h, --help       Prints help information
    -t               Enables output of execution time
    -V, --version    Prints version information
    -v               Sets the logging level

ARGS:
    <INPUT>    Sets the input file containing pseudoassembly
```

Flags may be combined, e.g. `-tv`, `-vvt`, etc.

## Logging levels

* `OFF` by default
* `-v` = `WARN`: Enable warnings
* `-vv` = `INFO`: Enable info logs
* `-vvv` = `DEBUG`: Enable debugging logs
* `-vvvv` = `TRACE`: Trace execution line by line

### `WARN`

Arithmetic overflows are logged as warnings. That may change in the future.

### `INFO`

General status is logged as info.

### `DEBUG`

Steps in the parsing procedure and internal structs created are shown in debug logs

### `TRACE`

Line-by-line execution is logged

## Example program

```pasm
LOOP: LDX 201
OUT
INC IX
LDD CNT
INC ACC
STO CNT
CMP #5
JPN LOOP
LDM #10 // Code for newline
OUT // Output newline
END // This program prints HELLO


CNT: 0
201 72 // H
202 69 // E
203 76 // L
204 76 // L
205 79 // O
```
