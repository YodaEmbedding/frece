#!/bin/bash

root_dir=$(dirname "$0")"/.."

"$root_dir/list_index.sh" ~/.dir_index.txt ~/.dir_frequent.txt | \
    rofi "$@" -dmenu | \
    (read x; [[ -z $x ]] || (
        gio open "$x"
        "$root_dir/update_freq.sh" ~/.dir_frequent.txt "$x"
        "$root_dir/update_index.sh" ~/.dir_index.txt ~/.dir_frequent.txt "$x"))
