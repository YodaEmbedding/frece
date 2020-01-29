extern crate chrono;
extern crate clap;
extern crate failure;
extern crate fs2;

mod args;
mod db;
mod field;

use chrono::{prelude::*, DateTime, NaiveDateTime};
use std::io::{prelude::*, stdout, BufWriter};
use std::path::Path;

use self::args::get_matches;
use self::db::*;
use self::field::{Field, FieldSlice};

type Result<T> = std::result::Result<T, failure::Error>;

fn main() -> Result<()> {
    let native_dt_epoch = NaiveDateTime::from_timestamp(0, 0);
    let epoch = DateTime::<Utc>::from_utc(native_dt_epoch, Utc);
    let now = Utc::now();
    let matches = get_matches();

    if let Some(matches) = matches.subcommand_matches("init") {
        let db_filename = matches.value_of("DB_FILE").unwrap();
        let raw_filename = matches.value_of("ENTRY_FILE").unwrap();
        init_db(raw_filename, db_filename, epoch)?;
    }

    if let Some(matches) = matches.subcommand_matches("add") {
        let db_filename = matches.value_of("DB_FILE").unwrap();
        let entry = matches.value_of("ENTRY").unwrap();
        let (fields, _lines) = read_db(db_filename)?;
        add_db(&fields, db_filename, entry, epoch)?;
    }

    if let Some(matches) = matches.subcommand_matches("increment") {
        let db_filename = matches.value_of("DB_FILE").unwrap();
        let entry = matches.value_of("ENTRY").unwrap();
        let (fields, lines) = read_db(db_filename)?;
        setfield_db(&fields, &lines, db_filename, entry, |x| {
            Field::new(x.count + 1, now, &x.data)
        })?;
    }

    if let Some(matches) = matches.subcommand_matches("set") {
        let db_filename = matches.value_of("DB_FILE").unwrap();
        let entry = matches.value_of("ENTRY").unwrap();
        let parse_count = str::parse::<i64>;
        let count = matches.value_of("count").map(parse_count).transpose()?;
        let time = matches.value_of("time").map(parse_time).transpose()?;
        let (fields, lines) = read_db(db_filename)?;
        setfield_db(&fields, &lines, db_filename, entry, |x| {
            Field::new(
                count.unwrap_or(x.count),
                time.unwrap_or(x.time),
                &x.data,
            )
        })?;
    }

    if let Some(matches) = matches.subcommand_matches("print") {
        let db_filename = matches.value_of("DB_FILE").unwrap();
        let sort_method = matches.value_of("sort").unwrap();
        let verbose = matches.is_present("verbose");
        let (mut fields, _lines) = read_db(db_filename)?;

        match sort_method {
            "alphabetical" => fields.sort_by_data(),
            "frecency" => fields.sort_by_frecency(now),
            "frequency" => fields.sort_by_frequency(),
            "recency" => fields.sort_by_recency(),
            "none" => {}
            _ => return Err(failure::err_msg("Unrecognized sort method")),
        }

        let stdout = stdout();
        let lock = stdout.lock();
        let mut w = BufWriter::new(lock);
        let to_str = |field: Field| {
            if verbose {
                field.to_info_str(now)
            } else {
                field.data
            }
        };

        for field in fields {
            writeln!(w, "{}", to_str(field))?;
        }
    }

    if let Some(matches) = matches.subcommand_matches("update") {
        let db_filename = matches.value_of("DB_FILE").unwrap();
        let raw_filename = matches.value_of("ENTRY_FILE").unwrap();
        let purge_old = matches.is_present("purge-old");

        if !Path::new(db_filename).exists() {
            init_db(raw_filename, db_filename, epoch)?;
        } else {
            let (fields, _lines) = read_db(db_filename)?;
            update_db(&fields, raw_filename, db_filename, epoch, purge_old)?;
        }
    }

    Ok(())
}

// TODO Unit tests
// TODO --recency-weight and --frequency-weight
// TODO --null-delimiter (avoids errors with Linux file paths...)
// TODO Options for escaping \n and \\? Or perhaps store string length headers.
// TODO Exception on duplicate entries in raw_lines. (Or silently filter.)
// TODO CPU: Vectorized frecency computations.
// TODO IO: Fast write to stdout/pipe.
// https://codereview.stackexchange.com/questions/73753/how-can-i-find-out-why-this-rust-program-to-echo-lines-from-stdin-is-slow
// https://www.reddit.com/r/rust/comments/5puyx2/why_is_println_so_slow/dcu2lf0
// https://www.reddit.com/r/rust/comments/94roxv/how_to_print_files_to_stdout_fast
// https://www.reddit.com/r/rust/comments/ab9b3z/locking_stdout_once_and_writing_to_it_versus
// fastcat https://matthias-endler.de/2018/fastcat/
