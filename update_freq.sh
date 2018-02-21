#!/bin/bash

dir="$1"

# TODO Also consider sorting afterwards

freq=$(cat "$HOME/.dir_frequent.txt" | sort -r)

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

	echo "$line" >> "$HOME/.dir_frequent.txt.1"
done <<< "$freq"

if ! "$is_found"; then
	echo "1	$dir" >> "$HOME/.dir_frequent.txt.1"
fi

sort -r -o "$HOME/.dir_frequent.txt.1" "$HOME/.dir_frequent.txt.1"

mv "$HOME/.dir_frequent.txt.1" "$HOME/.dir_frequent.txt"

