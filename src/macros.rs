// A simpler and more terse way to change terminal colors.
#[macro_export]
macro_rules! p {
  ($stdout:ident, None) => {
      $stdout.set_color(ColorSpec::new().set_fg(None))?;
  };
  ($stdout:ident, $color:ident) => {
      $stdout.set_color(ColorSpec::new().set_fg(Some(Color::$color)))?;
  };

  ($stdout:ident, None, $fmt:expr) => {
      $stdout.set_color(ColorSpec::new().set_fg(None))?;
      write!(&mut $stdout, $fmt)?;
  };
  ($stdout:ident, $color:ident, $fmt:expr) => {
      $stdout.set_color(ColorSpec::new().set_fg(Some(Color::$color)))?;
      write!(&mut $stdout, $fmt)?;
  };

  ($stdout:ident, None, $fmt:expr, $( $fmt_arg:expr ),*) => {
      $stdout.set_color(ColorSpec::new().set_fg(None))?;
      write!(&mut $stdout, $fmt, $($fmt_arg),*)?;
  };
  ($stdout:ident, $color:ident, $fmt:expr, $( $fmt_arg:expr ),*) => {
      $stdout.set_color(ColorSpec::new().set_fg(Some(Color::$color)))?;
      write!(&mut $stdout, $fmt, $($fmt_arg),*)?;
  };
}
