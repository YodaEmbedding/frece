#!/bin/bash

DIR=$(dirname "$0")"/.."
FRECE="frece"
DB_FILE="$HOME/.frece_dir.db"

"$FRECE" print "$DB_FILE" | \
    rofi "$@" -dmenu | \
    (read x; [[ -z $x ]] || (
        gio open "$x"
        "$FRECE" increment "$DB_FILE" "$x"))
