use clap::Clap;
use clap::{crate_authors, crate_version};

use crate::git;

#[derive(Clap, Debug)]
#[clap(version = crate_version!(), author = crate_authors!())]
pub struct Args {
  /// A hex string which is the desired prefix of the hash. If this is not
  /// provided then it defaults to "git config --global gash.default".
  prefix: Option<String>,

  /// This field is used to cache the computed prefix so it's not computed each
  /// time that `.prefix()` is called.
  #[clap(skip)]
  _prefix: String,

  /// Whether brute forcing the hash should be run in parallel.
  /// You may also set "git config --global gash.parallel true" as well.
  #[clap(short = "p", long = "parallel")]
  parallel: bool,

  /// This field is used to cache the computed parallel so it's not computed
  /// each time that `.parallel()` is called.
  #[clap(skip)]
  _parallel: bool,

  /// The max distance (in seconds) gash can modify the commit times.
  /// Defaults to one hour.
  #[clap(short = "m", long = "max-variance")]
  max_variance: Option<i64>,

  /// This field is used to cache the computed max_variance so it's not computed
  /// each time that `.max_variance()` is called.
  #[clap(skip)]
  _max_variance: i64,

  /// Whether or not to perform a dry run. This won't create a new repository,
  /// it will just run log out the generated pattern.
  #[clap(short = "d", long = "dry-run")]
  pub dry_run: bool,
}

impl Args {
  pub fn parse() -> Args {
    let mut args = <Args as Clap>::parse();

    args._prefix = match &args.prefix {
      Some(prefix) => prefix.to_string(),
      None => git(&["config", "gash.default"])
        .expect("No prefix given and no value set for gash.default in git config"),
    };

    args._parallel = match args.parallel {
      // If set via CLI, then honour that.
      true => true,
      // Otherwise, try and read git config
      false => match git(&["config", "gash.parallel"]) {
        Ok(s) => s == "true",
        _ => false,
      },
    };

    args._max_variance = match args.max_variance {
      Some(max_variance) => max_variance,
      None => git(&["config", "gash.max-variance"]).map_or_else(
        |_| 3600,
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

  pub fn max_variance(&self) -> i64 {
    self._max_variance
  }
}
