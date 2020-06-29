#!/usr/bin/env bash
# List all used features in the source
rg  -o 'cfg\( *feature *= *".*" *\)'  --no-filename ./. | sort | uniq -c
