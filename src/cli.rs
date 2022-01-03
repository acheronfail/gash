use clap::Parser;
use clap::{crate_authors, crate_version};

const DEFAULT_MAX_VARIANCE: i64 = 3600;

#[derive(Parser, Debug)]
#[clap(version = crate_version!(), author = crate_authors!())]
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
    #[clap(short = 'p', long = "parallel")]
    parallel: bool,

    /// This field is used to cache the computed parallel so it's not re-computed
    /// each time that `.parallel()` is called.
    #[clap(skip)]
    _parallel: bool,

    /// The max distance (in seconds) gash can modify the commit times.
    /// Defaults to one hour.
    #[clap(short = 'm', long = "max-variance")]
    max_variance: Option<i64>,

    /// This field is used to cache the computed max_variance so it's not re-computed
    /// each time that `.max_variance()` is called.
    #[clap(skip)]
    _max_variance: i64,

    /// Whether or not to print progress. Note that this has a negative performance impact.
    /// Alternatively you may set "git config --global gash.progress true".
    #[clap(short = 'P', long = "progress")]
    progress: bool,

    /// This field is used to cache the computed progress so it's not re-computed
    /// each time that `.progress()` is called.
    #[clap(skip)]
    _progress: bool,

    /// Color text output when printing to the terminal.
    /// Alternatively you may set "git config --global gash.color true".
    #[clap(short = 'c', long = "color")]
    color: bool,

    /// This field is used to cache the computed color so it's not re-computed
    /// each time that `.color()` is called.
    #[clap(skip)]
    _color: bool,

    /// This will always attempt to find a new hash, even if the current commit
    /// already starts with the prefix.
    #[clap(short = 'f', long = "force")]
    pub force: bool,

    /// Whether or not to perform a dry run. This won't patch the latest commit,
    /// it will just print the hash.
    #[clap(short = 'd', long = "dry-run")]
    pub dry_run: bool,

    /// Output more information.
    #[clap(short = 'v', long = "verbose", parse(from_occurrences))]
    pub verbosity: usize,
}

impl Args {
    pub fn parse(get_config: fn(name: &str) -> Option<String>) -> Args {
        let mut args = <Args as Parser>::parse();

        let parse_bool = |val, name| {
            if val {
                true
            } else {
                match get_config(&format!("gash.{}", name)) {
                    Some(s) => s == "true",
                    None => false,
                }
            }
        };

        args._prefix = match &args.prefix {
            Some(prefix) => prefix.to_string(),
            None => get_config("gash.default")
                .expect("No prefix given and no value set for gash.default in git config"),
        };

        args._max_variance = match args.max_variance {
            Some(max_variance) => max_variance,
            None => get_config("gash.max-variance").map_or_else(
                || DEFAULT_MAX_VARIANCE,
                |s| {
                    s.parse::<i64>()
                        .expect("Failed to parse gash.max-variance as i64!")
                },
            ),
        };

        args._parallel = parse_bool(args.parallel, "parallel");
        args._progress = parse_bool(args.progress, "progress");
        args._color = parse_bool(args.color, "color");

        args
    }

    pub fn prefix(&self) -> String {
        String::from(&self._prefix)
    }

    pub fn max_variance(&self) -> i64 {
        self._max_variance
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
}
