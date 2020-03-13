#!/usr/bin/env bash
set -euf -o pipefail

cd "$(dirname "$0")"

function doit() {
    echo "Building: $1"
    docker build -t "rust-build-$1" "./$1"
    echo
    echo
}

doit common
doit linux
doit web
doit win64
