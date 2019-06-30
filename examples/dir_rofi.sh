#!/bin/bash

DB_FILE="$HOME/.frece_dir.db"

item=$(frece print "$DB_FILE" | rofi "$@" -dmenu)
[[ -z $item ]] && exit
frece increment "$DB_FILE" "$item"

gio open "$item"
