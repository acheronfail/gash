use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::{self, Command};

mod cli;
mod commit;
mod spiral;
mod time;

use cli::Args;
use commit::CommitTemplate;
use spiral::Spiral;

fn git_sha1<S: AsRef<str>>(temp_file: S) -> String {
    let output = Command::new("git")
        .args(&["hash-object", "-t", "commit", temp_file.as_ref()])
        .output()
        .expect("Failed to run git");

    if !output.status.success() {
        panic!("Failed to hash object with git: {:?}", output);
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn git<S, I>(args: I) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("git")
        .args(args)
        .output()
        .expect("Failed to run git");

    if !output.status.success() {
        Err(format!("Failed to run git!\n{:?}", output))
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

fn git_config(name: &str) -> Option<String> {
    git(&["config", name]).ok()
}

use termcolor::{Color, ColorSpec};
use termcolor::{ColorChoice, StandardStream, WriteColor};

fn main() {
    let args = Args::parse(git_config);
    let commit_template = CommitTemplate::new();

    let mut stdout = StandardStream::stdout(match args.color {
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

    let result = commit_template.brute_force_sha1(&args).expect(
        "Failed to brute force hash! Try increasing the variance with the --max-variance flag.",
    );

    // Print more result information.
    if args.verbosity > 0 {
        c!(None, "      author ∆ {}\n", result.author_delta);
        c!(None, "   committer ∆ {}\n", result.committer_delta);
    }

    // Print hash.
    c!(Some(Color::Green), "Found hash ");
    c!(Some(Color::Cyan), "{}", args.prefix());
    c!(None, "{}\n", &result.sha1[args.prefix().len()..]);

    // Write out patched commit.
    let temp_file = "/tmp/gash";
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&temp_file)
        .expect("Failed to open temp file")
        .write_fmt(format_args!("{}", result.commit_contents))
        .expect("Failed to write to temp file");

    // Get git to hash the temp file, and double check we patched it correctly.
    let sha1_from_git = git_sha1(&temp_file);
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
            "Patching last commit to include new hash...\n"
        );

        // Soft reset to the previous commit.
        // If there's only one commit in the repository then this will fail, but that's okay.
        let _ = git(&["reset", "--soft", "HEAD^"]);

        // Hash the patched commit, and write it into git's object store.
        if git(&["hash-object", "-t", "commit", "-w", &temp_file]).is_err() {
            c!(Some(Color::Red), "Failed to hash patched commit");
            process::exit(1);
        }

        // Reset the repository to be pointing at the patched commit.
        if git(&["reset", "--soft", &result.sha1]).is_err() {
            c!(Some(Color::Red), "Failed to reset repo to patched commit");
            process::exit(1);
        }

        c!(Some(Color::Green), "Success!");
    }
}
