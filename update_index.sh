#!/bin/bash

# TODO Maybe the fastest way is to maintain two separate indexes?
# idk maybe... but try this first; just load everything into memory
# Actually just rerun update_index (without the reconstruction of index)

path_index="$HOME/.dir_index.txt"
path_freq="$HOME/.dir_frequent.txt"

# Acquire lists of items
items=$(cat "$path_index")
freq_items=$(cat "$path_freq" | sort -r | sed 's/^\([0-9]*\)\t\(.*\)/\2/')

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

# Write most frequent items
while read -r item; do
	echo "$item" >> "$path_index.1"
done <<< "$freq_items"

# Count number of frequent items
[ -z "$freq_items" ] && freq_len=0 || freq_len=$(echo "$freq_items" | wc -l)

# Write other items
n=0
while read -r item; do
	if ((n >= freq_len)) || ! is_freq "$item"; then
		echo "$item" >> "$path_index.1"
	else
		((n++))
	fi
done <<< "$items"

mv "$path_index.1" "$path_index"

