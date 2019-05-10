extern crate chrono;
extern crate failure;

use chrono::{prelude::*, DateTime};
use std::{fmt, iter};
use std::fs::{self, File, OpenOptions};
use std::io::{prelude::*, SeekFrom};

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
    let x = 1.0
        + 0.08 * (1.0 + count as f64).ln()
        - 0.10 * (1.0 + secs  as f64).ln();

    1.0 / (1.0 + (-x).exp())
}

fn sort_by_frecency(fields: &mut [Field], now: DateTime<Utc>) {
    fields.sort_by_cached_key(|x| -x.frecency(&now));
}

fn to_name_str(fields: &[Field]) -> String {
    fields.iter()
        .map(|x| x.data.as_ref())
        .collect::<Vec<&str>>()
        .join("\n")
}

fn to_info_str(fields: &[Field], now: DateTime<Utc>) -> String {
    fields.iter()
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

fn increment_db(fields: &[Field],
                lines: &[String],
                now: DateTime<Utc>,
                filename: &str,
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

    let (write_str, seek_begin, seek_end) = {
        if line == 0 || fields[line - 1].count >= field.count {
            // Partial ordering is still valid; no line swap required
            (field.to_string(), seek_pos(line), seek_pos(line + 1))
        }
        else {
            // Partial ordering is invalid; line swap required
            let line1 = fields[..line].iter()
                .rposition(|x| x.count >= field.count)
                .map(|x| x + 1)
                .unwrap_or(0);
            let line2 = line;

            let top = iter::once(field_str);
            let middle = lines.iter()
                .skip(line1 + 1)
                .take(line2 - line1 - 1)
                .map(|x| x.to_owned())
                .collect::<Vec<String>>();
            let bottom = iter::once(lines[line1].to_owned());

            let write_str = top
                .chain(middle)
                .chain(bottom)
                .collect::<Vec<String>>()
                .join("\n");

            (write_str, seek_pos(line1), seek_pos(line2 + 1))
        }
    };

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(filename)?;

    assert!(write_str.len() == seek_end - seek_begin - 1);
    file.seek(SeekFrom::Start(seek_begin as u64))?;
    file.write_all(write_str.as_bytes())?;

    Ok(())
}

#[allow(dead_code)]
fn init_db(raw_filename: &str,
           db_filename: &str,
           now: DateTime<Utc>
) -> Result<()> {
    let raw_str = fs::read_to_string(raw_filename)?;
    let mut db_file = File::create(db_filename)?;

    for line in raw_str.lines() {
        let out_line = Field::new(0, now, line.to_owned()).to_string();
        writeln!(&mut db_file, "{}", out_line)?;
    }

    Ok(())
}

fn read_db(filename: &str) -> Result<(Vec<Field>, Vec<String>)> {
    let db_str = fs::read_to_string(filename)?;

    let lines = db_str.lines()
        .map(|x| x.to_owned())
        .collect::<Vec<String>>();

    let fields = lines.iter()
        .map(|x| parse_line(&x))
        .collect::<Result<Vec<Field>>>()?;

    Ok((fields, lines))
}

#[allow(dead_code)]
fn update_db(raw_filename: &str,
             db_filename: &str,
             now: DateTime<Utc>
) -> Result<()> {
    Ok(())
}

fn main() -> Result<()> {
    let now = Utc::now();
    let raw_filename = "raw.txt";
    let db_filename = "db.txt";

    // init_db(raw_filename, db_filename, now)?;

    let (fields, lines) = read_db(db_filename)?;
    increment_db(&fields, &lines, now, db_filename, 5)?;

    let (fields, _) = read_db(db_filename)?;
    let mut sorted_fields = fields.iter()
        .map(|x| x.clone())
        .collect::<Vec<Field>>();

    sort_by_frecency(&mut sorted_fields, now);
    println!("{}", to_name_str(&sorted_fields));
    println!("{}", to_info_str(&sorted_fields, now));

    Ok(())
}

// TODO "Database" object?
// TODO unit tests? doc strings?
// TODO unnecessary collect before join?
// TODO file::create automatically
// TODO might be more elegant/faster to sort in reverse order?
// TODO allow frecency weights to be tweaked?
// TODO DbWriter: writes only invalidated entries? ...too complicated for a simple program, dude...

// TODO argparse
// init
// update/pull/merge
//   --purge clear unused entries in frecent index?
// increment
