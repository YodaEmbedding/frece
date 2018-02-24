#!/bin/bash

path_index="$HOME/.dir_index.txt"
path_freq="$HOME/.dir_frequent.txt"

# Acquire lists of directories
dirs=$(find "$HOME/Downloads" "/mnt/data" -type d | sort | sed '/\/\.[^\.]*/d')
freq_dirs=$(cat "$path_freq" | sort -r | sed 's/^\([0-9]*\)\t\(.*\)/\2/')

# Checks if given directory is within list of frequent directories
function is_freq() {
	dir="$1"
	while read -r freq_dir; do
		if [ "$dir" == "$freq_dir" ]; then
			return
		fi
	done <<< "$freq_dirs"
	false
}

# Write most frequent directories
while read -r dir; do
	echo "$dir" >> "$path_index.1"
done <<< "$freq_dirs"

# Write other directories
while read -r dir; do
	if ! is_freq "$dir"; then echo "$dir" >> "$path_index.1"; fi
done <<< "$dirs"

mv "$path_index.1" "$path_index"

