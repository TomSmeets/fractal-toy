#!/usr/bin/env bash
set -euf -o pipefail

dir=git-remote-rust

if [ ! -d /tmp/gdrive/$dir ]; then
  echo "/tmp/gdrive/$dir does not exist!"
  exit 1
fi

rclone sync \
  --progress \
  --create-empty-src-dirs \
  "/tmp/gdrive/$dir" \
  "gdrive-crypt:/$dir"
