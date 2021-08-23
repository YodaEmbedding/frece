#!/bin/bash

DATA_DIR="$HOME/.config/frece"
DB_FILE="$DATA_DIR/emoji.db"

item=$(frece print "$DB_FILE" | rofi "$@" -dmenu)
[[ -z $item ]] && exit 1
frece increment "$DB_FILE" "$item"

emoji=$(cut -d$'\t' -f1 <<<"$item" | sed -e 's/[[:space:]]*$//')
echo -n "$emoji" | xclip -selection clipboard
