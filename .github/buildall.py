#!/bin/env python3

from os import system, listdir

arches = [
    ("x86_64-unknown-linux-gnu", "x64-linux"),
    ("aarch64-unknown-linux-gnu", "arm64-linux"),
    ("riscv64gc-unknown-linux-gnu", "riscv64-linux"),
    ("x86_64-pc-windows-gnu", "x64-windows")
]

for triple, _ in arches:
    system(f"cargo build --profile prod --target {triple}")

for triple, user_arch in arches:
    system(f"zip cambridge-asm-{user_arch} target/{triple}/prod/casm")

for triple, _ in arches:
    system(f"cargo build --features=cambridge --profile prod --target {triple}")

for triple, user_arch in arches:
    system(f"zip cambridge-asm-caie-{user_arch} target/{triple}/prod/casm")

for name in listdir('../.circleci'):
    if name.endswith(".zip"):
        system(f"sha256sum {name} > {name}.sha256sum")
