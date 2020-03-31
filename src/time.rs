use std::fmt::{self, Display, Formatter};

/// A simple struct used to format seconds into a human readable representation.
pub struct TimeDelta(pub i64);

impl Display for TimeDelta {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let minutes = self.0.abs() / 60;
    let seconds = self.0.abs() % 60;
    let direction = if self.0 < 0 { "-" } else { "+" };

    write!(f, "{}{:2}:{:02}", direction, minutes, seconds)
  }
}

#[cfg(test)]
mod tests {
  use crate::time::TimeDelta;

  #[test]
  fn it_works() {
    let format_td = |n| format!("{}", TimeDelta(n));

    assert_eq!(format_td(10), "+ 0:10");
    assert_eq!(format_td(60), "+ 1:00");
    assert_eq!(format_td(90), "+ 1:30");
    assert_eq!(format_td(1337), "+22:17");
    assert_eq!(format_td(3599), "+59:59");
    assert_eq!(format_td(3600), "+60:00");

    assert_eq!(format_td(-10), "- 0:10");
    assert_eq!(format_td(-60), "- 1:00");
    assert_eq!(format_td(-90), "- 1:30");
    assert_eq!(format_td(-1337), "-22:17");
    assert_eq!(format_td(-3599), "-59:59");
    assert_eq!(format_td(-3600), "-60:00");
  }
}
