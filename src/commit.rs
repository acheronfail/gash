use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use rayon::prelude::*;
use regex::{Captures, Regex};
use sha1::{Digest, Sha1};

use crate::hash::create_validator;
use crate::time::TimeDelta;
use crate::{Args, Spiral};

const COMMIT_MESSAGE_RE: &str =
  r"(?m)^(?P<prefix>(?:author|committer).*> )(?P<timestamp>\d+)(?P<suffix>.*)$";

/// Simple struct used to count iterations and track hash progress.
struct BruteForceState {
  count: usize,
  found: bool,
}

impl BruteForceState {
  fn new() -> BruteForceState {
    BruteForceState {
      count: 0,
      found: false,
    }
  }
}

/// Simple struct to return the results of our brute force search.
pub struct BruteForceResult {
  /// The new brute forced hash.
  pub sha1: String,
  /// The commit that created the hash.
  pub patched_commit: String,
  /// How much the author timestamp has been changed.
  pub da: TimeDelta,
  /// How much the committer timestamp has been changed.
  pub dc: TimeDelta,
}

struct Timestamp {
  pub val: i64,
  pub pos: (usize, usize),
}

impl Timestamp {
  pub fn new(cap: &Captures) -> Timestamp {
    let cap_match = cap
      .get(2)
      .expect("Failed to extract match from capturing group");

    Timestamp {
      val: cap_match
        .as_str()
        .parse::<i64>()
        .expect("Failed to parse timestamp from commit"),
      pos: (cap_match.start(), cap_match.end()),
    }
  }
}

/// A struct which contains the commit message, and utilities to patch the message and hash it.
pub struct Commit<'a> {
  commit: &'a str,

  a_timestamp: Timestamp,
  c_timestamp: Timestamp,
}

impl<'a> Commit<'a> {
  pub fn new(commit: &'a impl AsRef<str>) -> Commit<'a> {
    let commit = commit.as_ref();

    // Extract the timestamps and their locations from the commit.
    let re = Regex::new(COMMIT_MESSAGE_RE).unwrap();
    let captures = re.captures_iter(&commit).collect::<Vec<_>>();

    Commit {
      a_timestamp: Timestamp::new(&captures[0]),
      c_timestamp: Timestamp::new(&captures[1]),
      commit,
    }
  }

  /// Build a new commit with the patched timestamps.
  pub fn with_diff(&self, da: i64, cd: i64) -> String {
    let mut text = String::with_capacity(self.commit.len());
    text.push_str(&self.commit[..self.a_timestamp.pos.0]);
    text.push_str(&format!("{}", self.a_timestamp.val + da));
    text.push_str(&self.commit[self.a_timestamp.pos.1..self.c_timestamp.pos.0]);
    text.push_str(&format!("{}", self.c_timestamp.val + cd));
    text.push_str(&self.commit[self.c_timestamp.pos.1..]);

    text
  }

  pub fn brute_force_sha1(&self, args: &Args) -> Option<BruteForceResult> {
    let prefix = &args.prefix();
    let hash_is_correct = create_validator(prefix);

    let padding = if args.verbosity > 0 { "        " } else { "" };
    let state = Arc::new(Mutex::new(BruteForceState::new()));
    let mapper = |(da, dc)| {
      // Update progress.
      if args.progress() {
        let mut state = state.lock().unwrap();
        state.count += 1;

        if state.found {
          return None;
        }

        if state.count % 1000 == 0 {
          print!("\r{}hashes {}k", padding, state.count / 1000);
          io::stdout().flush().unwrap();
        }
      }

      // Patch the commit.
      let new_commit = self.with_diff(da, dc);

      // Hash the commit.
      let mut hasher = Sha1::new();
      hasher.input(&format!("commit {}\0{}", new_commit.len(), new_commit));

      // Check if the hash starts with our prefix.
      let hash = hasher.result();
      match hash_is_correct(hash) {
        // We found a match!
        true => {
          if args.progress() {
            // Update the state (this stops other parallel threads from logging after we've already found something).
            let mut state = state.lock().unwrap();
            state.found = true;

            // Move the terminal cursor to the next line.
            println!("");
          }

          // Return our result.
          Some(BruteForceResult {
            sha1: hex::encode(hash),
            patched_commit: new_commit,
            da: TimeDelta(da),
            dc: TimeDelta(dc),
          })
        }

        // Keep looking.
        false => None,
      }
    };

    let spiral = Spiral::new(args.max_variance());
    if args.parallel() {
      spiral.par_iter().find_map_any(mapper)
    } else {
      spiral.iter().find_map(mapper)
    }
  }
}
