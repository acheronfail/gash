use std::fs::OpenOptions;
use std::io::Write;
use std::process;

use anyhow::{bail, Result};
use clap::crate_version;
use termcolor::{ColorChoice, StandardStream, WriteColor};

mod args;
mod commit;
mod git;
mod hash;
mod macros;
mod spiral;
mod time;

use args::Args;
use commit::Commit;
use spiral::Spiral;

fn main() -> Result<()> {
    let args = Args::parse(git::config)?;
    let signature = args.signature();

    // Handle special "hook" value
    if matches!(&signature.as_str(), &"hook") {
        git::create_post_commit_hook()?;
        process::exit(0);
    }

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

    // Check if the hash doesn't already start/end with the signature.
    let curr_hash = git::run(&["rev-parse", "HEAD"])?;
    if !args.force {
        if !args.stealth() && curr_hash.starts_with(&signature) {
            p!(stdout, Green, "Nothing to do, current hash: ");
            p!(stdout, Cyan, "{}", signature);
            p!(stdout, None, "{}\n", &curr_hash[signature.len()..]);
            return Ok(());
        } else if args.stealth() && curr_hash.ends_with(&signature) {
            p!(stdout, Green, "Nothing to do, current hash: ");
            p!(
                stdout,
                None,
                "{}",
                &curr_hash[..curr_hash.len() - signature.len()]
            );
            p!(stdout, Cyan, "{}\n", signature);
            return Ok(());
        }
    }

    let commit_str = git::run(&["cat-file", "-p", "HEAD"])?;
    let commit = Commit::new(&commit_str);

    // Print results.
    p!(
        stdout,
        Yellow,
        "Searching for hash with {} ",
        if args.stealth() { "suffix" } else { "prefix" }
    );
    p!(stdout, Cyan, "{}\n", signature);
    p!(stdout, None);

    // Print settings.
    if args.verbosity > 1 {
        p!(stderr, None, "  max_variance {}\n", args.max_variance());
        p!(stderr, None, "      parallel {}\n", args.parallel());
        p!(stderr, None, "       version {}\n", crate_version!());
    }

    let result = match commit.brute_force_sha1(&args)? {
        Some(result) => result,
        None => bail!(
            "Failed to brute force hash! Try increasing the variance with the --max-variance flag."
        ),
    };

    // Print more result information.
    if args.verbosity > 0 {
        p!(stderr, None, "      author ∆ {}\n", result.da);
        p!(stderr, None, "   committer ∆ {}\n", result.dc);
    }

    // Print hash.
    p!(stdout, Green, "Found hash ");
    if args.stealth() {
        p!(
            stdout,
            None,
            "{}",
            &result.sha1[..result.sha1.len() - signature.len()]
        );
        p!(stdout, Cyan, "{}\n", signature);
    } else {
        p!(stdout, Cyan, "{}", signature);
        p!(stdout, None, "{}\n", &result.sha1[signature.len()..]);
    }

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
