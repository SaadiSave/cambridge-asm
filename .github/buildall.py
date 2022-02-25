#!/bin/env python3

from os import system, listdir

system("mkdir release")

arches = [
    ("x86_64-unknown-linux-gnu", "x64-linux"),
    ("aarch64-unknown-linux-gnu", "arm64-linux"),
    ("riscv64gc-unknown-linux-gnu", "riscv64-linux"),
    ("x86_64-pc-windows-gnu", "x64-windows"),
]


def extension(triple: str) -> str:
    return ".exe" if ("windows" in triple) else ""


for triple, _ in arches:
    system(f"cargo build --profile prod --target {triple}")

for triple, user_arch in arches:
    system(
        f"zip release/cambridge-asm-{user_arch} target/{triple}/prod/casm{extension(triple)}"
    )

for triple, _ in arches:
    system(f"cargo build --features=cambridge --profile prod --target {triple}")

for triple, user_arch in arches:
    system(
        f"zip release/cambridge-asm-{user_arch}-caie target/{triple}/prod/casm{extension(triple)}"
    )

for name in listdir("./release"):
    if name.endswith(".zip"):
        system(f"sha256sum release/{name} > release/{name}.sha256sum")
