#!/bin/bash

dir="$1"

# TODO hmm... maybe to prevent corruption, should do in-place sed replace?
# TODO Also consider sorting afterwards

freq=$(cat "$HOME/.dir_frequent.txt" | sort -r)
#freq_counts=$(echo "$freq" | sed 's/^\([0-9]*\)\t\(.*\)/\1/')
#freq_dirs=$(  echo "$freq" | sed 's/^\([0-9]*\)\t\(.*\)/\2/')

rm "$HOME/.dir_frequent.txt"

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

	echo "$line" >> "$HOME/.dir_frequent.txt"
done <<< "$freq"

if ! "$is_found"; then
	echo "1	$dir" >> "$HOME/.dir_frequent.txt"
fi

