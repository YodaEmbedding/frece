#!/bin/bash

path_index="$1"
path_freq="$2"

# Create files if they don't exist
touch -a "$path_freq"
touch -a "$path_index"

cat "$path_freq" | sort -nr | sed 's/^\([0-9]*\)\t\([0-9]*\)\t\(.*\)/\3/'
cat "$path_index"

