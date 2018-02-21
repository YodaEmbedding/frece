#!/bin/bash

# find $HOME/Downloads /mnt/data -type d | sort | sed '/\/\.[^\.]*/d' > ~/.dir_index.txt

dirs=$(find "$HOME/Downloads" "/mnt/data" -type d | sort | sed '/\/\.[^\.]*/d')
freq_dirs=$(cat "$HOME/.dir_frequent.txt" | sort -r | sed 's/^\([0-9]*\)\t\(.*\)/\2/')

IFS=$'\n'       # make newlines the only separator
set -f          # disable globbing

rm "$HOME/.dir_index.txt"

#for line in $(cat < "$HOME/.dir_frequent.txt"); do
#    freq_dir=$(echo "$line" | cut -f2 -d$'\t')
#    echo "$freq_dir" >> "$HOME/.dir_index.txt"
#done

while read -r dir; do
	echo "$dir" >> "$HOME/.dir_index.txt"
done <<< "$freq_dirs"

while read -r dir; do
	is_freq=false
	while read -r freq_dir; do
		#IFS=$'\t' read -ra items <<< "$line"
		#freq_dir="${items[1]}"
		if [ "$dir" == "$freq_dir" ]; then is_freq=true; break; fi
	done <<< "$freq_dirs"

	if [ "$is_freq" = false ]; then echo "$dir" >> "$HOME/.dir_index.txt"; fi
done <<< "$dirs"

