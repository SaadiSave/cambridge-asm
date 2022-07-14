#!/bin/env python3

from os import system, listdir

system("mkdir release")

targets = [
    ("x86_64-unknown-linux-gnu", "x64-linux"),
    ("aarch64-unknown-linux-gnu", "arm64-linux"),
    ("riscv64gc-unknown-linux-gnu", "riscv64-linux"),
    ("x86_64-pc-windows-gnu", "x64-windows"),
]


def extension(target: str) -> str:
    return ".exe" if ("windows" in target) else ""


def build():
    for triple, _ in targets:
        system(f"cargo build --profile prod --target {triple}")

    for triple, arch in targets:
        system(
            f"zip -j release/cambridge-asm-{arch} target/{triple}/prod/casm{extension(triple)} README.md LICENSE"
        )


def build_caie():
    for triple, _ in targets:
        system(f"cargo build --no-default-features --profile prod --target {triple}")

    for triple, arch in targets:
        system(
            f"zip -j release/cambridge-asm-{arch}-caie target/{triple}/prod/casm{extension(triple)} README.md LICENSE"
        )


def checksum():
    for name in listdir("./release"):
        if name.endswith(".zip"):
            system(f"sha256sum release/{name} > release/{name}.sha256sum")


if __name__ == "__main__":
    build()
    build_caie()
    checksum()
