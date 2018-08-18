#!/bin/bash

# Example usage:
#   ./examples/refresh_dir_index.sh
#   ./examples/refresh_emoji_index.sh

path_index="$1"
path_freq="$2"

touch -a "$path_freq"

# Acquire lists of items
items=$(cat "$path_index")
freq_items=$(cat "$path_freq" | sort -nr | sed 's/^\([0-9]*\)\t\([0-9]*\)\t\(.*\)/\3/')

# Checks if given item is within list of frequent items
function is_freq() {
    item="$1"
    while read -r freq_item; do
        if [ "$item" == "$freq_item" ]; then
            return
        fi
    done <<< "$freq_items"
    false
}

# Write other items
while read -r item; do
    if ! is_freq "$item"; then echo "$item" >> "$path_index.1"; fi
done <<< "$items"

mv "$path_index.1" "$path_index"
