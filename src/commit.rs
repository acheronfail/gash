use std::process::Command;

use crypto::digest::Digest;
use crypto::sha1::Sha1;
use rayon::prelude::*;
use regex::{Captures, Regex};

use crate::Spiral;

const SPIRAL_MAX: i64 = 3600;
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

pub struct BruteForceResult {
  pub sha1: String,
  pub commit_contents: String,
  pub author_timestamp_delta: i64,
  pub committer_timestamp_delta: i64,
}

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

  pub fn brute_force_sha1<S: AsRef<str>>(
    &self,
    prefix: S,
    parallel: bool,
  ) -> Option<BruteForceResult> {
    let prefix = prefix.as_ref();
    let mapper = |(da, dc)| {
      let new_commit = self.with_diff(da, dc);

      let mut hasher = Sha1::new();
      hasher.input_str(&format!("commit {}\0{}", new_commit.len(), new_commit));
      let hash = hasher.result_str();
      if hash.starts_with(prefix) {
        Some(BruteForceResult {
          sha1: hash,
          commit_contents: new_commit,
          author_timestamp_delta: da,
          committer_timestamp_delta: dc,
        })
      } else {
        None
      }
    };

    let spiral = Spiral::new(SPIRAL_MAX);
    if parallel {
      spiral.par_iter().find_map_any(mapper)
    } else {
      spiral.iter().find_map(mapper)
    }
  }
}
