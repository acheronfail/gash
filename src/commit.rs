use std::io::{self, Write};
use std::process::Command;
use std::sync::{Arc, Mutex};

use hex;
use rayon::prelude::*;
use regex::{Captures, Regex};
use sha1::{Digest, Sha1};

use crate::time::TimeDelta;
use crate::{Args, Spiral};

const COMMIT_MESSAGE_RE: &str =
  r"(?m)^(?P<prefix>(?:author|committer).*> )(?P<timestamp>\d+)(?P<suffix>.*)$";

struct Match {
  pub source: String,
  pub prefix: String,
  pub suffix: String,
  pub timestamp: i64,
}

impl Match {
  pub fn new(capture: &Captures) -> Match {
    Match {
      source: capture[0].to_string(),
      prefix: capture["prefix"].to_string(),
      suffix: capture["suffix"].to_string(),
      timestamp: capture["timestamp"].parse::<i64>().unwrap(),
    }
  }

  pub fn with_timestamp(&self, timestamp: i64) -> String {
    format!("{}{}{}", self.prefix, timestamp, self.suffix)
  }
}

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
  pub commit_contents: String,
  /// How much the author timestamp has been changed.
  pub author_delta: TimeDelta,
  /// How much the committer timestamp has been changed.
  pub committer_delta: TimeDelta,
}

/// A struct which contains the commit message, and utilities to patch the message and hash it.
pub struct CommitTemplate {
  pub commit_contents: String,
  author_match: Match,
  committer_match: Match,
}

impl CommitTemplate {
  pub fn new() -> CommitTemplate {
    let output = Command::new("git")
      .args(&["cat-file", "-p", "HEAD"])
      .output()
      .expect("Failed to run git");

    if !output.status.success() {
      panic!("Failed to cat HEAD commit! {:?}", output);
    }

    // Extract the author and committer timestamps.
    let commit_contents = String::from_utf8_lossy(&output.stdout).to_string();
    let re = Regex::new(COMMIT_MESSAGE_RE).unwrap();
    let captures = re.captures_iter(&commit_contents).collect::<Vec<_>>();

    CommitTemplate {
      author_match: Match::new(&captures[0]),
      committer_match: Match::new(&captures[1]),
      commit_contents,
    }
  }

  /// Format the string as a template string.
  pub fn with_diff(&self, author_diff: i64, committer_diff: i64) -> String {
    self
      .commit_contents
      .replace(
        &self.author_match.source,
        &self
          .author_match
          .with_timestamp(self.author_match.timestamp + author_diff),
      )
      .replace(
        &self.committer_match.source,
        &self
          .committer_match
          .with_timestamp(self.committer_match.timestamp + committer_diff),
      )
      .to_string()
  }

  pub fn brute_force_sha1(&self, args: &Args) -> Option<BruteForceResult> {
    let state = Arc::new(Mutex::new(BruteForceState::new()));

    let prefix = &args.prefix();
    let padding = if args.verbosity > 0 { "        " } else { "" };
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
      let hash = hex::encode(hasher.result());

      // Check if the hash starts with our prefix.
      match hash.starts_with(prefix) {
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
            sha1: hash,
            commit_contents: new_commit,
            author_delta: TimeDelta(da),
            committer_delta: TimeDelta(dc),
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
