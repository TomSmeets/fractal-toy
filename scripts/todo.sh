#!/usr/bin/env bash
git ls-files | grep '.rs' | while read f; do git annotate -e "$f" | awk -v name="$f" '{ date=$3 " " $4; $1=$2=$3=$4=""; print date "\t" name "\t" $0 }'; done | grep 'TODO' | sort
