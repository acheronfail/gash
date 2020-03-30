use std::collections::HashSet;

use clap::Clap;
use clap::{crate_authors, crate_version};

#[derive(Clap, Debug)]
#[clap(version = crate_version!(), author = crate_authors!())]
pub struct Args {
  /// A hex string which is the desired prefix of the hash. If this is not
  /// provided then it defaults to "git config --global gash.default".
  prefix: Option<String>,

  /// This field is used to cache the computed prefix so it's not re-computed each
  /// time that `.prefix()` is called.
  #[clap(skip)]
  _prefix: String,

  /// Whether brute forcing the hash should be run in parallel.
  /// You may also set "git config --global gash.parallel true" as well.
  #[clap(short = "p", long = "parallel")]
  parallel: bool,

  /// This field is used to cache the computed parallel so it's not re-computed
  /// each time that `.parallel()` is called.
  #[clap(skip)]
  _parallel: bool,

  /// The max distance (in seconds) gash can modify the commit times.
  /// Defaults to one hour.
  #[clap(short = "m", long = "max-variance")]
  max_variance: Option<i64>,

  /// This field is used to cache the computed max_variance so it's not re-computed
  /// each time that `.max_variance()` is called.
  #[clap(skip)]
  _max_variance: i64,

  /// Whether or not to print progress. Note that this has a negative performance impact.
  #[clap(short = "P", long = "progress")]
  progress: bool,

  /// This field is used to cache the computed progress so it's not re-computed
  /// each time that `.progress()` is called.
  #[clap(skip)]
  _progress: bool,

  /// Whether or not to perform a dry run. This won't create a new repository,
  /// it will just run log out the generated pattern.
  #[clap(short = "d", long = "dry-run")]
  pub dry_run: bool,
}

impl Args {
  pub fn parse(git_config: fn(name: &str) -> Option<String>) -> Args {
    let mut args = <Args as Clap>::parse();

    args._prefix = match &args.prefix {
      Some(prefix) => prefix.to_string(),
      None => git_config("gash.default")
        .expect("No prefix given and no value set for gash.default in git config"),
    };

    if !Args::validate_hex(&args._prefix) {
      panic!(
        "The prefix must only contain hex characters! Got: {}",
        &args._prefix
      );
    }

    args._parallel = match args.parallel {
      true => true,
      false => match git_config("gash.parallel") {
        Some(s) => s == "true",
        None => false,
      },
    };

    args._progress = match args.progress {
      true => true,
      false => match git_config("gash.progress") {
        Some(s) => s == "true",
        None => false,
      },
    };

    args._max_variance = match args.max_variance {
      Some(max_variance) => max_variance,
      None => git_config("gash.max-variance").map_or_else(
        || 3600,
        |s| {
          s.parse::<i64>()
            .expect("Failed to parse gash.max-variance as i64!")
        },
      ),
    };

    args
  }

  pub fn prefix(&self) -> String {
    String::from(&self._prefix)
  }

  pub fn parallel(&self) -> bool {
    self._parallel
  }

  pub fn progress(&self) -> bool {
    self._progress
  }

  pub fn max_variance(&self) -> i64 {
    self._max_variance
  }

  fn validate_hex(s: &str) -> bool {
    let mut valid_chars = HashSet::new();
    valid_chars.insert('0');
    valid_chars.insert('1');
    valid_chars.insert('2');
    valid_chars.insert('3');
    valid_chars.insert('4');
    valid_chars.insert('5');
    valid_chars.insert('6');
    valid_chars.insert('7');
    valid_chars.insert('8');
    valid_chars.insert('9');
    valid_chars.insert('a');
    valid_chars.insert('b');
    valid_chars.insert('c');
    valid_chars.insert('d');
    valid_chars.insert('e');
    valid_chars.insert('f');

    for c in s.chars() {
      if !valid_chars.contains(&c) {
        return false;
      }
    }

    return true;
  }
}
