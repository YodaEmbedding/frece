#!/bin/bash

DIR=$(dirname "$0")"/.."
FRECE="frece"
DB_FILE="$HOME/.frece_emoji.db"

"$FRECE" print "$DB_FILE" | \
    rofi "$@" -dmenu | \
    (read x; [[ -z $x ]] || (
        emoji=$(cut -d$'\t' -f1 <<<"$x" | sed -e 's/[[:space:]]*$//')
        echo -n "$emoji" | xclip -selection clipboard
        "$FRECE" increment "$DB_FILE" "$x"))
