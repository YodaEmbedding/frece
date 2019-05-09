extern crate chrono;
extern crate failure;

use chrono::{prelude::*, DateTime};
use std::{cmp, fmt, iter};
use std::fs::{self, File, OpenOptions};
use std::io::{prelude::*, SeekFrom};

// init
// update
//   --purge clear unused entries in frecent index?
// increment

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Clone)]
struct Field {
    count: i64,
    time: DateTime<Utc>,
    data: String
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

fn frecency(count: i64, secs: i64) -> f64 {
    let x = 0.0
        + 0.25 * (1.0 + count as f64).ln()
        - 0.30 * (1.0 + secs  as f64).ln();

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

fn init_db(now: DateTime<Utc>) -> Result<()> {
    let raw_str = fs::read_to_string("raw.txt")?;
    let mut db_file = File::create("db.txt")?;

    for line in raw_str.lines() {
        let out_line = Field::new(0, now, line.to_owned()).to_string();
        writeln!(&mut db_file, "{}", out_line)?;
    }

    Ok(())
}

// TODO generic type?
fn db_read_fields() -> Result<Vec<Field>> {
    let db_str = fs::read_to_string("db.txt")?;
    let fields = db_str.lines()
        .map(parse_line)
        .collect::<Result<Vec<Field>>>()?;

    Ok(fields)
}

// fn sort_fields<I>(<...>) -> Result<...>
//
// fn my_func<T: AsRef<str>>(list: &[T]) {
//     for s in list {
//         println!("{}", s.as_ref());
//     }
// }
//
// fn my_func<I>(list: I)
//     where I: IntoIterator,
//           I::Item: AsRef<str>,
// {
//     for s in list {
//         println!("{}", s.as_ref());
//     }
// }

// TODO return value...? or rename to "db_print_sorted?"
fn db_sorted(now: DateTime<Utc>) -> Result<()> {
    let mut fields = db_read_fields()?;
    fields.sort_by_cached_key(|x| -x.frecency(&now));

    let sorted_str = fields.iter()
        .map(|x| x.data.as_ref())
        .collect::<Vec<&str>>()
        .join("\n");

    println!("{}", sorted_str);

    Ok(())
}

fn db_info(now: DateTime<Utc>) -> Result<()> {
    // TODO SORT???

    let db_str = fs::read_to_string("db.txt")?;
    for line in db_str.lines() {
        let field = parse_line(line)?;
        let secs = now.signed_duration_since(field.time).num_seconds();
        println!("{:.6} {:6} {} {}",
                 frecency(field.count, secs),
                 field.count, secs, field.data);
    }

    Ok(())
}

// TODO might be better that these functions only read the whole file once...
// And don't recompute lines/lengths/etc...
fn increment_db(line: usize) -> Result<()> {
    let mut fields = db_read_fields()?;
    fields[line].count += 1;  // TODO is this really needed? why not just make new var...

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("db.txt")?;

    let mut file_str = String::new();
    file.read_to_string(&mut file_str)?;

    let lines = file_str.lines()
        .collect::<Vec<&str>>();

    // TODO just zip this with fields? or make it a part of Field? idk...
    // maybe just calculate directly from each field's string length...
    // or have parse_line return a tuple (len, field)
    let lengths = lines.iter()
        .map(|x| x.len())
        .collect::<Vec<usize>>();

    // Check if line swap required
    if line == 0 || fields[line - 1].count >= fields[line].count {
        let seek_pos = |x| lengths.iter().take(x).sum::<usize>() + x;
        let seek_begin = seek_pos(line);
        let seek_end   = seek_pos(line + 1);

        let write_str = fields[line].to_string();
        assert!(write_str.len() == seek_end - seek_begin - 1);

        file.seek(SeekFrom::Start(seek_begin as u64))?;
        file.write_all(write_str.as_bytes())?;

        return Ok(());
    }

    // TODO this could be done instead of explicit if check above?
    let line1 = fields[..line].iter()
        .rposition(|x| x.count >= fields[line].count)
        .map(|x| x + 1)
        .unwrap_or(0);
    let line2 = line;

    let seek_pos = |x| lengths.iter().take(x).sum::<usize>() + x;
    let seek_begin = seek_pos(line1);
    let seek_end   = seek_pos(line2 + 1);

    let s = fields[line].to_string();
    let top = iter::once(&s[..]);  // TODO wat?
    // let top = iter::once(lines[line2]);
    let middle = lines.iter()
        .skip(line1 + 1)
        .take(line2 - line1 - 1)
        .map(|&x| x);
    let bottom = iter::once(lines[line1]);

    // TODO is collect necessary? maybe itertools::join? benchmark.
    let write_str = top
        .chain(middle)
        .chain(bottom)
        .collect::<Vec<&str>>()
        .join("\n");

    assert!(write_str.len() == seek_end - seek_begin - 1);
    file.seek(SeekFrom::Start(seek_begin as u64))?;
    file.write_all(write_str.as_bytes())?;

    Ok(())
}

fn main() -> Result<()> {
    let now = Utc::now();

    // init_db(now)?;
    db_sorted(now)?;
    // swap_db(0, 8)?;
    db_info(now)?;
    increment_db(5)?;
    // Write db (increment and swap if needed?)

    Ok(())
}

// TODO "Database" object?
// TODO unit tests? doc strings?
// TODO unnecessary collect before join?
// TODO file::create automatically
// TODO might be more elegant/faster to sort in reverse order?
// TODO reduce code duplication... and make proper pure functions and file access!
