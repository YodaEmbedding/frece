#!/bin/bash

DB_FILE="$HOME/.frece_file.db"

frece print "$DB_FILE" | \
    rofi "$@" -dmenu | \
    (read x; [[ -z $x ]] || (
        gio open "$x"
        frece increment "$DB_FILE" "$x"))
