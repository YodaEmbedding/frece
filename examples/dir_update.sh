#!/bin/bash

DB_FILE="$HOME/.frece/db/dir.db"
ENTRIES_FILE="/tmp/frece_dir_entries.txt"

find "$@" -path '*/\.*' -prune -o -not -name '.*' -type d -print | \
    sort > "$ENTRIES_FILE"

if [ ! -f "$DB_FILE" ]; then
    frece init "$DB_FILE" "$ENTRIES_FILE"
else
    frece update "$DB_FILE" "$ENTRIES_FILE" --purge-old
fi
