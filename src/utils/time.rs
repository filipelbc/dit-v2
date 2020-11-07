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
    static ref DELTA_RE: Regex = Regex::new(
        r"^((?P<d>[+-]?\d+)d)?((?P<h>[+-]?\d+)h)?((?P<min>[+-]?\d+)min)?((?P<s>[+-]?\d+)s)?$"
    )
    .unwrap();
}

pub type LocalDateTime = DateTime<Local>;

pub fn resolve(at: Option<&str>) -> Result<LocalDateTime> {
    match at {
        Some(x) => parse(x),
        None => Ok(Local::now()),
    }
}

fn parse(x: &str) -> Result<LocalDateTime> {
    try_datetime(x)
        .or(try_time(x))
        .or(try_delta(x))
        .with_context(|| format!("Invalid date/time value: {}", x))
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

fn try_delta(x: &str) -> Option<LocalDateTime> {
    DELTA_RE.captures(x).map(|m| {
        let s = i(&m, "h") * 3600 + i(&m, "min") * 60 + i(&m, "s");
        Local::now() + Duration::seconds(i64::from(s))
    })
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

#[cfg(test)]
mod tests {

    use super::parse;

    macro_rules! assert_parses {
        ($expr:expr) => {{
            if let Err(e) = parse($expr) {
                panic!(
                    "assertion failed: parse({}) should be Ok(_) but is Err({:?})",
                    stringify!($expr),
                    e,
                );
            }
        }};
    }

    macro_rules! assert_parses_not {
        ($expr:expr) => {{
            if let Ok(e) = parse($expr) {
                panic!(
                    "assertion failed: parse({}) should be Err(_) but is Ok({:?})",
                    stringify!($expr),
                    e,
                );
            }
        }};
    }

    #[test]
    fn test_parse() {
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
