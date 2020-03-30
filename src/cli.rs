use std::collections::HashSet;
use std::fs::{set_permissions, OpenOptions};
use std::io::Write;
use std::process;

use clap::AppSettings::ColoredHelp;
use clap::Clap;
use clap::{crate_authors, crate_version};

#[derive(Clap, Debug)]
#[clap(version = crate_version!(), author = crate_authors!(), global_setting(ColoredHelp))]
pub struct Args {
  /// A hex string which is the desired prefix of the hash. If this is not provided then it defaults
  /// to "git config --global gash.default".
  /// Avoid using strings greater than four characters long, since the brute-forcing time increases
  /// exponentially.
  ///
  /// Pass the special value "hook" to install a git hook in the current repository.
  prefix: Option<String>,

  /// This field is used to cache the computed prefix so it's not re-computed each
  /// time that `.prefix()` is called.
  #[clap(skip)]
  _prefix: String,

  /// Whether brute forcing the hash should be run in parallel.
  /// Alternatively you may set "git config --global gash.parallel true".
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
  /// Alternatively you may set "git config --global gash.progress true".
  #[clap(short = "P", long = "progress")]
  progress: bool,

  /// This field is used to cache the computed progress so it's not re-computed
  /// each time that `.progress()` is called.
  #[clap(skip)]
  _progress: bool,

  /// Color text output when printing to the terminal.
  /// Alternatively you may set "git config --global gash.color true".
  #[clap(short = "c", long = "color")]
  color: bool,

  /// This field is used to cache the computed color so it's not re-computed
  /// each time that `.color()` is called.
  #[clap(skip)]
  _color: bool,

  /// Whether or not to perform a dry run. This won't patch the latest commit,
  /// it will just print the hash.
  #[clap(short = "d", long = "dry-run")]
  pub dry_run: bool,

  /// Output more information.
  #[clap(short = "v", long = "verbose", parse(from_occurrences))]
  pub verbosity: usize,
}

impl Args {
  pub fn parse(git_config: fn(name: &str) -> Option<String>) -> Args {
    let mut args = <Args as Clap>::parse();

    args._prefix = match &args.prefix {
      Some(prefix) => prefix.to_string(),
      None => git_config("gash.default")
        .expect("No prefix given and no value set for gash.default in git config"),
    };

    // Validate the prefix.
    if !Args::validate_hex(&args._prefix) {
      match &args._prefix[..] {
        "hook" => {
          create_git_hook().expect("Failed to create git hook");
          process::exit(0);
        }
        _ => {
          println!(
            "The prefix must only contain hex characters! Got: {}",
            &args._prefix
          );
          process::exit(1);
        }
      }
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

    args._color = match args.color {
      true => true,
      false => match git_config("gash.color") {
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

  pub fn color(&self) -> bool {
    self._color
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

fn create_git_hook() -> std::io::Result<()> {
  let post_commit_hook = ".git/hooks/post-commit";
  println!("Adding git hook to {}", post_commit_hook);

  // Create file.
  let mut file = OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(true)
    .open(post_commit_hook)?;

  // Make it executable on unix.
  if cfg!(unix) {
    use std::os::unix::fs::PermissionsExt;

    let mut perms = file.metadata()?.permissions();
    perms.set_mode(0o755);
    set_permissions(post_commit_hook, perms)?;
  }

  // Write the hook.
  file.write_fmt(format_args!("{}", "#!/bin/bash\ngash"))?;

  Ok(())
}
