use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const TEST_DIR: &str = "gash-tests";
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

pub fn git(dir: impl AsRef<Path>, args: &[&str]) -> Result<String, String> {
    let dir = dir.as_ref();
    let output = Command::new("git")
        .current_dir(&dir)
        // Do not use system git config (/etc/gitconfig).
        .env("GIT_CONFIG_NOSYSTEM", "1")
        // Change `HOME` so ~/.gitconfig isn't used.
        .env("HOME", &format!("{}", dir.display()))
        .args(args)
        .output()
        .expect("Failed to run git");

    if !output.status.success() {
        Err(format!("Failed to run git!\n{:?}", output))
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

pub fn git_last_hash(dir: impl AsRef<Path>) -> String {
    git(&dir, &["rev-parse", "HEAD"]).unwrap()
}

#[derive(Debug)]
pub struct TestCommand {
    cmd: Command,
    dir: PathBuf,
    exe: PathBuf,
}

impl TestCommand {
    pub fn new(name: &str) -> TestCommand {
        // Find the location of the binary we're testing.
        let exe = env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join(format!("../gash{}", env::consts::EXE_SUFFIX));

        // Create a temporary directory for each test.
        let next_id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        let dir = env::temp_dir()
            .join(TEST_DIR)
            .join(name)
            .join(&format!("{}", next_id));

        if dir.exists() {
            fs::remove_dir_all(&dir).expect("Failed to remove pre-existing directory");
        }

        // Initialise a git repository with a single commit.
        TestCommand::git_init(&dir);

        // Create a command.
        let mut cmd = Command::new(&exe);
        cmd.current_dir(&dir);
        // Do not use system git config (/etc/gitconfig).
        cmd.env("GIT_CONFIG_NOSYSTEM", "1");
        // Change `HOME` so ~/.gitconfig isn't used.
        cmd.env("HOME", &format!("{}", dir.display()));

        TestCommand { cmd, dir, exe }
    }

    pub fn dir(&self) -> PathBuf {
        self.dir.clone()
    }

    fn git_init(dir: impl AsRef<Path>) {
        let dir = dir.as_ref();
        if dir.exists() {
            panic!("The path {} already exists!", dir.display());
        }

        // Run from parent directory since this creates the target directory.
        let parent_dir = dir.parent().unwrap();
        fs::create_dir_all(&parent_dir).unwrap();

        // Initialise the repository.
        git(&parent_dir, &["init", &format!("{}", dir.display())]).unwrap();

        // Set "user.name" and "user.email" in the new repository.
        git(&dir, &["config", "user.name", "elliot alderson"]).unwrap();
        git(
            &dir,
            &["config", "user.email", "elliotalderson@protonmail.ch"],
        )
        .unwrap();

        // Add an initial commit.
        fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(dir.join("foo"))
            .unwrap()
            .write_all(b"hello")
            .unwrap();
        git(&dir, &["add", "foo"]).unwrap();
        git(&dir, &["commit", "-m", "initial commit"]).unwrap();
    }

    pub fn args(&mut self, args: &[&str]) -> &mut Self {
        self.cmd.args(args);
        self
    }

    pub fn git(&self, args: &[&str]) -> String {
        git(&self.dir, args).unwrap()
    }

    /// Get combined output from stdout and stderr. This consumes the TestCommand
    /// since it makes one-way changes to `self.cmd`.
    #[allow(dead_code)]
    pub fn all_output(mut self) -> String {
        let (mut reader, writer) = os_pipe::pipe().unwrap();
        self.cmd.stdout(writer.try_clone().unwrap());
        self.cmd.stderr(writer);

        let mut handle = self.cmd.spawn().unwrap();

        // Need to drop the Command object since it is now owning the references to
        // the writers. When it's dropped, they're dropped and this makes the reader
        // report EOF.
        // See: https://docs.rs/os_pipe/0.9.1/os_pipe/
        drop(self.cmd);

        let mut output = String::new();
        reader.read_to_string(&mut output).unwrap();
        handle.wait().unwrap();

        output
    }

    pub fn stdout(&mut self) -> String {
        let output = self.cmd.output().unwrap();
        self.expect_success(&output);
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    pub fn stderr(&mut self) -> String {
        let output = self.cmd.output().unwrap();
        String::from_utf8_lossy(&output.stderr).to_string()
    }

    fn expect_success(&self, output: &Output) {
        if !output.status.success() {
            panic!(
                "\n\n==========\n\
                command failed but expected success!\
                \
                \n\ncommand: {:?}\
                \n\ncwd: {}\
                \n\nstatus: {}\
                \n\nstdout: {}\
                \n\nstderr: {}\
                \n\n==========\n",
                self.cmd,
                self.dir.display(),
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

pub fn setup(test_name: &str) -> TestCommand {
    TestCommand::new(test_name)
}
