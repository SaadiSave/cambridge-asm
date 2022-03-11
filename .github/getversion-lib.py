#!/bin/env python3

from os import popen

version = popen("git describe --tags --abbrev=0").read()

assert "lib" in version

version = version.split("-")[1]

print(version, end="")
