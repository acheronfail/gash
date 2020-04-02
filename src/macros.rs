// A simpler and more terse way to change terminal colors.

#[macro_export]
macro_rules! p {
    // Single value: just changes the color.
    ($stdout:ident, None) => {
        $stdout.set_color(termcolor::ColorSpec::new().set_fg(None))?;
    };
    ($stdout:ident, $color:ident) => {
        $stdout.set_color(termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::$color)))?;
    };

    // Two values: change the color and print string.
    ($stdout:ident, None, $fmt:expr) => {
        $stdout.set_color(termcolor::ColorSpec::new().set_fg(None))?;
        write!(&mut $stdout, $fmt)?;
    };
    ($stdout:ident, $color:ident, $fmt:expr) => {
        $stdout.set_color(termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::$color)))?;
        write!(&mut $stdout, $fmt)?;
    };

    // Three or more values: change the color and format a string.
    ($stdout:ident, None, $fmt:expr, $( $fmt_arg:expr ),*) => {
        $stdout.set_color(termcolor::ColorSpec::new().set_fg(None))?;
        write!(&mut $stdout, $fmt, $($fmt_arg),*)?;
    };
    ($stdout:ident, $color:ident, $fmt:expr, $( $fmt_arg:expr ),*) => {
        $stdout.set_color(termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::$color)))?;
        write!(&mut $stdout, $fmt, $($fmt_arg),*)?;
    };
}
