#!/bin/bash

DIR=$(dirname "$0")"/.."
FRECE="frece"
DB_FILE="$HOME/.frece_dir.db"
ENTRIES_FILE="/tmp/frece_dir_entries.txt"
export RUST_BACKTRACE=full

find "$@" -path '*/\.*' -prune -o -not -name '.*' -type d -print | \
    sort > "$ENTRIES_FILE"

if [ ! -f "$DB_FILE" ]; then
    "$FRECE" init "$DB_FILE" "$ENTRIES_FILE"
    exit
fi

"$FRECE" update --purge-old "$DB_FILE" "$ENTRIES_FILE"
