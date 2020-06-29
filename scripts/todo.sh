#!/usr/bin/env bash
git ls-files \
| grep -v '\.png' \
| grep -v 'scripts/todo.sh' \
| while read f; do \
    git annotate -e "$f" \
    | awk -F $'\t' -v name="$f" '{
        OFS="\t";
        date=$3;
        $1=$2=$3=""
        print date,name,$0
    }';
  done \
| grep 'TODO' \
| sort \
| sed 's/).*TODO/\tTODO/' \
| column -t -s $'\t'
