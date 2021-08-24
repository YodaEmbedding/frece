#!/bin/bash

DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/frece"
DB_FILE="$DATA_DIR/emoji.db"
ENTRIES_FILE="/tmp/frece_emoji_entries.txt"
CUSTOM_ENTRIES_FILE="$DATA_DIR/emoji_custom.txt"
URL="http://www.unicode.org/Public/emoji/11.0/emoji-test.txt"

curl -s "$URL" | \
    sed 's/^[^#]*; fully-qualified *# \([^ ]*\)/\1 \t/gp;d' > "$ENTRIES_FILE"

[ ! -d "$DATA_DIR" ] && mkdir -p "$DATA_DIR"

[ -f "$CUSTOM_ENTRIES_FILE" ] && cat "$CUSTOM_ENTRIES_FILE" >> "$ENTRIES_FILE"

if [ ! -f "$DB_FILE" ]; then
    frece init "$DB_FILE" "$ENTRIES_FILE"
else
    frece update "$DB_FILE" "$ENTRIES_FILE" --purge-old
fi
