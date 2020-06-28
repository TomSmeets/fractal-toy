#!/usr/bin/env bash
git ls-files | grep '.rs' | while read f; do git annotate -e "$f" | awk -F $'\t' -v name="$f" '{ date=$3 " " $4; $1=$2=$3=$4=$5=""; print date,name,$0 }'; done | grep 'TODO' | sort | column -t -s $'\t'
