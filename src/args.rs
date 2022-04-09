use anyhow::{bail, Result};
use clap::Parser;
use clap::{crate_authors, crate_version};

const DEFAULT_MAX_VARIANCE: i64 = 3600;

#[derive(Parser, Debug)]
#[clap(version = crate_version!(), author = crate_authors!())]
pub struct Args {
    /// A hex string which is the desired prefix (or suffix if --stealth is used) of the hash.
    /// If this is not provided then it defaults to "git config --global gash.default".
    /// Avoid using strings greater than four characters long, since the brute-forcing time increases
    /// exponentially.
    ///
    /// Pass the special value "hook" to install a git hook in the current repository.
    signature: Option<String>,

    /// This field is used to cache the computed signature so it's not re-computed each
    /// time that `.signature()` is called.
    #[clap(skip)]
    _signature: String,

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

    /// Stealth mode! Leave your mark as a suffix on the rather rather than a prefix!
    /// Alternatively you may set "git config --global gash.stealth true".
    #[clap(short = 's', long = "stealth")]
    stealth: bool,

    /// This field is used to cache the computed stealth so it's not re-computed
    /// each time that `.stealth()` is called.
    #[clap(skip)]
    _stealth: bool,

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
    pub fn parse(get_config: fn(name: &str) -> Option<String>) -> Result<Args> {
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

        args._signature = match &args.signature {
            Some(prefix) => prefix.to_string(),
            None => match get_config("gash.default") {
                Some(s) => s,
                None => bail!("No signature given and no value set for gash.default in git config"),
            },
        };
        // SHA1 hashes are 40 characters, so it doesn't make sense if the signature is longer!
        if args._signature.len() > 40 {
            bail!("Signature cannot exceed 40 characters in length!");
        }
        // Validate signature characters
        if args._signature != "hook"
            && args
                ._signature
                .chars()
                .any(|ch| !matches!(ch, '0'..='9' | 'a'..='f'))
        {
            bail!(
                "Signature may only contain [a-z0-9] characters! Got: {}",
                args._signature
            );
        }

        args._max_variance = match args.max_variance {
            Some(max_variance) => max_variance,
            None => match get_config("gash.max-variance")
                .map_or_else(|| Ok(DEFAULT_MAX_VARIANCE), |s| s.parse::<i64>())
            {
                Ok(n) => n,
                Err(e) => bail!("Failed to parse gash.max-variance as i64! Error: {}", e),
            },
        };

        args._parallel = parse_bool(args.parallel, "parallel");
        args._progress = parse_bool(args.progress, "progress");
        args._color = parse_bool(args.color, "color");
        args._stealth = parse_bool(args.stealth, "stealth");

        Ok(args)
    }

    pub fn signature(&self) -> String {
        String::from(&self._signature)
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

    pub fn stealth(&self) -> bool {
        self._stealth
    }
}
