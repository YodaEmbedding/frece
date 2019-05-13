extern crate chrono;
extern crate clap;
extern crate failure;
extern crate fs2;

use chrono::{prelude::*, DateTime, NaiveDateTime};
use clap::{App, Arg, SubCommand};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{prelude::*, SeekFrom};
use std::slice::Iter;
use std::{fmt, iter};

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

trait FieldIterator {
    fn to_data_str(self) -> String;
    fn to_info_str(self, dt: DateTime<Utc>) -> String;
}

impl Field {
    pub fn new(count: i64, time: DateTime<Utc>, data: &str) -> Self {
        Self { count, time, data: data.to_owned() }
    }

    pub fn frecency(&self, dt: &DateTime<Utc>) -> i64 {
        let secs = dt.signed_duration_since(self.time).num_seconds();
        (1e15 * frecency(self.count, secs)) as i64
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

impl<'a> FieldIterator for Iter<'a, Field> {
    fn to_data_str(self) -> String {
        self
            .map(|x| x.data.as_ref())
            .collect::<Vec<&str>>()
            .join("\n")
    }

    fn to_info_str(self, dt: DateTime<Utc>) -> String {
        self
            .map(|field| {
                let secs = dt.signed_duration_since(field.time).num_seconds();
                format!("{:.6}  {:6}  {:25}  {}",
                        frecency(field.count, secs),
                        field.count,
                        field.time.to_rfc3339_opts(SecondsFormat::Secs, false),
                        field.data)
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}

fn frecency(count: i64, secs: i64) -> f64 {
    if count == 0 {
        return 0.0;
    }

    let x = 0.0
        + 0.25 * (1.0 + count as f64).ln()
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
    db_file.seek(SeekFrom::Start(seek_begin as u64))?;
    db_file.write_all(write_str.as_bytes())?;

    Ok(())
}

fn init_db(
    raw_filename: &str,
    db_filename: &str,
    dt: DateTime<Utc>
) -> Result<()> {
    let raw_str = fs::read_to_string(raw_filename)?;
    let mut db_file = File::create(db_filename)?;

    for line in raw_str.lines() {
        let field = Field::new(0, dt, line);
        writeln!(&mut db_file, "{}", field)?;
    }

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

    for field in fields {
        writeln!(&mut db_file, "{}", field)?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let epoch = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(0, 0), Utc);
    let now = Utc::now();

    let matches = App::new("frece")
        .version("1.0")
        .author("Mateen Ulhaq <mulhaq2005@gmail.com>")
        .about("Frecency indexed database manager.")
        .subcommand(
            SubCommand::with_name("increment")
            .about("Print frecency sorted entries")
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
            .about("Create a database file from given entries file")
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
            .about("Print frecency sorted entries")
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
            .about("Updates a database file from given entries file")
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

        if verbose {
            println!("{}", fields.iter().to_info_str(now));
        }
        else {
            println!("{}", fields.iter().to_data_str());
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

// TODO remame frece, update readme, put on AUR (bin, git), fix examples, continuous integration
// TODO unit tests? doc strings?
// TODO lock database file
// TODO Allow user to specify custom frecency weights
