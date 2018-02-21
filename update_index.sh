#!/bin/bash

# Disable globbing
set -f

# Acquire lists of directories
dirs=$(find "$HOME/Downloads" "/mnt/data" -type d | sort | sed '/\/\.[^\.]*/d')
freq_dirs=$(cat "$HOME/.dir_frequent.txt" | sort -r | sed 's/^\([0-9]*\)\t\(.*\)/\2/')

# Write most frequent directories
while read -r dir; do
	echo "$dir" >> "$HOME/.dir_index.txt.1"
done <<< "$freq_dirs"

function is_freq() {
	dir="$1"
	while read -r freq_dir; do
		if [ "$dir" == "$freq_dir" ]; then
			return
		fi
	done <<< "$freq_dirs"
	false
}

# Write other directories
while read -r dir; do
	if ! is_freq "$dir"; then echo "$dir" >> "$HOME/.dir_index.txt.1"; fi
done <<< "$dirs"

mv "$HOME/.dir_index.txt.1" "$HOME/.dir_index.txt"

