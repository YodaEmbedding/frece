#!/bin/bash

# Disable globbing
set -f

# Acquire lists of directories
dirs=$(find "$HOME/Downloads" "/mnt/data" -type d | sort | sed '/\/\.[^\.]*/d')
freq_dirs=$(cat "$HOME/.dir_frequent.txt" | sort -r | sed 's/^\([0-9]*\)\t\(.*\)/\2/')

rm "$HOME/.dir_index.txt"

# Write most frequent directories
while read -r dir; do
	echo "$dir" >> "$HOME/.dir_index.txt"
done <<< "$freq_dirs"

# Write other directories
while read -r dir; do
	is_freq=false
	while read -r freq_dir; do
		if [ "$dir" == "$freq_dir" ]; then is_freq=true; break; fi
	done <<< "$freq_dirs"

	if [ "$is_freq" = false ]; then echo "$dir" >> "$HOME/.dir_index.txt"; fi
done <<< "$dirs"

