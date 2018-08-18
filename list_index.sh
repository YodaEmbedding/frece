#!/bin/bash

# Example usage:
#   ./list_index.sh ~/.dir_index.txt ~/.dir_frequent.txt

path_index="$1"
path_freq="$2"
args="$3"
start_time=$(date +%s)

# Create files if they don't exist
touch -a "$path_freq"
touch -a "$path_index"

freq=$(cat "$path_freq")

IFS=$'\n' read -rd '' -a freq_counts <<<$(echo "$freq" | cut -d$'\t' -f1)
IFS=$'\n' read -rd '' -a freq_times  <<<$(echo "$freq" | cut -d$'\t' -f2)
IFS=$'\n' read -rd '' -a freq_items  <<<$(echo "$freq" | cut -d$'\t' -f3)

# Uses modified version of z's algorithm https://github.com/rupa/z/wiki/frecency
function frecency() {
    freq_count="$1"
    freq_time="$2"

    dt=$((start_time - freq_time))

    if ((dt == 0)); then
        echo "scale=3; $freq_count * 2" | bc -l
    else
        # 2^(-log10(dt / 6))
        #    = exp(-ln(2) * ln(dt / 6) / ln(10))
        #    = exp(-0.3 * ln(dt) - 0.54)
        echo "scale=3; $freq_count * e(-0.3 * l($dt) - 0.54)" | bc -l
    fi
}

function get_ranks() {
    for ((i = 0; i < ${#freq_items[@]}; i++))
    do
        rank=$(frecency "${freq_counts[i]}" "${freq_times[i]}")
        echo "$rank	${freq_items[i]}"
    done
}

if [ "$args" == "list" ]; then
    get_ranks | sort -nr
    exit 1
fi

get_ranks | sort -nr | cut -d$'\t' -f2
cat "$path_index"
