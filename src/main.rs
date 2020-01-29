extern crate chrono;
extern crate clap;
extern crate failure;
extern crate fs2;

use chrono::{prelude::*, DateTime, NaiveDateTime};
use clap::{App, Arg, SubCommand};
use fs2::FileExt;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{prelude::*, stdout, BufWriter, SeekFrom};
use std::iter;
use std::path::Path;

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Field {
    count: i64,
    time: DateTime<Utc>,
    data: String,
}

trait FieldSlice {
    fn sort_by_data(&mut self);
    fn sort_by_frecency(&mut self, dt: DateTime<Utc>);
    fn sort_by_frequency(&mut self);
    fn sort_by_recency(&mut self);
}

impl Field {
    pub fn new(count: i64, time: DateTime<Utc>, data: &str) -> Self {
        Self {
            count,
            time,
            data: data.to_owned(),
        }
    }

    pub fn frecency(&self, dt: &DateTime<Utc>) -> i64 {
        let secs = dt.signed_duration_since(self.time).num_seconds();
        (1e15 * frecency(self.count, secs)) as i64
    }

    pub fn to_info_str(&self, dt: DateTime<Utc>) -> String {
        let secs = dt.signed_duration_since(self.time).num_seconds();
        format!(
            "{:.6}  {:6}  {:25}  {}",
            frecency(self.count, secs),
            self.count,
            self.time.to_rfc3339_opts(SecondsFormat::Secs, false),
            self.data
        )
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:06},{},{}",
            self.count,
            self.time.to_rfc3339_opts(SecondsFormat::Micros, false),
            self.data
        )
    }
}

impl FieldSlice for [Field] {
    fn sort_by_data(&mut self) {
        // self.sort_by_cached_key(|x| x.data);
        self.sort_by(|x, y| x.data.cmp(&y.data));
    }

    fn sort_by_frecency(&mut self, dt: DateTime<Utc>) {
        self.sort_by_cached_key(|x| std::cmp::Reverse(x.frecency(&dt)));
    }

    fn sort_by_frequency(&mut self) {
        self.sort_by_cached_key(|x| std::cmp::Reverse(x.count));
    }

    fn sort_by_recency(&mut self) {
        self.sort_by_cached_key(|x| std::cmp::Reverse(x.time));
    }
}

/// Calculate frecency.
///
/// Linear combination of logarithmically weighted `counts` and `secs`.
/// Normalizes with a sigmoid to ensure frecency is constrained to the
/// interval [0, 1].
fn frecency(count: i64, secs: i64) -> f64 {
    if count == 0 {
        return 0.0;
    }

    let c = 0.75 * (1.0 + count as f64).ln();
    let s = -0.25 * (1.0 + secs as f64).ln();
    let x = c + s;

    1.0 / (1.0 + (-x).exp())
}

/// Update list of fields with new entries.
///
/// Returns list of fields with values specified by `raw_lines`, using
/// fields from `db_fields` if their values matche, and otherwise
/// creating new fields with time `dt`. The order of the list
/// corresponds to the order of values within `raw_lines`. If
/// `purge_old` is `false`, the entries from `db_fields` that are not in
/// `raw_lines` are appended to the list, in their original order.
fn update_fields(
    raw_lines: &[String],
    db_fields: &[Field],
    dt: DateTime<Utc>,
    purge_old: bool,
) -> Vec<Field> {
    let db_lookup = db_fields
        .iter()
        .enumerate()
        .map(|(i, x)| (&x.data, (i, x)))
        .collect::<HashMap<_, _>>();

    let new_fields = raw_lines.iter().map(|x| {
        db_lookup
            .get(x)
            .map(|&(_, field)| field.to_owned())
            .unwrap_or_else(|| Field::new(0, dt, x))
    });

    let old_fields: Box<dyn Iterator<Item = &Field>> = if purge_old {
        Box::new(iter::empty::<&Field>())
    } else {
        Box::new(get_old_fields(&raw_lines, &db_fields, &db_lookup))
    };
    let old_fields = old_fields.map(|x| x.to_owned());

    new_fields.chain(old_fields).collect()
}

/// Filter `db_fields` by `raw_lines`.
///
/// Returns new list containing fields in `db_fields` that do not match
/// any of the values in `raw_lines`. List is returned in the ordering
/// specified by `db_lookup`. Note that `db_lookup` must contain a valid
/// mapping from `&String` to all `Field`s within `db_fields` paired
/// with a uniquely ordered key for sorting.
fn get_old_fields<'a>(
    raw_lines: &[String],
    db_fields: &[Field],
    db_lookup: &HashMap<&String, (usize, &'a Field)>,
) -> impl Iterator<Item = &'a Field> {
    let old_set = {
        let raw_set = raw_lines.iter().collect::<HashSet<&String>>();

        let db_set = db_fields
            .iter()
            .map(|x| &x.data)
            .collect::<HashSet<&String>>();

        db_set
            .difference(&raw_set)
            .map(|&x| x)
            .collect::<HashSet<&String>>()
    };

    let mut old_fields = old_set
        .into_iter()
        .map(|x| db_lookup[x])
        .collect::<Vec<_>>();

    old_fields.sort_by_key(|&(i, _)| i);
    old_fields.into_iter().map(|(_, x)| x)
}

fn parse_line(line: &str) -> Result<Field> {
    let split = line.splitn(3, ',').collect::<Vec<&str>>();

    let [count_str, time_str, data] = match split[0..3] {
        [x, y, z] => [x, y, z],
        _ => return Err(failure::err_msg("Insufficient entries")),
    };

    let time = DateTime::parse_from_rfc3339(&time_str)?.with_timezone(&Utc);
    let count = count_str.parse::<i64>()?;

    Ok(Field::new(count, time, data))
}

/// Increment specified database entry.
///
/// Increments total access count and sets last access time of entry.
fn increment_db(
    fields: &[Field],
    lines: &[String],
    dt: DateTime<Utc>,
    db_filename: &str,
    entry: &str,
) -> Result<()> {
    let line = fields
        .iter()
        .position(|x| x.data == entry)
        .ok_or(failure::err_msg("Entry not found in database"))?;

    let field = Field::new(fields[line].count + 1, dt, &fields[line].data);
    let write_str = field.to_string();
    let lengths = lines.iter().map(|x| x.len());
    let seek_begin = lengths.take(line).sum::<usize>() + line;
    let seek_end = seek_begin + lines[line].len() + 1;
    assert!(write_str.len() == seek_end - seek_begin - 1);

    let mut db_file = OpenOptions::new().write(true).open(db_filename)?;

    db_file.lock_exclusive()?;
    db_file.seek(SeekFrom::Start(seek_begin as u64))?;
    db_file.write_all(write_str.as_bytes())?;
    db_file.unlock()?;

    Ok(())
}

/// Initializes database using given list of entries.
fn init_db(
    raw_filename: &str,
    db_filename: &str,
    dt: DateTime<Utc>,
) -> Result<()> {
    let raw_str = fs::read_to_string(raw_filename)?;
    let fields = raw_str.lines().map(|line| Field::new(0, dt, line));
    write_fields(fields, db_filename)?;
    Ok(())
}

/// Get list of all fields from database.
fn read_db(db_filename: &str) -> Result<(Vec<Field>, Vec<String>)> {
    let db_str = fs::read_to_string(db_filename)?;

    let lines = db_str
        .lines()
        .map(|x| x.to_owned())
        .collect::<Vec<String>>();

    let fields = lines
        .iter()
        .map(|x| parse_line(&x))
        .collect::<Result<Vec<Field>>>()?;

    Ok((fields, lines))
}

/// Update database with list of entries.
fn update_db(
    db_fields: &[Field],
    raw_filename: &str,
    db_filename: &str,
    dt: DateTime<Utc>,
    purge_old: bool,
) -> Result<()> {
    let raw_str = fs::read_to_string(raw_filename)?;
    let raw_lines = raw_str
        .lines()
        .map(|x| x.to_owned())
        .collect::<Vec<String>>();
    let fields = update_fields(&raw_lines, &db_fields, dt, purge_old);
    write_fields(fields.into_iter(), db_filename)?;
    Ok(())
}

/// Write fields to new database.
fn write_fields(
    fields: impl Iterator<Item = Field>,
    filename: &str,
) -> Result<()> {
    let tmp_filename = format!("{}{}", filename, ".tmp");
    let mut tmp_file = BufWriter::new(
        OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&tmp_filename)?,
    );

    for field in fields {
        writeln!(&mut tmp_file, "{}", field)?;
    }

    tmp_file.flush()?;
    drop(tmp_file);
    fs::rename(tmp_filename, filename)?;
    Ok(())
}

fn main() -> Result<()> {
    let native_dt_epoch = NaiveDateTime::from_timestamp(0, 0);
    let epoch = DateTime::<Utc>::from_utc(native_dt_epoch, Utc);
    let now = Utc::now();

    let matches = App::new("frece")
        .version("1.0.3")
        .author("Mateen Ulhaq <mulhaq2005@gmail.com>")
        .about("Frecency indexed database")
        .subcommand(
            SubCommand::with_name("increment")
                .about("Increases an entry's count and resets its timer")
                .arg(
                    Arg::with_name("DB_FILE")
                        .help("Path to frecency database file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("ENTRY")
                        .help("Entry to increment")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("init")
                .about("Creates a database file from given list of entries")
                .arg(
                    Arg::with_name("DB_FILE")
                        .help("Path to frecency database file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("ENTRY_FILE")
                        .help("Path to list of entries, separated by newlines")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("print")
                .about("Prints list of frecency sorted entries")
                .arg(
                    Arg::with_name("DB_FILE")
                        .help("Path to frecency database file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("sort")
                        .help("Sort method")
                        .long("sort")
                        .takes_value(true)
                        .default_value("frecency")
                        .possible_values(&[
                            "none",
                            "alphabetical",
                            "frecency",
                            "frequency",
                            "recency",
                        ]),
                )
                .arg(
                    Arg::with_name("verbose")
                        .help("Outputs frecency, counts, date, and entries")
                        .short("v")
                        .long("verbose"),
                ),
        )
        .subcommand(
            SubCommand::with_name("update")
                .about("Updates a database file from given list of entries")
                .arg(
                    Arg::with_name("DB_FILE")
                        .help("Path to frecency database file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("ENTRY_FILE")
                        .help("Path to list of entries, separated by newlines")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::with_name("purge-old")
                        .help("Purge any entries *not* in ENTRY_FILE")
                        .long("purge-old"),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("init") {
        let db_filename = matches.value_of("DB_FILE").unwrap();
        let raw_filename = matches.value_of("ENTRY_FILE").unwrap();
        init_db(raw_filename, db_filename, epoch)?;
    }

    if let Some(matches) = matches.subcommand_matches("increment") {
        let db_filename = matches.value_of("DB_FILE").unwrap();
        let entry = matches.value_of("ENTRY").unwrap();
        let (fields, lines) = read_db(db_filename)?;
        increment_db(&fields, &lines, now, db_filename, entry)?;
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
