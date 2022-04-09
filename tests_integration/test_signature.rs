use std::fs;

use pretty_assertions::assert_eq;

use crate::util::{git_last_hash, TestCommand, git};

const SIGNATURE: &str = "1337";

// Simple run (prefix).
gashtest!(finds_prefix_and_patches_commit, |mut tcmd: TestCommand| {
    let stdout = tcmd.args(&[SIGNATURE]).stdout();
    let expected = format!(
        "\
Searching for hash with prefix {prefix}
Found hash {prefix}{hash}
Patching last commit to include new hash... Success!
",
        prefix = SIGNATURE,
        hash = &git_last_hash(tcmd.dir())[SIGNATURE.len()..]
    );
    assert_eq!(expected, stdout);
});

// Simple run (suffix).
gashtest!(finds_suffix_and_patches_commit, |mut tcmd: TestCommand| {
    let stdout = tcmd.args(&[SIGNATURE, "--stealth"]).stdout();
    let last_hash = git_last_hash(tcmd.dir());
    println!("====> {:?}", &last_hash);
    println!(
        "====> {:?}",
        &last_hash[..last_hash.len() - SIGNATURE.len()]
    );
    let expected = format!(
        "\
Searching for hash with suffix {suffix}
Found hash {hash}{suffix}
Patching last commit to include new hash... Success!
",
        suffix = SIGNATURE,
        hash = &last_hash[..last_hash.len() - SIGNATURE.len()]
    );
    assert_eq!(expected, stdout);
});

// Works with a signature of odd length (prefix).
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

// Works with a signature of odd length (suffix).
gashtest!(allows_suffixes_of_odd_length, |mut tcmd: TestCommand| {
    let odd_suffix = "123";
    let stdout = tcmd.args(&[odd_suffix, "--stealth"]).stdout();
    let last_hash = git_last_hash(tcmd.dir());
    let expected = format!(
        "\
Searching for hash with suffix {suffix}
Found hash {hash}{suffix}
Patching last commit to include new hash... Success!
",
        suffix = odd_suffix,
        hash = &last_hash[..last_hash.len() - odd_suffix.len()]
    );
    assert_eq!(expected, stdout);
});

// Does nothing if hash already has signature (prefix).
gashtest!(noop_if_has_prefix, |mut tcmd: TestCommand| {
    tcmd.args(&[SIGNATURE]);
    // Perform once to give hash the signature.
    let _ = tcmd.stdout();
    // A second time to noop.
    let stdout = tcmd.stdout();

    let last_hash = git_last_hash(tcmd.dir());
    let expected = format!(
        "Nothing to do, current hash: {prefix}{hash}\n",
        prefix = SIGNATURE,
        hash = &last_hash[SIGNATURE.len()..]
    );
    assert_eq!(expected, stdout);
});
// Does nothing if hash already has signature (suffix).
gashtest!(noop_if_has_suffix, |mut tcmd: TestCommand| {
    tcmd.args(&[SIGNATURE, "--stealth"]);
    // Perform once to give hash the signature.
    let _ = tcmd.stdout();
    // A second time to noop.
    let stdout = tcmd.stdout();

    let last_hash = git_last_hash(tcmd.dir());
    let expected = format!(
        "Nothing to do, current hash: {hash}{suffix}\n",
        suffix = SIGNATURE,
        hash = &last_hash[..last_hash.len() - SIGNATURE.len()]
    );
    assert_eq!(expected, stdout);
});

// Recomputes if asking for prefix but has suffix
gashtest!(want_prefix_but_has_suffix, |mut tcmd: TestCommand| {
    tcmd.args(&[SIGNATURE]);

    // Give hash a suffix
    git(tcmd.dir(), &["config", "gash.stealth", "true"]).unwrap();
    let _ = tcmd.stdout();
    git(tcmd.dir(), &["config", "gash.stealth", "false"]).unwrap();
    // Now, try for a prefix
    let stdout = tcmd.stdout();

    let last_hash = git_last_hash(tcmd.dir());
    let expected = format!(
        "\
Searching for hash with prefix {prefix}
Found hash {prefix}{hash}
Patching last commit to include new hash... Success!
",
        prefix = SIGNATURE,
        hash = &last_hash[SIGNATURE.len()..]
    );
    assert_eq!(expected, stdout);
});
// Recomputes if asking for suffix but has prefix
gashtest!(want_suffix_but_has_prefix, |mut tcmd: TestCommand| {
    tcmd.args(&[SIGNATURE]);

    // Give hash a prefix
    git(tcmd.dir(), &["config", "gash.stealth", "false"]).unwrap();
    let _ = tcmd.stdout();
    git(tcmd.dir(), &["config", "gash.stealth", "true"]).unwrap();
    // Now, try for a suffix
    let stdout = tcmd.stdout();

    let last_hash = git_last_hash(tcmd.dir());
    let expected = format!(
        "\
Searching for hash with suffix {suffix}
Found hash {hash}{suffix}
Patching last commit to include new hash... Success!
",
        suffix = SIGNATURE,
        hash = &last_hash[..last_hash.len() - SIGNATURE.len()]
    );
    assert_eq!(expected, stdout);
});

// Does not allow non-hex characters as signature.
gashtest!(does_not_allow_non_hex_chars, |mut tcmd: TestCommand| {
    let bad_signature = "food";
    let stderr = tcmd.args(&[bad_signature]).stderr();
    let expected = format!(
        "Error: Signature may only contain [a-z0-9] characters! Got: {}\n",
        bad_signature
    );
    assert_eq!(expected, stderr);
});
gashtest!(
    does_not_allow_non_hex_chars_odd_length,
    |mut tcmd: TestCommand| {
        let bad_signature = "hello";
        let stderr = tcmd.args(&[bad_signature]).stderr();
        let expected = format!(
            "Error: Signature may only contain [a-z0-9] characters! Got: {}\n",
            bad_signature
        );
        assert_eq!(expected, stderr);
    }
);

// Does not allow signatures longer than a sha1 hash
gashtest!(does_not_allow_long_signatures, |mut tcmd: TestCommand| {
    let bad_signature = "babebabebabebabebabebabebabebabebabebabedead";
    let stderr = tcmd.args(&[bad_signature]).stderr();
    let expected = format!("Error: Signature cannot exceed 40 characters in length!\n");
    assert_eq!(expected, stderr);
});

// Does not patch the commit with --dry-run.
gashtest!(dry_run_long_signature, |mut tcmd: TestCommand| {
    let hash_before = git_last_hash(tcmd.dir());
    let stderr = tcmd.args(&[SIGNATURE, "--dry-run"]).stderr();
    let hash_after = git_last_hash(tcmd.dir());

    assert_eq!(
        true,
        stderr.contains(&"Not amending commit due to --dry-run")
    );
    assert_eq!(hash_before, hash_after);
});

// Should fail if not run in a git repository.
gashtest!(it_does_not_work_outside_of_git, |mut tcmd: TestCommand| {
    fs::remove_dir_all(&tcmd.dir().join(".git")).unwrap();

    let expected = "\
Error: the command: 'git rev-parse HEAD' failed with:

fatal: not a git repository";
    let result = tcmd.args(&[SIGNATURE]).stderr();
    assert!(result.contains(expected));
});
