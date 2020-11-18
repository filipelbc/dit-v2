use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Local, TimeZone};
use lazy_static::lazy_static;
use regex::{Captures, Regex};

lazy_static! {
    static ref DATETIME_RE: Regex = Regex::new(
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

pub type LocalDateTime = DateTime<Local>;

pub fn resolve(at: Option<&str>) -> Result<LocalDateTime> {
    match at {
        Some(x) => parse_datetime(x).with_context(|| format!("Invalid date/time value: {}", x)),
        None => Ok(Local::now()),
    }
}

pub fn format_duration(x: &Duration) -> String {
    let mut r = x.num_seconds();

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

fn format_duration_piece(x: i64, suffix: &str) -> String {
    if x == 0 {
        "".to_string()
    } else {
        format!("{}{}", x, suffix)
    }
}

fn parse_datetime(x: &str) -> Option<LocalDateTime> {
    try_datetime(x).or(try_time(x)).or(try_duration(x))
}

fn parse_duration(x: &str) -> Option<Duration> {
    DURATION_RE.captures(x).map(|m| {
        let s = i(&m, "h") * 3600 + i(&m, "min") * 60 + i(&m, "s");
        Duration::seconds(i64::from(s))
    })
}

fn try_datetime(x: &str) -> Option<LocalDateTime> {
    DATETIME_RE.captures(x).map(|m| {
        Local
            .ymd(i(&m, "y"), u(&m, "m"), u(&m, "d"))
            .and_hms(u(&m, "h"), u(&m, "min"), u(&m, "s"))
    })
}

fn try_time(x: &str) -> Option<LocalDateTime> {
    TIME_RE
        .captures(x)
        .map(|m| Local::today().and_hms(u(&m, "h"), u(&m, "min"), u(&m, "s")))
}

fn try_duration(x: &str) -> Option<LocalDateTime> {
    parse_duration(x).map(|d| Local::now() + d)
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

pub mod duration {

    use chrono::Duration;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    use super::{format_duration, parse_duration};

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
        serializer.serialize_str(format_duration(value).as_str())
    }
}

#[cfg(test)]
mod tests {

    use super::parse_datetime;

    macro_rules! assert_parses {
        ($expr:expr) => {{
            if let None = parse_datetime($expr) {
                panic!(
                    "assertion failed: parse_datetime({}) should be Some(_) but is None",
                    stringify!($expr),
                );
            }
        }};
    }

    macro_rules! assert_parses_not {
        ($expr:expr) => {{
            if let Some(x) = parse_datetime($expr) {
                panic!(
                    "assertion failed: parse_datetime({}) should be None but is Some({})",
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
