#!/bin/bash

path_index="$1"
path_freq="$2"

cat "$path_freq" | sort -nr | sed 's/^\([0-9]*\)\t\(.*\)/\2/'
cat "$path_index"

