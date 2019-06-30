#!/bin/bash

DB_FILE="$HOME/.frece_file.db"

item=$(frece print "$DB_FILE" | rofi "$@" -dmenu)
[[ -z $item ]] && exit
frece increment "$DB_FILE" "$item"

xdg-open "$item"
