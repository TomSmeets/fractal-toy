#!/usr/bin/env bash
set -euf -o pipefail

function doit() {
    docker build -t "rust-build-$1" "./$1"
}

doit common
doit linux
doit web
doit win64
