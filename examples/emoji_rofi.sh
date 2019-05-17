#!/bin/bash

DB_FILE="$HOME/.frece_emoji.db"

frece print "$DB_FILE" | \
    rofi "$@" -dmenu | \
    (read x; [[ -z $x ]] || (
        emoji=$(cut -d$'\t' -f1 <<<"$x" | sed -e 's/[[:space:]]*$//')
        echo -n "$emoji" | xclip -selection clipboard
        frece increment "$DB_FILE" "$x"))
