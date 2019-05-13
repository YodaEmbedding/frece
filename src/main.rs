extern crate chrono;
extern crate failure;

use chrono::{prelude::*, DateTime, NaiveDateTime};
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
    fn sort_by_frecency(&mut self, dt: DateTime<Utc>);
    fn sort_by_data(&mut self);
}

trait FieldIterator {
    fn to_data_str(self) -> String;
    fn to_info_str(self, dt: DateTime<Utc>) -> String;
}

impl Field {
    pub fn new(count: i64, time: DateTime<Utc>, data: String) -> Self {
        Self { count, time, data }
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

// impl<I> FieldIterator for I where
//     I: Iterator<Item = Field> {
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
    let x = 0.0
        + 0.25 * (1.0 + count as f64).ln()
        - 0.25 * (1.0 + secs  as f64).ln();

    1.0 / (1.0 + (-x).exp())

    // TODO maybe if secs == 0, return 0.0?
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
    dt: DateTime<Utc>,
    db_filename: &str,
    line: usize
) -> Result<()> {
    let field = Field::new(
        fields[line].count + 1,
        dt,
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
    dt: DateTime<Utc>
) -> Result<()> {
    let raw_str = fs::read_to_string(raw_filename)?;
    let mut db_file = File::create(db_filename)?;

    for line in raw_str.lines() {
        let field = Field::new(0, dt, line.to_owned());
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

fn get_old_fields(
    raw_lines: &[String],
    db_fields: &[Field],
    db_lookup: &HashMap<&String, (usize, &Field)>,
) -> Vec::<Field> {
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

    let old_fields = old_fields.into_iter()
        .map(|(_, x)| x.to_owned());

    old_fields.collect::<Vec<_>>()
}

fn update_db(
    raw_filename: &str,
    db_filename: &str,
    dt: DateTime<Utc>
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
             .unwrap_or_else(|| Field::new(0, dt, x.to_owned()))))
        .map(|x| *x);

    let purge_old = false;

    let old_fields: Box<Iterator<Item = Field>> = match purge_old {
        true  => Box::new(iter::empty::<Field>()),
        false => Box::new(get_old_fields(&raw_lines, &db_fields, &db_lookup)
                          .into_iter()),
    };

    let fields = new_fields.chain(old_fields);

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
    let raw_filename = "raw.txt";
    let db_filename = "db.txt";

    // init_db(raw_filename, db_filename, epoch)?;

    update_db(raw_filename, db_filename, epoch)?;

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


// Switch to "simple" for now...
// But (sortByCount . update) probably preserves behavior due to stability of frecency sort?
// (Verify that sortByFrecency . sortByCount . sortById == sortByFrecency . sortById)
// errr actually it sounds like it doesn't...? but time is causal... so maybe? ...but non-linear? but is linear weighted? idk...


// TODO remame frece, update readme, put on AUR (bin, git), fix examples, continuous integration
// TODO unit tests? doc strings?
// TODO lock database file
// TODO "Database" object?
// TODO unnecessary collect before join?
// TODO file::create automatically
// TODO allow frecency weights to be tweaked?
// TODO DbWriter: writes only invalidated entries? ...too complicated for a simple program, dude...
// TODO to_owned can all be fixed by changing collect type into &String?
// TODO some copying can be reduced using into_iter() instead of iter()
// TODO cloned/to_owned/borrowed()?
// See https://hermanradtke.com/2015/06/22/effectively-using-iterators-in-rust.html

// TODO argparse
// init
// update/pull/merge
//   --purge clear unused entries in frecent index?
//   --reset-time  reset time for all 0-count entries to UTC-0
// TODO --reset-time should use oldest time...
// maybe --epoch can be a parameter!
// increment



// TODO WHY ARE WE BOTHERING TO SORT BY COUNT, ANYWAYS?
// if we do this, we don't preserve the initial ordering when it matters!!
// (but it rarely matters, I guess...)


// TODO so... remove "swap" functionality
// Probably need to purge to maintain order? Actually... not really. Can interweave.


// Alternatively, add an GUID for insertion order... doesn't work too well with merges though... makes things complicated
