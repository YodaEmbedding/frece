#!/bin/bash

DIR=$(dirname "$0")"/.."
FRECE="frece"
DB_FILE="$HOME/.frece_emoji.db"
ENTRIES_FILE="/tmp/frece_emoji_entries.txt"
CUSTOM_ENTRIES_FILE="$HOME/.frece_emoji_custom.txt"
URL="http://www.unicode.org/Public/emoji/11.0/emoji-test.txt"
export RUST_BACKTRACE=full

curl -s "$URL" | \
    sed 's/^[^#]*; fully-qualified *# \([^ ]*\)/\1 \t/gp;d' > "$ENTRIES_FILE"

[ -f "$CUSTOM_ENTRIES_FILE" ] && cat "$CUSTOM_ENTRIES_FILE" >> "$ENTRIES_FILE"

if [ ! -f "$DB_FILE" ]; then
    "$FRECE" init "$DB_FILE" "$ENTRIES_FILE"
    exit
fi

"$FRECE" update --purge-old "$DB_FILE" "$ENTRIES_FILE"
