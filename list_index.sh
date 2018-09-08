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
IFS=$'\n' read -rd '' -a freq_items  <<<$(echo "$freq" | cut -d$'\t' -f3-)

# Uses modified version of z's algorithm https://github.com/rupa/z/wiki/frecency
function frecency() {
    local freq_count="$1"
    local freq_time="$2"

    local dt=$((start_time - freq_time))

    if ((dt == 0)); then
        echo "scale=3; $freq_count * 2" | bc -l
    else
        # 2^(-log10(dt / 6))
        #    = exp(-ln(2) * ln(dt / 6) / ln(10))
        #    = exp(-0.3 * ln(dt) - 0.54)
        # Hmmm... the 0.54 isn't needed. But it can bring numerical stability?
        # Vary constants depending on size of maximum item?
        # Also, this is kind of like plain old $freq_count / $dt
        echo "scale=3; e(0.25 * l($freq_count) - 0.3 * l($dt) + 3.0)" | bc -l
    fi
}

function display_time() {
    local T=$1
    local D=$((T / 60 / 60 / 24))
    local H=$((T / 60 / 60 % 24))
    local M=$((T / 60 % 60))
    local S=$((T % 60))
    if   (($D > 0)); then printf '%dd' $D;
    elif (($H > 0)); then printf '%dh' $H;
    elif (($M > 0)); then printf '%dm' $M;
    else                  printf '%ds' $S;
    fi
}

function get_ranks() {
    for ((i = 0; i < ${#freq_items[@]}; i++))
    do
        local rank=$(frecency "${freq_counts[i]}" "${freq_times[i]}")
        echo "$rank	${freq_items[i]}"
    done
}

function get_list() {
    echo "score	count	last	item"
    for ((i = 0; i < ${#freq_items[@]}; i++))
    do
        local rank=$(frecency "${freq_counts[i]}" "${freq_times[i]}")
        local dt=$(display_time $((start_time - freq_times[i])))
        echo "$rank	${freq_counts[i]}	$dt	${freq_items[i]}"
    done
}

if [ "$args" == "list" ]; then
    get_list | sort -nr
    exit 1
fi

get_ranks | sort -nr | cut -d$'\t' -f2-
cat "$path_index"
