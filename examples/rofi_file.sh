#!/bin/bash

DIR=$(dirname "$0")"/.."
FRECE="$DIR/target/release/frece"
DB_FILE="$HOME/.frece_file.db"

"$FRECE" print "$DB_FILE" | \
    rofi "$@" -dmenu | \
    (read x; [[ -z $x ]] || (
        gio open "$x"
        "$FRECE" increment "$DB_FILE" "$x"))
