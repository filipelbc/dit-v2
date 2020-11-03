use chrono::{DateTime, Duration, Local, TimeZone};
use lazy_static::lazy_static;
use regex::{Captures, Regex};

lazy_static! {
    static ref DATETIME_RE: Regex = Regex::new(
        r"^(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})(-(?P<h>\d{2}):(?P<min>\d{2})(:(?P<s>\d{2}))?)?$"
    )
    .unwrap();
    static ref TIME_RE: Regex =
        Regex::new(r"^(?P<h>\d{2}):(?P<min>\d{2})(:(?P<s>\d{2}))?$").unwrap();
    static ref DELTA_RE: Regex = Regex::new(
        r"^((?P<d>[+-]?\d+)d)?((?P<h>[+-]?\d+)h)?((?P<min>[+-]?\d+)min)?((?P<s>[+-]?\d+)s)?$"
    )
    .unwrap();
}

pub type LocalDateTime = DateTime<Local>;

pub fn resolve(at: Option<&str>) -> Result<LocalDateTime, String> {
    match at {
        Some(x) => parse(x),
        None => Ok(Local::now()),
    }
}

fn parse(x: &str) -> Result<LocalDateTime, String> {
    match try_datetime(x).or(try_time(x)).or(try_delta(x)) {
        Some(d) => Ok(d),
        None => Err(format!("Invalid date/time value: {}", x)),
    }
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
