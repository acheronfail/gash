use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::process::Command;

pub fn run(args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Result<String, String> {
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

pub fn hash_object<S: AsRef<str>>(temp_file: S) -> Result<String, String> {
  run(&["hash-object", "-t", "commit", temp_file.as_ref()])
}

pub fn config(name: &str) -> Option<String> {
  run(&["config", name]).ok()
}

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
  file.write_fmt(format_args!("{}", "#!/bin/bash\ngash"))?;

  Ok(())
}
