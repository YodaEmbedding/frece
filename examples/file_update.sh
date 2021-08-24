#!/bin/bash

DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/frece"
DB_FILE="$DATA_DIR/file.db"
ENTRIES_FILE="/tmp/frece_file_entries.txt"

find "$@" -path '*/\.*' -prune -o -not -name '.*' -type f -print | \
    sort > "$ENTRIES_FILE"

[ ! -d "$DATA_DIR" ] && mkdir -p "$DATA_DIR"

if [ ! -f "$DB_FILE" ]; then
    frece init "$DB_FILE" "$ENTRIES_FILE"
else
    frece update "$DB_FILE" "$ENTRIES_FILE" --purge-old
fi
