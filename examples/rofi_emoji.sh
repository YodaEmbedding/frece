root_dir=$(dirname "$0")"/.."

"$root_dir/list_index.sh" ~/.emoji_index.txt ~/.emoji_frequent.txt | \
    rofi "$@" -dmenu | \
    (read x; [[ -z $x ]] || (
        emoji=$(cut -d$'\t' -f1 <<<"$x" | sed -e 's/[[:space:]]*$//')
        echo -n "$emoji" | xclip -selection clipboard
        "$root_dir/update_freq.sh" ~/.emoji_frequent.txt "$x"
        "$root_dir/update_index.sh" ~/.emoji_index.txt ~/.emoji_frequent.txt "$x"))
