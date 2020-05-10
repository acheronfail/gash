use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub struct GitError {
    stderr: String,
}

impl GitError {
    pub fn new(stderr: impl AsRef<str>) -> GitError {
        GitError {
            stderr: stderr.as_ref().to_string(),
        }
    }
}

impl Display for GitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.stderr)
    }
}

impl Error for GitError {}

/// Run git with args.
pub fn run(args: &[&str]) -> Result<String, GitError> {
    let output = Command::new("git")
        .args(args)
        .output()
        .expect("Failed to run git");

    if !output.status.success() {
        Err(GitError::new(format!(
            "the command: 'git {}' failed with:\n\n{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        )))
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

/// Ask git to hash an object.
pub fn hash_object<R: AsRef<Path>>(temp_file: R) -> Result<String, GitError> {
    run(&[
        "hash-object",
        "-t",
        "commit",
        &format!("{}", temp_file.as_ref().display()),
    ])
}

/// Get a git config value.
pub fn config(name: &str) -> Option<String> {
    run(&["config", name]).ok()
}

/// Create a simple post-commit hook that runs `gash`.
pub fn create_post_commit_hook() -> io::Result<()> {
    let post_commit_hook = ".git/hooks/post-commit";
    println!("Adding git hook to {}", post_commit_hook);

    // Create file.
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(post_commit_hook)?;

    // Make it executable on unix.
    if cfg!(unix) {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(post_commit_hook, perms)?;
    }

    // Write the hook.
    file.write_fmt(format_args!("{}", "#!/bin/bash\ngash\n"))?;

    Ok(())
}
