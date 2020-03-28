#!/usr/bin/env bash
set -euf -o pipefail
find . -name 'target' -type d -exec du -shc '{}' \+ | sort -h
