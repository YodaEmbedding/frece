#!/bin/bash

root_dir=$(dirname "$0")"/.."
path_index="$HOME/.dir_index.txt"
path_freq="$HOME/.dir_frequent.txt"

find "$HOME/Downloads" "/mnt/data" -type d | sort | sed '/\/\.[^\.]*/d' > "$path_index.1"

"$root_dir/refresh_index.sh" "$path_index.1" "$path_freq"
mv "$path_index.1" "$path_index"
