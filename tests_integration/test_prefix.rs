use std::fs;

use crate::util::{git_last_hash, TestCommand};

// Simple run with prefix.
gashtest!(it_finds_a_prefix, |mut tcmd: TestCommand| {
  let prefix = "dead";
  let stdout = tcmd.args(&[prefix]).stdout();

  let expected = format!(
    "\
Searching for hash with prefix {prefix}
Found hash {prefix}{hash}
Patching last commit to include new hash... Success!
",
    prefix = prefix,
    hash = &git_last_hash(tcmd.dir())[prefix.len()..]
  );

  eqnice!(expected, stdout);
});

// Does not patch the commit with --dry-run.
gashtest!(dry_run_long_prefix, |mut tcmd: TestCommand| {
  let hash_before = git_last_hash(tcmd.dir());
  let stdout = tcmd.args(&["dead", "--dry-run"]).stdout();
  let hash_after = git_last_hash(tcmd.dir());

  assert_eq!(
    true,
    stdout.contains(&"Not amending commit due to --dry-run")
  );
  assert_eq!(hash_before, hash_after);
});

// Test fails if not run in a git repostiory.
gashtest!(it_does_not_work_outside_of_git, |mut tcmd: TestCommand| {
  fs::remove_dir_all(&tcmd.dir().join(".git")).unwrap();

  let expected = "\
Error: the command: 'git rev-parse HEAD' failed with:

fatal: not a git repository (or any parent up to mount point /)
Stopping at filesystem boundary (GIT_DISCOVERY_ACROSS_FILESYSTEM not set).
";
  eqnice!(expected, tcmd.args(&["dead"]).stderr());
});
