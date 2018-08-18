#!/bin/bash

root_dir=$(dirname "$0")"/.."
path_index="$HOME/.emoji_index.txt"
path_freq="$HOME/.emoji_frequent.txt"
url="http://www.unicode.org/Public/emoji/5.0/emoji-test.txt"

curl -s "$url" | sed 's/^[^#]*; fully-qualified *# \([^ ]*\)/\1 \t/gp;d' > "$path_index.1"

"$root_dir/refresh_index.sh" "$path_index.1" "$path_freq"
mv "$path_index.1" "$path_index"
