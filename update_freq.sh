#!/bin/bash

# Updates given frequency list with given item
# Usage: update_freq "$HOME/.dir_frequent.txt" "$HOME/Desktop"
# Usage: update_freq "$HOME/.emoji_frequent.txt" ":)"

path="$1"
input_item="$2"

touch -a "$path"
freq=$(cat "$path")
start_time=$(date +%s)

# Increment frequency count for given item
is_found=false
while read -r line; do
    IFS=$'\t' read -ra items <<< "$line"
    freq_count="${items[0]}"
    freq_time="${items[1]}"
    freq_item=$(cut -d$'\t' -f3- <<< "$line")

    if [ "$freq_item" == "$input_item" ]; then
        let "freq_count=freq_count+1"
        line="$freq_count	$start_time	$freq_item"
        is_found=true
    fi

    echo "$line" >> "$path.1"
done <<< "$freq"

# Create entry for given item if necessary
if ! "$is_found"; then
    echo "1	$start_time	$input_item" >> "$path.1"
fi

# Sort and update
sort -rno "$path.1" "$path.1"
mv "$path.1" "$path"

# TODO Scale down when count becomes too large
