[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://travis-ci.com/SicariusNoctis/frece.svg?branch=master)](https://travis-ci.com/SicariusNoctis/frece)

Maintain a database sorted by [frecency](https://en.wikipedia.org/wiki/Frecency) (frequency + recency).

- [Installation](#installation)
- [Usage](#usage)
  * [Commands](#commands)
  * [Examples](#examples)
    + [General](#general)
    + [Rofi](#rofi)

## Installation

Simply [download the latest release](https://github.com/SicariusNoctis/frece/releases) and add the `frece` executable to `PATH`.

Arch Linux users may install from the AUR packages [`frece`](https://aur.archlinux.org/packages/frece) or [`frece-git`](https://aur.archlinux.org/packages/frece-git).

## Usage

### Commands

`frece` provides the following subcommands:

```
increment    Increases an entry's count and resets its timer
init         Creates a database file from given list of entries
print        Prints list of frecency sorted entries
update       Updates a database file from given list of entries
```

See `frece --help` or the [`Examples`](#examples) section for more information.

### Examples

#### General

Begin by creating a database:

```bash
# Create list of entries
$ echo "apple
banana
cherry" > fruits.txt

# Initialize a database using list of items
$ frece init "fruits.db" "fruits.txt"
```

Access entries and print out a frecency sorted list of entries:

```bash
# Access an entry
$ frece increment "fruits.db" "cherry"

# Print out frecency sorted list
$ frece print "fruits.db"

cherry
apple
banana
```

Furthermore, a database can be updated with *new* entries:

```bash
# Create another list
$ echo "apple
cherry
elderberry
grapefruit" > fruits.txt

# Update database with new list
# Optionally, purge old entries like "banana"
$ frece update "fruits.db" "fruits.txt" --purge-old

# Print out frecency sorted list (verbosely)
$ frece print "fruits.db" --verbose

frecency   count  last access date           item
0.128476       1  2019-05-17T00:49:14+00:00  cherry
0.000000       0  1970-01-01T00:00:00+00:00  apple
0.000000       0  1970-01-01T00:00:00+00:00  elderberry
0.000000       0  1970-01-01T00:00:00+00:00  grapefruit
```

#### Rofi

The following examples may be found in the [`examples`](examples) directory:

```bash
examples/dir        Open a directory
examples/emoji      Copy an emoji to clipboard
examples/file       Open a file
```

For instance, `examples/dir` may be used as follows:

1. Initialize/update a database in `$HOME/.frece_dir.db`, providing a list of paths to directories to index:

    ```bash
    SEARCH_PATHS=("$HOME" "/some/other/path")
    ./examples/dir_update.sh "${SEARCH_PATHS[@]}"
    ```

   Tip: run this periodically via a systemd timer or cronjob to refresh the database.

2. Open with [rofi](https://github.com/davatorium/rofi), optionally providing a list of arguments:

    ```bash
    ROFI_ARGS=(-p 'folders' -i -levenshtein-sort)
    ./examples/dir_rofi.sh "${ROFI_ARGS[@]}"
    ```

    <!-- TODO verify above -->

    This will open up a rofi menu with entries sorted by frecency:

    ![](https://i.imgur.com/ylkVqBg.jpg)

Similarly, other examples are offered, including a rofi script to copy emojis to clipboard:

![](https://i.imgur.com/1PAaIGm.jpg)
