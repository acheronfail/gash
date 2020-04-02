use std::fs::OpenOptions;
use std::io::Write;
use std::process;

use anyhow::Result;
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
    match hex::decode(&prefix) {
        Ok(_) => {}
        Err(_) => {
            if prefix == "hook" {
                git::create_post_commit_hook().expect("Failed to create git hook");
                process::exit(0);
            }

            println!(
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

    let mut s = StandardStream::stdout(match args.color() {
        true => ColorChoice::Auto,
        false => ColorChoice::Never,
    });

    // Check if the hash doesn't already start with the prefix.
    let curr_hash = git::run(&["rev-parse", "HEAD"])?;
    if !args.force && curr_hash.starts_with(&prefix) {
        p!(s, Green, "Nothing to do, current hash: ");
        p!(s, Cyan, "{}", prefix);
        p!(s, None, "{}\n", &curr_hash[prefix.len()..]);
        return Ok(());
    }

    let commit_str = git::run(&["cat-file", "-p", "HEAD"])?;
    let commit = Commit::new(&commit_str);

    // Print results.
    p!(s, Yellow, "Searching for hash with prefix ");
    p!(s, Cyan, "{}\n", prefix);
    p!(s, None);

    // Print settings.
    if args.verbosity > 1 {
        p!(s, None, "  max_variance {}\n", args.max_variance());
        p!(s, None, "      parallel {}\n", args.parallel());
    }

    let result = commit.brute_force_sha1(&args).expect(
        "Failed to brute force hash! Try increasing the variance with the --max-variance flag.",
    );

    // Print more result information.
    if args.verbosity > 0 {
        p!(s, None, "      author ∆ {}\n", result.da);
        p!(s, None, "   committer ∆ {}\n", result.dc);
    }

    // Print hash.
    p!(s, Green, "Found hash ");
    p!(s, Cyan, "{}", prefix);
    p!(s, None, "{}\n", &result.sha1[prefix.len()..]);

    // Print out new commit.
    if args.verbosity > 2 {
        p!(s, Yellow, "Patched commit ---\n");
        p!(s, None, "{}\n", result.patched_commit);
        p!(s, Yellow, "------------------\n");
    }

    // Write out patched commit.
    let temp_file = "/tmp/gash";
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&temp_file)?
        .write_fmt(format_args!("{}", result.patched_commit))?;

    // Get git to hash the temp file, and double check we patched it correctly.
    let sha1_from_git = git::hash_object(&temp_file)?;
    if result.sha1 != sha1_from_git {
        p!(
            s,
            Red,
            "Git's hash differs from patched hash!\nOurs:  {}\nGit's: {}\n",
            result.sha1,
            sha1_from_git
        );
        process::exit(1);
    }

    // Re-write the repository so the last commit has the hash.
    if args.dry_run {
        p!(s, Red, "Not amending commit due to --dry-run\n");
    } else {
        p!(s, Yellow, "Patching last commit to include new hash... ");

        // Soft reset to the previous commit.
        // If there's only one commit in the repository then this will fail, but that's okay.
        let _ = git::run(&["reset", "--soft", "HEAD^"]);

        // Hash the patched commit, and write it into git's object store.
        git::run(&["hash-object", "-t", "commit", "-w", &temp_file])?;

        // Reset the repository to be pointing at the patched commit.
        git::run(&["reset", "--soft", &result.sha1])?;

        p!(s, Green, "Success!\n");
    }

    p!(s, None);

    Ok(())
}
