use std::fs;

use pretty_assertions::assert_eq;

use crate::util::{git_last_hash, TestCommand};

const PREFIX: &str = "1337";

// Simple run with prefix.
gashtest!(finds_prefix_and_patches_commit, |mut tcmd: TestCommand| {
    let stdout = tcmd.args(&[PREFIX]).stdout();

    let expected = format!(
        "\
Searching for hash with prefix {prefix}
Found hash {prefix}{hash}
Patching last commit to include new hash... Success!
",
        prefix = PREFIX,
        hash = &git_last_hash(tcmd.dir())[PREFIX.len()..]
    );

    assert_eq!(expected, stdout);
});

// Works with a prefix of odd length.
gashtest!(allows_prefixes_of_odd_length, |mut tcmd: TestCommand| {
    let odd_prefix = "123";
    let stdout = tcmd.args(&[odd_prefix]).stdout();

    let expected = format!(
        "\
Searching for hash with prefix {prefix}
Found hash {prefix}{hash}
Patching last commit to include new hash... Success!
",
        prefix = odd_prefix,
        hash = &git_last_hash(tcmd.dir())[odd_prefix.len()..]
    );

    assert_eq!(expected, stdout);
});

// Does not allow non-hex characters as prefix.
gashtest!(does_not_allow_non_hex_chars, |mut tcmd: TestCommand| {
    let bad_prefix = "hello";
    let stderr = tcmd.args(&[bad_prefix]).stderr();
    let expected = format!(
        "\
The prefix must only contain hex characters! Got: {}
",
        bad_prefix
    );
    assert_eq!(expected, stderr);
});

// Does not patch the commit with --dry-run.
gashtest!(dry_run_long_prefix, |mut tcmd: TestCommand| {
    let hash_before = git_last_hash(tcmd.dir());
    let stderr = tcmd.args(&[PREFIX, "--dry-run"]).stderr();
    let hash_after = git_last_hash(tcmd.dir());

    assert_eq!(
        true,
        stderr.contains(&"Not amending commit due to --dry-run")
    );
    assert_eq!(hash_before, hash_after);
});

// Should fail if not run in a git repostiory.
gashtest!(it_does_not_work_outside_of_git, |mut tcmd: TestCommand| {
    fs::remove_dir_all(&tcmd.dir().join(".git")).unwrap();

    let expected = "\
Error: the command: 'git rev-parse HEAD' failed with:

fatal: not a git repository (or any parent up to mount point /)
Stopping at filesystem boundary (GIT_DISCOVERY_ACROSS_FILESYSTEM not set).
";
    assert_eq!(expected, tcmd.args(&[PREFIX]).stderr());
});
