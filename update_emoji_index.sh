#!/bin/bash

path_index="$HOME/.emoji_index.txt"
path_freq="$HOME/.emoji_frequent.txt"
url="http://www.unicode.org/Public/emoji/5.0/emoji-test.txt"

touch "$path_freq"

# Acquire lists of emojis
emojis=$(curl -s "$url" | sed 's/^[^#]*; fully-qualified *# \([^ ]*\)/\1 \t/gp;d')
freq_emojis=$(cat "$path_freq" | sort -r | sed 's/^\([0-9]*\)\t\(.*\)/\2/')

# Checks if given emoji is within list of frequent emojis
function is_freq() {
	emoji="$1"
	while read -r freq_emojis; do
		if [ "$emoji" == "$freq_emojis" ]; then
			return
		fi
	done <<< "$freq_emojis"
	false
}

# Write other emojis
while read -r emoji; do
	if ! is_freq "$emoji"; then echo "$emoji" >> "$path_index.1"; fi
done <<< "$emojis"

mv "$path_index.1" "$path_index"
