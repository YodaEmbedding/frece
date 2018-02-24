#!/bin/bash

# Updates frequency list with given directory
# Usage: update_freq "/home/user/Desktop"

dir="$1"
path="$HOME/.dir_frequent.txt"
path_tmp="$HOME/.dir_frequent.txt.1"
freq=$(cat "$path")

# Increment frequency count for given directory
is_found=false
while read -r line; do
	IFS=$'\t' read -ra items <<< "$line"
	freq_count="${items[0]}"
	freq_dir="${items[1]}"

	if [ "$freq_dir" == "$dir" ]; then
		let "freq_count=freq_count+1"
		line="$freq_count	$freq_dir"
		is_found=true
	fi

	echo "$line" >> "$path_tmp"
done <<< "$freq"

# Create entry for given directory if necessary
if ! "$is_found"; then
	echo "1	$dir" >> "$path_tmp"
fi

# Sort and update
sort -o "$path_tmp" "$path_tmp"
mv "$path_tmp" "$path"

# TODO Frecency

