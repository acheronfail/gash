use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;

mod cli;
mod commit;
mod spiral;
mod time;

use cli::Args;
use commit::CommitTemplate;
use spiral::Spiral;
use time::TimeDelta;

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

fn main() {
    let args = Args::parse(git_config);
    let commit_template = CommitTemplate::new();

    // Print results.
    if args.dry_run {
        println!("dry_run:        {}", args.dry_run);
    }
    println!("max_variance:   {}", args.max_variance());
    println!("parallel:       {}", args.parallel());
    println!("prefix:         {}", args.prefix());

    let result = commit_template.brute_force_sha1(&args).expect(
        "Failed to brute force hash! Try increasing the variance with the --max-variance flag.",
    );

    println!("sha1:           {}", &result.sha1);
    println!(
        "author_diff:    {}",
        TimeDelta(result.author_timestamp_delta)
    );
    println!(
        "committer_diff: {}",
        TimeDelta(result.committer_timestamp_delta)
    );

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
        panic!(
            "Git's hash differs from patched hash!\nOurs:  {}\nGit's: {}",
            result.sha1, sha1_from_git
        );
    }

    // Re-write the repository so the last commit has the hash.
    if args.dry_run {
        eprintln!("Not amending commit due to --dry-run");
    } else {
        eprintln!("Patching last commit to include new hash...");

        // Soft reset to the previous commit.
        // If there's only one commit in the repository then this will fail, but that's okay.
        let _ = git(&["reset", "--soft", "HEAD^"]);

        // Hash the patched commit, and write it into git's object store.
        git(&["hash-object", "-t", "commit", "-w", &temp_file])
            .expect("Failed to hash patched commit");

        // Reset the repository to be pointing at the patched commit.
        git(&["reset", "--soft", &result.sha1]).expect("Failed to reset repo to patched commit");
    }
}
