use std::fmt::{self, Display, Formatter};

use chrono::{DateTime, Duration, FixedOffset};

/// A simple struct used to format seconds into a human readable representation.
pub struct TimeDelta {
    original: DateTime<FixedOffset>,
    diff_secs: i64,
}

impl TimeDelta {
    pub fn new(original: DateTime<FixedOffset>, diff_secs: i64) -> TimeDelta {
        TimeDelta {
            original,
            diff_secs,
        }
    }
}

impl Display for TimeDelta {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let minutes = self.diff_secs.abs() / 60;
        let seconds = self.diff_secs.abs() % 60;
        let direction = if self.diff_secs < 0 { "-" } else { "+" };

        let dt = self.original + Duration::seconds(self.diff_secs);
        write!(
            f,
            "{}{:2}:{:02} ({})",
            direction,
            minutes,
            seconds,
            dt.to_rfc2822()
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, FixedOffset};

    use crate::time::TimeDelta;

    fn tdstr(dt: DateTime<FixedOffset>, diff: i64) -> String {
        format!("{}", TimeDelta::new(dt, diff))
    }

    #[test]
    fn it_works_with_positive_diffs() {
        let dt = DateTime::parse_from_str("1585818825 +1100", "%s %z").unwrap();
        assert_eq!(tdstr(dt, 10), "+ 0:10 (Thu, 02 Apr 2020 20:13:55 +1100)");
        assert_eq!(tdstr(dt, 60), "+ 1:00 (Thu, 02 Apr 2020 20:14:45 +1100)");
        assert_eq!(tdstr(dt, 90), "+ 1:30 (Thu, 02 Apr 2020 20:15:15 +1100)");
        assert_eq!(tdstr(dt, 1337), "+22:17 (Thu, 02 Apr 2020 20:36:02 +1100)");
        assert_eq!(tdstr(dt, 3599), "+59:59 (Thu, 02 Apr 2020 21:13:44 +1100)");
        assert_eq!(tdstr(dt, 3600), "+60:00 (Thu, 02 Apr 2020 21:13:45 +1100)");
    }

    #[test]
    fn it_works_with_negative_diffs() {
        let dt = DateTime::parse_from_str("1585818825 +1100", "%s %z").unwrap();
        assert_eq!(tdstr(dt, -10), "- 0:10 (Thu, 02 Apr 2020 20:13:35 +1100)");
        assert_eq!(tdstr(dt, -60), "- 1:00 (Thu, 02 Apr 2020 20:12:45 +1100)");
        assert_eq!(tdstr(dt, -90), "- 1:30 (Thu, 02 Apr 2020 20:12:15 +1100)");
        assert_eq!(tdstr(dt, -1337), "-22:17 (Thu, 02 Apr 2020 19:51:28 +1100)");
        assert_eq!(tdstr(dt, -3599), "-59:59 (Thu, 02 Apr 2020 19:13:46 +1100)");
        assert_eq!(tdstr(dt, -3600), "-60:00 (Thu, 02 Apr 2020 19:13:45 +1100)");
    }

    #[test]
    fn it_works_with_different_timezones() {
        let dt = DateTime::parse_from_str("1585818825 +0000", "%s %z").unwrap();
        assert_eq!(tdstr(dt, 10), "+ 0:10 (Thu, 02 Apr 2020 09:13:55 +0000)");

        let dt = DateTime::parse_from_str("1585818825 -0500", "%s %z").unwrap();
        assert_eq!(tdstr(dt, 10), "+ 0:10 (Thu, 02 Apr 2020 04:13:55 -0500)");
    }
}
