#!/bin/bash

DATA_DIR="$HOME/.config/frece"
DB_FILE="$DATA_DIR/file.db"

item=$(frece print "$DB_FILE" | rofi "$@" -dmenu)
[[ -z $item ]] && exit 1
frece increment "$DB_FILE" "$item"

xdg-open "$item"
