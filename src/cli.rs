use clap::Clap;

#[derive(Clap)]
#[clap(version = "1.0")]
pub struct Args {
  /// The desired prefix of the hash.
  /// TODO: parse and validate as hex
  pub prefix: String,

  #[clap(short = "p", long = "parallel")]
  pub parallel: bool,

  /// Whether or not to perform a dry run. This won't create a new repository,
  /// it will just run log out the generated pattern.
  #[clap(short = "d", long = "dry-run")]
  pub dry_run: bool,
}
