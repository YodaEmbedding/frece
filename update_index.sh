#!/bin/bash

path_index="$1"
path_freq="$2"
input_item="$3"

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

if ! is_freq "$input_item"; then
	exit 1
fi

# Remove input_item from items
items=$(cat "$path_index")
while read -r item; do
	if [[ $item != $input_item ]]; then
		echo "$item" >> "$path_index.1"
	fi
done <<< "$items"

mv "$path_index.1" "$path_index"

