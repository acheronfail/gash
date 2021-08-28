use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
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

    pub fn from_io_error(e: io::Error) -> GitError {
        GitError::new(format!("{}", e))
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

const SCRIPT_SHEBANGS: [&str; 8] = [
    "#!/bin/sh",
    "#!/bin/bash",
    "#!/bin/zsh",
    "#!/bin/ash",
    "#!/usr/bin/env sh",
    "#!/usr/bin/env bash",
    "#!/usr/bin/env zsh",
    "#!/usr/bin/env ash",
];

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

    // Create directories.
    fs::create_dir_all(&hooks_dir).map_err(GitError::from_io_error)?;

    if post_commit_hook.exists() {
        let existing_hook =
            fs::read_to_string(&post_commit_hook).map_err(GitError::from_io_error)?;
        let existing_hook_lines = existing_hook.lines().collect::<Vec<_>>();

        // Dumb check to see if hook is a shell script
        let is_shell_script = existing_hook_lines
            .first()
            .map(|s| SCRIPT_SHEBANGS.iter().any(|shebang| s.contains(shebang)))
            .unwrap_or(false);

        // Another dumb check to see if our hook already exists
        let contains_hook = existing_hook_lines.iter().any(|line| line.contains("gash"));

        if contains_hook {
            println!(
                "The git hook at {} already calls gash!",
                relative_path_display(&post_commit_hook)?
            );
            return Ok(());
        }
        if !is_shell_script {
            println!(
                "The git hook at {} is not a script, cannot add in gash hook!",
                relative_path_display(&post_commit_hook)?
            );
            return Ok(());
        }

        println!(
            "Patching existing git hook at {}",
            relative_path_display(&post_commit_hook)?
        );
        OpenOptions::new()
            .write(true)
            .append(true)
            .open(&post_commit_hook)
            .map_err(GitError::from_io_error)?
            .write_fmt(format_args!("{}", "\n\ngash\n"))
            .map_err(GitError::from_io_error)?;
    } else {
        post_commit_hook_write_file(post_commit_hook)?;
    }

    Ok(())
}

fn relative_path_display<P: AsRef<Path>>(post_commit_hook: P) -> Result<String, GitError> {
    let cwd = std::env::current_dir().map_err(GitError::from_io_error)?;
    pathdiff::diff_paths(&post_commit_hook, cwd)
        .map(|p| p.display().to_string())
        .ok_or(GitError::new(format!(
            "failed to construct a relative path from PWD to {}",
            post_commit_hook.as_ref().display()
        )))
}

fn post_commit_hook_write_file(post_commit_hook: PathBuf) -> Result<(), GitError> {
    println!(
        "Creating git hook at {}",
        relative_path_display(&post_commit_hook)?
    );

    // Create file.
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&post_commit_hook)
        .map_err(GitError::from_io_error)?;

    // Make it executable on unix.
    if cfg!(unix) {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = file
            .metadata()
            .map_err(GitError::from_io_error)?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(post_commit_hook, perms).map_err(GitError::from_io_error)?;
    }

    // Write the hook.
    file.write_fmt(format_args!("{}", "#!/bin/bash\ngash\n"))
        .map_err(GitError::from_io_error)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_path() {
        let cwd = std::env::current_dir().unwrap();
        let hook = cwd.join(".git").join("hooks").join("post-commit");

        assert_eq!(relative_path_display(&hook).unwrap(), ".git/hooks/post-commit");

        std::env::set_current_dir(cwd.join("src")).unwrap();
        assert_eq!(relative_path_display(&hook).unwrap(), "../.git/hooks/post-commit");

        std::env::set_current_dir(cwd.join(".git").join("hooks")).unwrap();
        assert_eq!(relative_path_display(&hook).unwrap(), "post-commit");
    }
}
