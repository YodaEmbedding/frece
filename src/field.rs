use chrono::{prelude::*, DateTime};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Field {
    pub count: i64,
    pub time: DateTime<Utc>,
    pub data: String,
}

pub trait FieldSlice {
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
