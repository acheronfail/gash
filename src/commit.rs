use std::io::{self, Write};
use std::process::Command;
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
  pub commit_contents: String,
  /// How much the author timestamp has been changed.
  pub author_delta: TimeDelta,
  /// How much the committer timestamp has been changed.
  pub committer_delta: TimeDelta,
}

/// A struct which contains the commit message, and utilities to patch the message and hash it.
pub struct CommitTemplate {
  pub commit_contents: String,
  a_timestamp: i64,
  c_timestamp: i64,

  a_pos: (usize, usize),
  c_pos: (usize, usize),
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
    let timestamp_and_pos = |c: &Captures| {
      let m = c.get(2).unwrap();
      (
        m.as_str()
          .parse::<i64>()
          .expect("Failed to parse timestamp"),
        (m.start(), m.end()),
      )
    };

    let (a_timestamp, a_pos) = timestamp_and_pos(&captures[0]);
    let (c_timestamp, c_pos) = timestamp_and_pos(&captures[1]);

    CommitTemplate {
      a_pos,
      c_pos,
      a_timestamp,
      c_timestamp,
      commit_contents,
    }
  }

  /// Build a new commit with the patched timestamps.
  pub fn with_diff(&self, da: i64, cd: i64) -> String {
    let mut text = String::with_capacity(self.commit_contents.len());
    text.push_str(&self.commit_contents[..self.a_pos.0]);
    text.push_str(&format!("{}", self.a_timestamp + da));
    text.push_str(&self.commit_contents[self.a_pos.1..self.c_pos.0]);
    text.push_str(&format!("{}", self.c_timestamp + cd));
    text.push_str(&self.commit_contents[self.c_pos.1..]);

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
