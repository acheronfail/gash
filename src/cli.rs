use clap::Clap;
use clap::{crate_authors, crate_version};

use crate::git;

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
struct ClapArgs {
  /// A hex string which is the desired prefix of the hash. If this is not
  /// provided then it defaults to "git config --global gash.default".
  pub prefix: Option<String>,

  /// Whether brute forcing the hash should be run in parallel.
  /// You may also set "git config --global gash.parallel true" as well.
  #[clap(short = "p", long = "parallel")]
  pub parallel: bool,

  /// The max distance (in seconds) gash can modify the commit times.
  /// Defaults to one hour.
  #[clap(short = "m", long = "max-variance")]
  pub max_variance: Option<i64>,

  /// Whether or not to perform a dry run. This won't create a new repository,
  /// it will just run log out the generated pattern.
  #[clap(short = "d", long = "dry-run")]
  pub dry_run: bool,
}

/// Public interface for `ClapArgs` which parses some values.
pub struct Args {
  pub prefix: String,
  pub parallel: bool,
  pub max_variance: i64,
  pub dry_run: bool,
}

impl Args {
  pub fn parse() -> Args {
    let args = ClapArgs::parse();

    let prefix = match args.prefix {
      Some(prefix) => prefix,
      None => git(&["config", "gash.default"])
        .expect("No prefix given and no value set for gash.default in git config"),
    };

    let parallel = match args.parallel {
      // If set via CLI, then honour that.
      true => true,
      // Otherwise, try and read git config
      false => match git(&["config", "gash.parallel"]) {
        Ok(s) => s == "true",
        _ => false,
      },
    };

    let max_variance = match args.max_variance {
      Some(max_variance) => max_variance,
      None => git(&["config", "gash.max-variance"]).map_or_else(
        |_| 3600,
        |s| {
          s.parse::<i64>()
            .expect("Failed to parse gash.max-variance as i64!")
        },
      ),
    };

    Args {
      prefix,
      parallel,
      max_variance,
      dry_run: args.dry_run,
    }
  }
}
