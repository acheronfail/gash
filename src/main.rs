use std::fs::OpenOptions;
use std::io::Write;
use std::process;

use anyhow::Result;
use clap::crate_version;
use termcolor::{ColorChoice, StandardStream, WriteColor};

mod cli;
mod commit;
mod git;
mod hash;
mod macros;
mod spiral;
mod time;

use cli::Args;
use commit::Commit;
use spiral::Spiral;

// Validate prefix is hex or handle special "hook" value.
fn validate_prefix(prefix: &str) {
    // In order to validate as hex the string must be of even length.
    let normalised_prefix = if prefix.len() % 2 == 0 {
        String::from(prefix)
    } else {
        format!("{}0", prefix)
    };

    match hex::decode(normalised_prefix) {
        Ok(_) => {}
        Err(_) => {
            if prefix == "hook" {
                git::create_post_commit_hook().expect("Failed to create git hook");
                process::exit(0);
            }

            eprintln!(
                "The prefix must only contain hex characters! Got: {}",
                &prefix,
            );
            process::exit(1);
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse(git::config);
    let prefix = args.prefix();
    validate_prefix(&prefix);

    // Initialse output handles.
    let (mut stdout, mut stderr) = {
        let choice = if args.color() {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        };

        (
            StandardStream::stdout(choice),
            StandardStream::stderr(choice),
        )
    };

    // Check if the hash doesn't already start with the prefix.
    let curr_hash = git::run(&["rev-parse", "HEAD"])?;
    if !args.force && curr_hash.starts_with(&prefix) {
        p!(stdout, Green, "Nothing to do, current hash: ");
        p!(stdout, Cyan, "{}", prefix);
        p!(stdout, None, "{}\n", &curr_hash[prefix.len()..]);
        return Ok(());
    }

    let commit_str = git::run(&["cat-file", "-p", "HEAD"])?;
    let commit = Commit::new(&commit_str);

    // Print results.
    p!(stdout, Yellow, "Searching for hash with prefix ");
    p!(stdout, Cyan, "{}\n", prefix);
    p!(stdout, None);

    // Print settings.
    if args.verbosity > 1 {
        p!(stderr, None, "  max_variance {}\n", args.max_variance());
        p!(stderr, None, "      parallel {}\n", args.parallel());
        p!(stderr, None, "       version {}\n", crate_version!());
    }

    let result = commit.brute_force_sha1(&args).expect(
        "Failed to brute force hash! Try increasing the variance with the --max-variance flag.",
    );

    // Print more result information.
    if args.verbosity > 0 {
        p!(stderr, None, "      author ∆ {}\n", result.da);
        p!(stderr, None, "   committer ∆ {}\n", result.dc);
    }

    // Print hash.
    p!(stdout, Green, "Found hash ");
    p!(stdout, Cyan, "{}", prefix);
    p!(stdout, None, "{}\n", &result.sha1[prefix.len()..]);

    // Print out new commit.
    if args.verbosity > 2 {
        p!(stderr, Yellow, "Patched commit ---\n");
        p!(stderr, None, "{}\n", result.patched_commit);
        p!(stderr, Yellow, "------------------\n");
    }

    // Write out patched commit.
    let temp_file = tempfile::NamedTempFile::new()?;
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&temp_file)?
        .write_fmt(format_args!("{}", result.patched_commit))?;

    // Get git to hash the temp file, and double check we patched it correctly.
    let sha1_from_git = git::hash_object(temp_file.path())?;
    if result.sha1 != sha1_from_git {
        p!(
            stderr,
            Red,
            "Git's hash differs from patched hash!\nOurs:  {}\nGit's: {}\n",
            result.sha1,
            sha1_from_git
        );
        process::exit(1);
    }

    // Re-write the repository so the last commit has the hash.
    if args.dry_run {
        p!(stderr, Red, "Not amending commit due to --dry-run\n");
    } else {
        p!(
            stdout,
            Yellow,
            "Patching last commit to include new hash... "
        );

        // Soft reset to the previous commit.
        // If there's only one commit in the repository then this will fail, but that's okay.
        let _ = git::run(&["reset", "--soft", "HEAD^"]);

        // Hash the patched commit, and write it into git's object store.
        git::run(&[
            "hash-object",
            "-t",
            "commit",
            "-w",
            &format!("{}", temp_file.path().display()),
        ])?;

        // Reset the repository to be pointing at the patched commit.
        git::run(&["reset", "--soft", &result.sha1])?;

        p!(stdout, Green, "Success!\n");
    }

    p!(stderr, None);
    p!(stdout, None);

    Ok(())
}
