#!/bin/bash

path_index="$HOME/.dir_index.txt"
path_freq="$HOME/.dir_frequent.txt"

# Acquire lists of directories
dirs=$(find "$HOME/Downloads" "/mnt/data" -type d | sort | sed '/\/\.[^\.]*/d')
freq_dirs=$(cat "$path_freq" | sort -r | sed 's/^\([0-9]*\)\t\(.*\)/\2/')

echo "$freq_dirs" > "$path_index.1"
echo "$dirs" >> "$path_index.1"

# Remove duplicate entries
awk '!a[$0]++' "$path_index.1"

mv "$path_index.1" "$path_index"

