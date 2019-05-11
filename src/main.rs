extern crate chrono;
extern crate failure;

use chrono::{prelude::*, DateTime};
use std::{fmt, iter};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{prelude::*, SeekFrom};
use std::slice::Iter;

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Field {
    count: i64,
    time: DateTime<Utc>,
    data: String
}

trait FieldSlice {
    fn sort_by_frecency(&mut self, now: DateTime<Utc>);
    fn sort_by_data(&mut self);
}

trait FieldIterator {
    fn to_data_str(self) -> String;
    fn to_info_str(self, now: DateTime<Utc>) -> String;
}

impl Field {
    pub fn new(count: i64, time: DateTime<Utc>, data: String) -> Self {
        Self { count, time, data }
    }

    pub fn frecency(&self, now: &DateTime<Utc>) -> i64 {
        let secs = now.signed_duration_since(self.time).num_seconds();
        (1e15 * frecency(self.count, secs)) as i64
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:06},{},{}", self.count, self.time.to_rfc3339(), self.data)
    }
}

impl FieldSlice for [Field] {
    fn sort_by_frecency(&mut self, now: DateTime<Utc>) {
        self.sort_by_cached_key(|x| -x.frecency(&now));
    }

    fn sort_by_data(&mut self) {
        // self.sort_by_cached_key(|x| x.data);
        self.sort_by(|x, y| x.data.cmp(&y.data));
    }
}

// impl<I> FieldIterator for I where
//     I: Iterator<Item = Field> {
impl<'a> FieldIterator for Iter<'a, Field> {
    fn to_data_str(self) -> String {
        self
            .map(|x| x.data.as_ref())
            .collect::<Vec<&str>>()
            .join("\n")
    }

    fn to_info_str(self, now: DateTime<Utc>) -> String {
        self
            .map(|field| {
                let secs = now.signed_duration_since(field.time).num_seconds();
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
    let x = 1.0
        + 0.10 * (1.0 + count as f64).ln()
        - 0.10 * (1.0 + secs  as f64).ln();

    1.0 / (1.0 + (-x).exp())
}

fn parse_line(line: &str) -> Result<Field> {
    let split = line.splitn(3, ',').collect::<Vec<&str>>();

    let [count_str, time_str, data] = match split[0..3] {
        [x, y, z] => [x, y, z],
        _ => return Err(failure::err_msg("Insufficient entries"))
    };

    let time = DateTime::parse_from_rfc3339(&time_str)?.with_timezone(&Utc);
    let count = count_str.parse::<i64>()?;

    Ok(Field::new(count, time, data.to_owned()))
}

fn increment_db(
    fields: &[Field],
    lines: &[String],
    now: DateTime<Utc>,
    db_filename: &str,
    line: usize
) -> Result<()> {
    let field = Field::new(
        fields[line].count + 1,
        now,
        fields[line].data.clone());

    let field_str = field.to_string();

    let lengths = lines.iter()
        .map(|x| x.len())
        .collect::<Vec<usize>>();

    let seek_pos = |x| lengths.iter().take(x).sum::<usize>() + x;

    // TODO actually, this can be further simplified to only write "count,time"
    let write_str = field_str;
    let seek_begin = seek_pos(line);
    let seek_end = seek_pos(line + 1);

    // TODO Remove?
    // Maintain count order
    // let (write_str, seek_begin, seek_end) = {
    //     if line == 0 || fields[line - 1].count >= field.count {
    //         // Partial ordering is still valid; no line swap required
    //         (field_str, seek_pos(line), seek_pos(line + 1))
    //     }
    //     else {
    //         // Partial ordering is invalid; line swap required
    //         let line1 = fields[..line].iter()
    //             .rposition(|x| x.count >= field.count)
    //             .map(|x| x + 1)
    //             .unwrap_or(0);
    //         let line2 = line;
    //
    //         let middle = lines.iter()
    //             .skip(line1 + 1)
    //             .take(line2 - line1 - 1);
    //
    //         let write_str = iter::once(&field_str)
    //             .chain(middle)
    //             .chain(iter::once(&lines[line1]))
    //             .map(|x| x.as_str())
    //             .collect::<Vec<&str>>()
    //             .join("\n");
    //
    //         (write_str, seek_pos(line1), seek_pos(line2 + 1))
    //     }
    // };

    let mut db_file = OpenOptions::new()
        .write(true)
        .open(db_filename)?;

    assert!(write_str.len() == seek_end - seek_begin - 1);
    db_file.seek(SeekFrom::Start(seek_begin as u64))?;
    db_file.write_all(write_str.as_bytes())?;

    Ok(())
}

#[allow(dead_code)]
fn init_db(
    raw_filename: &str,
    db_filename: &str,
    now: DateTime<Utc>
) -> Result<()> {
    let raw_str = fs::read_to_string(raw_filename)?;
    let mut db_file = File::create(db_filename)?;

    for line in raw_str.lines() {
        let field = Field::new(0, now, line.to_owned());
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

#[allow(dead_code)]
fn update_db(
    raw_filename: &str,
    db_filename: &str,
    now: DateTime<Utc>
) -> Result<()> {
    let raw_str = fs::read_to_string(raw_filename)?;
    let (db_fields, _) = read_db(db_filename)?;

    let raw_lines = raw_str.lines()
        .map(|x| x.to_owned())
        .collect::<Vec<String>>();

    // let raw_set = raw_lines.iter()
    //     .map(|x| x.to_owned())
    //     .collect::<HashSet<String>>();
    //
    // let db_set = db_fields.iter()
    //     .map(|x| x.data.to_owned())
    //     .collect::<HashSet<String>>();
    //
    // let old_set = db_set
    //     .difference(&raw_set)
    //     .collect::<HashSet<&String>>();

    // TODO
    // Can also consider mapping "GUID" <-> "String" to conserve memory
    // Or use pointers... or reference counted strings!
    // Or uh... what's wrong with plain old references to immutable memory?

    let db_lookup = db_fields.into_iter()
        .enumerate()
        .map(|(i, x)| (x.data.to_owned(), (i, x)))
        .collect::<HashMap<_, _>>();

    let raw_fields = raw_lines.iter()
        .map(|x| db_lookup
             .get(x)
             .map(|(_, y)| y.to_owned())
             .unwrap_or_else(|| Field::new(0, now, x.to_owned())))
        .collect::<Vec<_>>();

    let fields = raw_fields;

    // let purge_old = false;

    // if !purge_old && !old_set.is_empty() {
    // let db_fields = ...
    // }

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
    let now = Utc::now();
    let raw_filename = "raw.txt";
    let db_filename = "db.txt";

    // init_db(raw_filename, db_filename, now)?;

    update_db(raw_filename, db_filename, now)?;

    let (fields, lines) = read_db(db_filename)?;
    increment_db(&fields, &lines, now, db_filename, 5)?;

    let (fields, _lines) = read_db(db_filename)?;
    let mut sorted_fields = fields.iter()
        .cloned()
        .collect::<Vec<Field>>();

    // TODO do you REALLY want to have to call iter() beforehand?
    sorted_fields.sort_by_frecency(now);
    println!("{}", sorted_fields.iter().to_data_str());
    println!("{}", sorted_fields.iter().to_info_str(now));

    Ok(())
}







// Draw out model and API on paper?

// Switch to "simple" for now...
// But (sortByCount . update) probably preserves behavior due to stability of frecency sort?
// (Verify that sortByFrecency . sortByCount . sortById == sortByFrecency . sortById)
// errr actually it sounds like it doesn't...? but time is causal... so maybe? ...but non-linear? but is linear weighted? idk...






// TODO lock database file
// TODO "Database" object?
// TODO unit tests? doc strings?
// TODO unnecessary collect before join?
// TODO file::create automatically
// TODO might be more elegant/faster to sort in reverse order?
// TODO allow frecency weights to be tweaked?
// TODO DbWriter: writes only invalidated entries? ...too complicated for a simple program, dude...
// TODO to_owned can all be fixed by changing collect type into &String?
// TODO some copying can be reduced using into_iter() instead of iter()
// TODO cloned/to_owned/borrowed()?
// See https://hermanradtke.com/2015/06/22/effectively-using-iterators-in-rust.html
// TODO remame frece, update readme, put on AUR, add examples

// TODO argparse
// init
// update/pull/merge
//   --purge clear unused entries in frecent index?
// increment




// TODO WHY ARE WE BOTHERING TO SORT BY COUNT, ANYWAYS?
// if we do this, we don't preserve the initial ordering when it matters!!
// (but it rarely matters, I guess...)


// TODO so... remove "swap" functionality
// Probably need to purge to maintain order? Actually... not really. Can interweave.


// Alternatively, add an GUID for insertion order... doesn't work too well with merges though... makes things complicated
