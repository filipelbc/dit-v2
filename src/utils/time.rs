use anyhow::{Context, Result};
use chrono::{DateTime, Duration, FixedOffset, Local, TimeZone};
use lazy_static::lazy_static;
use regex::{Captures, Regex};

use crate::utils::nice::Nice;

lazy_static! {
    static ref TIMESTAMP_RE: Regex = Regex::new(
        r"^(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})-(?P<h>\d{1,2}):(?P<min>\d{2})(:(?P<s>\d{2}))?$"
    )
    .unwrap();
    static ref TIME_RE: Regex =
        Regex::new(r"^(?P<h>\d{1,2}):(?P<min>\d{2})(:(?P<s>\d{2}))?$").unwrap();
    static ref DURATION_RE: Regex = Regex::new(
        r"^((?P<d>[+-]?\d+)d)?((?P<h>[+-]?\d+)h)?((?P<min>[+-]?\d+)min)?((?P<s>[+-]?\d+)s)?$"
    )
    .unwrap();
}

const TIMESTAMP_FORMAT: &str = "%F %T %z";

pub type Timestamp = DateTime<FixedOffset>;

pub fn now() -> Timestamp {
    local_to_fixed(Local::now())
}

fn local_to_fixed(local_date_time: DateTime<Local>) -> DateTime<FixedOffset> {
    local_date_time.with_timezone(local_date_time.offset())
}

impl Nice for Timestamp {
    fn nice(&self) -> String {
        self.format(TIMESTAMP_FORMAT).to_string()
    }
}

impl Nice for Duration {
    fn nice(&self) -> String {
        let mut r = self.num_seconds();

        if r == 0 {
            return "0s".to_string();
        }

        let hours = r / 3600;
        r %= 3600;

        format!(
            "{}{}{}",
            format_duration_piece(hours, "h"),
            format_duration_piece(r / 60, "min"),
            format_duration_piece(r % 60, "s"),
        )
    }
}

fn format_duration_piece(x: i64, suffix: &str) -> String {
    if x == 0 {
        String::new()
    } else {
        format!("{}{}", x, suffix)
    }
}

pub fn parse_timestamp(x: &str) -> Option<Timestamp> {
    try_timestamp(x).or(try_time(x)).or(try_duration(x))
}

fn parse_duration(x: &str) -> Option<Duration> {
    DURATION_RE.captures(x).map(|m| {
        let s = i(&m, "h") * 3600 + i(&m, "min") * 60 + i(&m, "s");
        Duration::seconds(i64::from(s))
    })
}

fn try_timestamp(x: &str) -> Option<Timestamp> {
    TIMESTAMP_RE
        .captures(x)
        .map(|m| {
            Local.ymd(i(&m, "y"), u(&m, "m"), u(&m, "d")).and_hms(
                u(&m, "h"),
                u(&m, "min"),
                u(&m, "s"),
            )
        })
        .map(local_to_fixed)
}

fn try_time(x: &str) -> Option<Timestamp> {
    TIME_RE
        .captures(x)
        .map(|m| Local::today().and_hms(u(&m, "h"), u(&m, "min"), u(&m, "s")))
        .map(local_to_fixed)
}

fn try_duration(x: &str) -> Option<Timestamp> {
    parse_duration(x)
        .map(|d| Local::now() + d)
        .map(local_to_fixed)
}

fn i(x: &Captures, n: &str) -> i32 {
    x.name(n)
        .map(|m| i32::from_str_radix(m.as_str(), 10).unwrap())
        .unwrap_or(0)
}

fn u(x: &Captures, n: &str) -> u32 {
    x.name(n)
        .map(|m| u32::from_str_radix(m.as_str(), 10).unwrap())
        .unwrap_or(0)
}

pub mod timestamp {

    use chrono::DateTime;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    use super::{Nice, Timestamp, TIMESTAMP_FORMAT};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_str(s.as_str(), TIMESTAMP_FORMAT)
            .map_err(|e| D::Error::custom(format!("Invalid datetime: {}", e)))
    }

    pub fn serialize<S>(value: &Timestamp, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(value.nice().as_str())
    }

    pub mod optional {

        use serde::{Deserializer, Serializer};

        use super::super::Timestamp;

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Timestamp>, D::Error>
        where
            D: Deserializer<'de>,
        {
            super::deserialize(deserializer).map(|x| Some(x))
        }

        pub fn serialize<S>(value: &Option<Timestamp>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match value {
                Some(x) => super::serialize(x, serializer),
                None => serializer.serialize_none(),
            }
        }
    }
}

pub mod duration {

    use chrono::Duration;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    use super::{parse_duration, Nice};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse_duration(s.as_str())
            .ok_or_else(|| D::Error::custom(format!("Invalid duration: {}", s)))
    }

    pub fn serialize<S>(value: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(value.nice().as_str())
    }
}

#[cfg(test)]
mod tests {

    use super::parse_timestamp;

    macro_rules! assert_parses {
        ($expr:expr) => {{
            if let None = parse_timestamp($expr) {
                panic!(
                    "assertion failed: parse_timestamp({}) should be Some(_) but is None",
                    stringify!($expr),
                );
            }
        }};
    }

    macro_rules! assert_parses_not {
        ($expr:expr) => {{
            if let Some(x) = parse_timestamp($expr) {
                panic!(
                    "assertion failed: parse_timestamp({}) should be None but is Some({})",
                    stringify!($expr),
                    x,
                );
            }
        }};
    }

    #[test]
    fn test_parse_datetime() {
        assert_parses!("1:22");
        assert_parses!("11:22");
        assert_parses!("11:22:33");

        assert_parses!("2020-10-20-1:22");
        assert_parses!("2020-10-20-11:22");
        assert_parses!("2020-10-20-11:22:33");

        assert_parses!("-11d");
        assert_parses!("-11h");
        assert_parses!("-11min");
        assert_parses!("-11s");
        assert_parses!("-3d-3h-123min-123s");

        assert_parses!("11d");
        assert_parses!("11h");
        assert_parses!("11min");
        assert_parses!("11s");
        assert_parses!("3d3h123min123s");

        assert_parses_not!("11");
        assert_parses_not!("11:2");
        assert_parses_not!("11:222");
        assert_parses_not!("11:22:3");

        assert_parses_not!("2020");
        assert_parses_not!("2020-10");
        assert_parses_not!("2020-10-20");
        assert_parses_not!("2020-10-20-11");
    }
}
