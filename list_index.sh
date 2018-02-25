#!/bin/bash

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

	min=60
	hour=3600
	day=86400
	week=604800

	dt=$((start_time - freq_time))

	if   ((dt < min));  then echo $((64 * freq_count))
	elif ((dt < hour)); then echo $((4 * freq_count))
	elif ((dt < day));  then echo $((2 * freq_count))
	elif ((dt < week)); then echo $((freq_count / 2))
	else                     echo $((freq_count / 4))
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
	get_ranks
else
	get_ranks | sort -nr | cut -d$'\t' -f2
	cat "$path_index"
fi

