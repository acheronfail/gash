use clap::Clap;
use clap::{crate_authors, crate_version};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
pub struct Args {
  /// A hex string which is the desired prefix of the hash. If this is not
  /// provided then it defaults to "git config --global gash.default".
  pub prefix: Option<String>,

  /// Whether brute forcing the hash should be run in parallel.
  /// You may also set "git config --global gash.parallel true" as well.
  #[clap(short = "p", long = "parallel")]
  pub parallel: bool,

  /// Whether or not to perform a dry run. This won't create a new repository,
  /// it will just run log out the generated pattern.
  #[clap(short = "d", long = "dry-run")]
  pub dry_run: bool,
}
