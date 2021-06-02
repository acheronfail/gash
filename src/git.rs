use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
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

/// Get the root of the current git repository.
fn root() -> Option<PathBuf> {
    run(&["rev-parse", "--show-toplevel"])
        .ok()
        .map(|s| PathBuf::from(s))
}

/// Create a simple post-commit hook that runs `gash`.
pub fn create_post_commit_hook() -> Result<(), GitError> {
    // Find git root.
    let git_root = match root() {
        Some(root) => root,
        None => {
            return Err(GitError::new(
                "Failed to find git root! Are you in a git repository?",
            ))
        }
    };

    let hooks_dir = git_root.join(".git").join("hooks");
    let post_commit_hook = hooks_dir.join("post-commit");
    println!("Adding git hook to {}", post_commit_hook.display());

    // Create directories.
    fs::create_dir_all(&hooks_dir).map_err(|e| GitError::new(format!("{}", e)))?;

    // Create file.
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&post_commit_hook)
        .map_err(|e| GitError::new(format!("{}", e)))?;

    // Make it executable on unix.
    if cfg!(unix) {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = file
            .metadata()
            .map_err(|e| GitError::new(format!("{}", e)))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(post_commit_hook, perms)
            .map_err(|e| GitError::new(format!("{}", e)))?;
    }

    // Write the hook.
    file.write_fmt(format_args!("{}", "#!/bin/bash\ngash\n"))
        .map_err(|e| GitError::new(format!("{}", e)))?;

    Ok(())
}
