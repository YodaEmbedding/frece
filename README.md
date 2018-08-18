Maintain a index sorted by [frecency](https://en.wikipedia.org/wiki/Frecency) (frequency + recency).
These scripts were designed for usage with [rofi](https://github.com/DaveDavenport/rofi).

## Usage

Initially, construct an index. For example, the following constructs a directory index:

    ./examples/refresh_dir_index.sh

Now we may use the index with rofi as follows:

    ./examples/rofi_dir.sh $ROFI_ARGS

where you may specify your own `$ROFI_ARGS`, such as `-hide-scrollbar`. This will open up a menu with entries sorted by frecency:

![](https://i.imgur.com/ylkVqBg.jpg)

Similarly, other examples are offered, including a rofi script to copy emojis to clipboard:

![](https://i.imgur.com/1PAaIGm.jpg)

## Features

To list the current frecency rankings:

    ./list_index.sh ~/.dir_index.txt ~/.dir_frequent.txt list

outputs:

    rank    directory
    .105    /mnt/data/Dropbox/sfu/Current
    .050    /mnt/data/Dropbox/sfu/Y4S2 2018Sp/ENSC 410
    .048    /mnt/data/Dropbox/DB Pictures/Digital Art/Landscapes
    .042    /mnt/data/Dropbox
    .039    /mnt/data/Dropbox/sfu/Y4S2 2018Sp/ENSC 495/Course Notes
    .030    /mnt/data/Dropbox/sfu/Y4S2 2018Sp/ENSC 495/Labs/Lab 1
    .030    /mnt/data/Dropbox/sfu/Y4S2 2018Sp/ENSC 495
    .030    /mnt/data/Dropbox/sfu/Y4S2 2018Sp/ENSC 327
    .030    /mnt/data/Downloads
    .022    /mnt/data/Dropbox/sfu/Y4S2 2018Sp/ENSC 350/Labs/lab4
    .022    /mnt/data/Dropbox/eBooks/0Current
    .021    /mnt/data/Dropbox/Camera Uploads/Screenshots
    .020    /mnt/data/Dropbox/DB Pictures/Wallpaper Unsorted

