extern crate chrono;
extern crate clap;
extern crate failure;
extern crate fs2;

use chrono::{prelude::*, DateTime, NaiveDateTime};
use clap::{App, Arg, SubCommand};
use fs2::FileExt;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{prelude::*, BufWriter, SeekFrom, stdout};
use std::iter;
use std::path::Path;

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Field {
    count: i64,
    time: DateTime<Utc>,
    data: String
}

trait FieldSlice {
    fn sort_by_frecency(&mut self, dt: DateTime<Utc>);
    fn sort_by_data(&mut self);
}

impl Field {
    pub fn new(count: i64, time: DateTime<Utc>, data: &str) -> Self {
        Self { count, time, data: data.to_owned() }
    }

    pub fn frecency(&self, dt: &DateTime<Utc>) -> i64 {
        let secs = dt.signed_duration_since(self.time).num_seconds();
        (1e15 * frecency(self.count, secs)) as i64
    }

    pub fn to_info_str(&self, dt: DateTime<Utc>) -> String {
        let secs = dt.signed_duration_since(self.time).num_seconds();
        format!("{:.6}  {:6}  {:25}  {}",
                frecency(self.count, secs),
                self.count,
                self.time.to_rfc3339_opts(SecondsFormat::Secs, false),
                self.data)
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:06},{},{}",
               self.count,
               self.time.to_rfc3339_opts(SecondsFormat::Micros, false),
               self.data)
    }
}

impl FieldSlice for [Field] {
    fn sort_by_frecency(&mut self, dt: DateTime<Utc>) {
        self.sort_by_cached_key(|x| -x.frecency(&dt));
    }

    fn sort_by_data(&mut self) {
        // self.sort_by_cached_key(|x| x.data);
        self.sort_by(|x, y| x.data.cmp(&y.data));
    }
}

fn frecency(count: i64, secs: i64) -> f64 {
    if count == 0 {
        return 0.0;
    }

    let x = 0.0
        + 0.75 * (1.0 + count as f64).ln()
        - 0.25 * (1.0 + secs  as f64).ln();

    1.0 / (1.0 + (-x).exp())
}

fn get_old_fields<'a>(
    raw_lines: &[String],
    db_fields: &[Field],
    db_lookup: &HashMap<&String, (usize, &'a Field)>,
) -> impl Iterator<Item = &'a Field> {
    let old_set = {
        let raw_set = raw_lines.iter()
            .collect::<HashSet<&String>>();

        let db_set = db_fields.iter()
            .map(|x| &x.data)
            .collect::<HashSet<&String>>();

        db_set
            .difference(&raw_set)
            .map(|&x| x)
            .collect::<HashSet<&String>>()
    };

    let mut old_fields = old_set.into_iter()
        .map(|x| db_lookup[x])
        .collect::<Vec<_>>();

    old_fields.sort_by_key(|&(i, _)| i);
    old_fields.into_iter().map(|(_, x)| x)
}

fn parse_line(line: &str) -> Result<Field> {
    let split = line.splitn(3, ',').collect::<Vec<&str>>();

    let [count_str, time_str, data] = match split[0..3] {
        [x, y, z] => [x, y, z],
        _ => return Err(failure::err_msg("Insufficient entries"))
    };

    let time = DateTime::parse_from_rfc3339(&time_str)?.with_timezone(&Utc);
    let count = count_str.parse::<i64>()?;

    Ok(Field::new(count, time, data))
}

fn increment_db(
    fields: &[Field],
    lines: &[String],
    dt: DateTime<Utc>,
    db_filename: &str,
    entry: &str,
) -> Result<()> {
    let line = fields.iter().position(|x| x.data == entry)
        .ok_or(failure::err_msg("Entry not found in database"))?;

    let field = Field::new(fields[line].count + 1, dt, &fields[line].data);
    let write_str = field.to_string();
    let lengths = lines.iter().map(|x| x.len());
    let seek_begin = lengths.take(line).sum::<usize>() + line;
    let seek_end = seek_begin + lines[line].len() + 1;

    let mut db_file = OpenOptions::new()
        .write(true)
        .open(db_filename)?;

    assert!(write_str.len() == seek_end - seek_begin - 1);
    db_file.lock_exclusive()?;
    db_file.seek(SeekFrom::Start(seek_begin as u64))?;
    db_file.write_all(write_str.as_bytes())?;
    db_file.unlock()?;

    Ok(())
}

fn init_db(
    raw_filename: &str,
    db_filename: &str,
    dt: DateTime<Utc>
) -> Result<()> {
    let raw_str = fs::read_to_string(raw_filename)?;
    let mut db_file = File::create(db_filename)?;
    db_file.lock_exclusive()?;

    for line in raw_str.lines() {
        let field = Field::new(0, dt, line);
        writeln!(&mut db_file, "{}", field)?;
    }

    db_file.unlock()?;

    Ok(())
}

fn read_db(db_filename: &str) -> Result<(Vec<Field>, Vec<String>)> {
    let db_str = fs::read_to_string(db_filename)?;

    let lines = db_str.lines()
        .map(|x| x.to_owned())
        .collect::<Vec<String>>();

    let fields = lines.iter()
        .map(|x| parse_line(&x))
        .collect::<Result<Vec<Field>>>()?;

    Ok((fields, lines))
}

fn update_db(
    raw_filename: &str,
    db_filename: &str,
    dt: DateTime<Utc>,
    purge_old: bool,
) -> Result<()> {
    if !Path::new(db_filename).exists() {
        return init_db(raw_filename, db_filename, dt);
    }

    let raw_str = fs::read_to_string(raw_filename)?;
    let (db_fields, _) = read_db(db_filename)?;

    let raw_lines = raw_str.lines()
        .map(|x| x.to_owned())
        .collect::<Vec<String>>();

    let db_lookup = db_fields.iter()
        .enumerate()
        .map(|(i, x)| (&x.data, (i, x)))
        .collect::<HashMap<_, _>>();

    let new_fields = raw_lines.iter()
        .map(|x| Box::new(db_lookup.get(x)
             .map(|&(_, field)| field.to_owned())
             .unwrap_or_else(|| Field::new(0, dt, x))))
        .map(|x| *x)
        .collect::<Vec<_>>();

    let old_fields: Box<Iterator<Item = &Field>> = match purge_old {
        true  => Box::new(iter::empty::<&Field>()),
        false => Box::new(get_old_fields(&raw_lines, &db_fields, &db_lookup)),
    };

    let fields = new_fields.iter().chain(old_fields);

    let mut db_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(db_filename)?;

    db_file.lock_exclusive()?;

    for field in fields {
        writeln!(&mut db_file, "{}", field)?;
    }

    db_file.unlock()?;

    Ok(())
}

fn main() -> Result<()> {
    let epoch = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(0, 0), Utc);
    let now = Utc::now();

    let matches = App::new("frece")
        .version("1.0")
        .author("Mateen Ulhaq <mulhaq2005@gmail.com>")
        .about("Frecency indexed database")
        .subcommand(
            SubCommand::with_name("increment")
                .about("Increases an entry's count and resets its timer")
                .arg(Arg::with_name("DB_FILE")
                     .help("Path to frecency database file")
                     .required(true)
                     .index(1))
                .arg(Arg::with_name("ENTRY")
                     .help("Entry to increment")
                     .required(true)
                     .index(2)))
        .subcommand(
            SubCommand::with_name("init")
                .about("Creates a database file from given list of entries")
                .arg(Arg::with_name("DB_FILE")
                     .help("Path to frecency database file")
                     .required(true)
                     .index(1))
                .arg(Arg::with_name("ENTRY_FILE")
                     .help("Path to list of entries, separated by newlines")
                     .required(true)
                     .index(2)))
        .subcommand(
            SubCommand::with_name("print")
                .about("Prints list of frecency sorted entries")
                .arg(Arg::with_name("DB_FILE")
                     .help("Path to frecency database file")
                     .required(true)
                     .index(1))
                .arg(Arg::with_name("verbose")
                     .help("Outputs frecency, counts, date, and entries")
                     .short("v")
                     .long("verbose")))
        .subcommand(
            SubCommand::with_name("update")
                .about("Updates a database file from given list of entries")
                .arg(Arg::with_name("DB_FILE")
                     .help("Path to frecency database file")
                     .required(true)
                     .index(1))
                .arg(Arg::with_name("ENTRY_FILE")
                     .help("Path to list of entries, separated by newlines")
                     .required(true)
                     .index(2))
                .arg(Arg::with_name("purge-old")
                     .help("Purge any entries *not* in ENTRY_FILE")
                     .long("purge-old")))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("init") {
        let db_filename  = matches.value_of("DB_FILE").unwrap();
        let raw_filename = matches.value_of("ENTRY_FILE").unwrap();
        init_db(raw_filename, db_filename, epoch)?;
    }

    if let Some(matches) = matches.subcommand_matches("increment") {
        let db_filename  = matches.value_of("DB_FILE").unwrap();
        let entry        = matches.value_of("ENTRY").unwrap();
        let (fields, lines) = read_db(db_filename)?;
        increment_db(&fields, &lines, now, db_filename, entry)?;
    }

    if let Some(matches) = matches.subcommand_matches("print") {
        let db_filename  = matches.value_of("DB_FILE").unwrap();
        let verbose      = matches.is_present("verbose");
        let (mut fields, _lines) = read_db(db_filename)?;
        fields.sort_by_frecency(now);

        let stdout = stdout();
        let lock = stdout.lock();
        let mut w = BufWriter::new(lock);
        let to_str = |field: Field|
            if verbose { field.to_info_str(now) }
            else { field.data };

        for field in fields {
            writeln!(w, "{}", to_str(field))?;
        }
    }

    if let Some(matches) = matches.subcommand_matches("update") {
        let db_filename  = matches.value_of("DB_FILE").unwrap();
        let raw_filename = matches.value_of("ENTRY_FILE").unwrap();
        let purge_old    = matches.is_present("purge-old");
        update_db(raw_filename, db_filename, epoch, purge_old)?;
    }

    Ok(())
}

// TODO unit tests? doc strings?
// TODO Allow user to specify custom frecency weights
// TODO --no-header flag for --verbose? or just header by default?
// TODO --null-delimiter (avoids errors with Linux file paths...)
// TODO escape \\ and \n? Or at least, warn on input containing newlines; also watch out for \r\n behavior with lines(); try .split("\n")?
// TODO exception on duplicate entries in raw_lines?
// TODO Fast write to stdout/pipe
// https://codereview.stackexchange.com/questions/73753/how-can-i-find-out-why-this-rust-program-to-echo-lines-from-stdin-is-slow
// https://www.reddit.com/r/rust/comments/5puyx2/why_is_println_so_slow/dcu2lf0
// https://www.reddit.com/r/rust/comments/94roxv/how_to_print_files_to_stdout_fast
// https://www.reddit.com/r/rust/comments/ab9b3z/locking_stdout_once_and_writing_to_it_versus
// fastcat https://matthias-endler.de/2018/fastcat/
