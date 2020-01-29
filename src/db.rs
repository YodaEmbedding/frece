use chrono::{prelude::*, DateTime};
use fs2::FileExt;
use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::{prelude::*, BufWriter, SeekFrom};
use std::iter;

use crate::field::Field;

type Result<T> = std::result::Result<T, failure::Error>;

/// Add specified database entry, if it does not exist.
pub fn add_db(
    fields: &[Field],
    db_filename: &str,
    entry: &str,
    dt: DateTime<Utc>,
) -> Result<()> {
    let line = fields.iter().position(|x| x.data == entry);

    if line != None {
        return Err(failure::err_msg("Entry found in database"));
    }

    let field = Field::new(0, dt, entry);

    let mut db_file = OpenOptions::new().write(true).open(db_filename)?;

    db_file.lock_exclusive()?;
    db_file.seek(SeekFrom::End(0))?;
    writeln!(&mut db_file, "{}", field)?;
    db_file.unlock()?;

    Ok(())
}

/// Initializes database using given list of entries.
pub fn init_db(
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
pub fn read_db(db_filename: &str) -> Result<(Vec<Field>, Vec<String>)> {
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

/// Set specified database entry's fields via given function.
pub fn setfield_db<FieldFunc>(
    fields: &[Field],
    lines: &[String],
    db_filename: &str,
    entry: &str,
    field_func: FieldFunc,
) -> Result<()>
where
    FieldFunc: Fn(&Field) -> Field,
{
    let line = fields
        .iter()
        .position(|x| x.data == entry)
        .ok_or(failure::err_msg("Entry not found in database"))?;

    let field = field_func(&fields[line]);
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

/// Update database with list of entries.
pub fn update_db(
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

pub fn parse_time(s: &str) -> Result<chrono::DateTime<chrono::Utc>> {
    Ok(DateTime::parse_from_rfc3339(&s)?.with_timezone(&Utc))
}

fn parse_line(line: &str) -> Result<Field> {
    let split = line.splitn(3, ',').collect::<Vec<&str>>();

    let [count_str, time_str, data] = match split[0..3] {
        [x, y, z] => [x, y, z],
        _ => return Err(failure::err_msg("Insufficient entries")),
    };

    let time = parse_time(&time_str)?;
    let count = count_str.parse::<i64>()?;

    Ok(Field::new(count, time, data))
}
