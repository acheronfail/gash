use std::fs::OpenOptions;
use std::io::Write;
use std::process;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

mod cli;
mod commit;
mod git;
mod hash;
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

fn main() {
    let args = Args::parse(git::config);
    validate_prefix(&args.prefix());

    let commit_str =
        git::run(&["cat-file", "-p", "HEAD"]).expect("Failed to fetch the latest commit");
    let commit = Commit::new(&commit_str);

    let mut stdout = StandardStream::stdout(match args.color() {
        true => ColorChoice::Auto,
        false => ColorChoice::Never,
    });

    // Simple way to change terminal color.
    macro_rules! c {
        ($spec:expr) => {
            stdout.set_color(ColorSpec::new().set_fg($spec)).unwrap();
        };
        ($spec:expr, $fmt:expr) => {
            stdout.set_color(ColorSpec::new().set_fg($spec)).unwrap();
            write!(&mut stdout, $fmt).unwrap();
        };
        ($spec:expr, $fmt:expr, $( $arg:expr ),*) => {
            stdout.set_color(ColorSpec::new().set_fg($spec)).unwrap();
            write!(&mut stdout, $fmt, $($arg),*).unwrap();
        };
    }

    // Print results.
    c!(Some(Color::Yellow), "Searching for hash with prefix ");
    c!(Some(Color::Cyan), "{}\n", args.prefix());
    c!(None);

    // Print settings.
    if args.verbosity > 1 {
        c!(None, "  max_variance {}\n", args.max_variance());
        c!(None, "      parallel {}\n", args.parallel());
    }

    let result = commit.brute_force_sha1(&args).expect(
        "Failed to brute force hash! Try increasing the variance with the --max-variance flag.",
    );

    // Print more result information.
    if args.verbosity > 0 {
        c!(None, "      author ∆ {}\n", result.da);
        c!(None, "   committer ∆ {}\n", result.dc);
    }

    // Print hash.
    c!(Some(Color::Green), "Found hash ");
    c!(Some(Color::Cyan), "{}", args.prefix());
    c!(None, "{}\n", &result.sha1[args.prefix().len()..]);

    // Print out new commit.
    if args.verbosity > 2 {
        c!(Some(Color::Yellow), "Patched commit ---\n");
        c!(None, "{}\n", result.patched_commit);
        c!(Some(Color::Yellow), "------------------\n");
    }

    // Write out patched commit.
    let temp_file = "/tmp/gash";
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&temp_file)
        .expect("Failed to open temp file")
        .write_fmt(format_args!("{}", result.patched_commit))
        .expect("Failed to write to temp file");

    // Get git to hash the temp file, and double check we patched it correctly.
    let sha1_from_git = git::hash_object(&temp_file).expect("Failed to hash patched commit");
    if result.sha1 != sha1_from_git {
        c!(
            Some(Color::Red),
            "Git's hash differs from patched hash!\nOurs:  {}\nGit's: {}\n",
            result.sha1,
            sha1_from_git
        );
        process::exit(1);
    }

    // Re-write the repository so the last commit has the hash.
    if args.dry_run {
        c!(Some(Color::Red), "Not amending commit due to --dry-run\n");
    } else {
        c!(
            Some(Color::Yellow),
            "Patching last commit to include new hash... "
        );

        // Soft reset to the previous commit.
        // If there's only one commit in the repository then this will fail, but that's okay.
        let _ = git::run(&["reset", "--soft", "HEAD^"]);

        // Hash the patched commit, and write it into git's object store.
        if git::run(&["hash-object", "-t", "commit", "-w", &temp_file]).is_err() {
            c!(Some(Color::Red), "Failed to hash patched commit");
            process::exit(1);
        }

        // Reset the repository to be pointing at the patched commit.
        if git::run(&["reset", "--soft", &result.sha1]).is_err() {
            c!(Some(Color::Red), "Failed to reset repo to patched commit");
            process::exit(1);
        }

        c!(Some(Color::Green), "Success!\n");
    }

    c!(None);
}
