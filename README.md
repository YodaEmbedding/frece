Maintain a index sorted by frequency. These scripts were designed for usage with rofi.

## Usage

Initially construct the index. For example, the following constructs a directory index:

    update_dir_index.sh

Now we may use the index with rofi as follows:

    ./list_index.sh ~/.dir_index.txt ~/.dir_frequent.txt | \
        rofi -dmenu | \
        (read x; [[ -z $x ]] || (
            gio open "$x"
            ./update_freq.sh ~/.dir_frequent.txt "$x"
            ./update_index.sh ~/.dir_index.txt ~/.dir_frequent.txt "$x"))

## Upcoming features

 - Frecency (frequency + recent)

