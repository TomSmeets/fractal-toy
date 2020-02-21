#!/bin/sh
set -euf -o pipefail

dir=git-remote-rust

rclone sync \
  --progress \
  --create-empty-src-dirs \
  "gdrive-crypt:/$dir" \
  "/tmp/gdrive/$dir"
