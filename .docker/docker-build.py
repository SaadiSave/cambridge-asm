#!/bin/env python3

from os import environ, system

tag = environ.get("DOCKER_TAG")

if tag is None:
    print("Please set the docker tag")
    exit(1)

system(f"docker build -t saadisave/cambridge-asm-ci:{tag} .")

if environ.get("DOCKER_PUSH"):
    print("About to proceed with push to DockerHub")
    if input("Do you want to continue (y/n):").lower().strip() == 'y':
        system(f"docker push saadisave/cambridge-asm-ci:{tag}")
