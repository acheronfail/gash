use std::fs;

use pretty_assertions::assert_eq;

use crate::util::TestCommand;

// Should allow special "hook" command.
gashtest!(should_allow_hook, |mut tcmd: TestCommand| {
    let stdout = tcmd.args(&["hook"]).stdout();
    let expected = "Creating git hook at .git/hooks/post-commit\n";
    assert_eq!(expected, stdout);
});

// Should error if special "hook" command fails.
gashtest!(should_handle_hook_failure, |mut tcmd: TestCommand| {
    fs::remove_dir_all(&tcmd.dir().join(".git")).unwrap();
    let stderr = tcmd.args(&["hook"]).stderr();
    let expected = "Error: Failed to find git root! Are you in a git repository?\n";
    assert_eq!(expected, stderr);
});

// Full test - check that the hook was created.
gashtest!(adds_git_post_commit_hook, |mut tcmd: TestCommand| {
    let stdout = tcmd.args(&["hook"]).stdout();

    let expected = "Creating git hook at .git/hooks/post-commit\n";
    assert_eq!(expected, stdout);

    let maybe_entry = fs::read_dir(tcmd.dir().join(".git").join("hooks"))
        .unwrap()
        .find(|e| e.as_ref().unwrap().path().ends_with("post-commit"));

    assert_eq!(true, maybe_entry.is_some());

    let entry = maybe_entry.unwrap();
    let hook_path = entry.unwrap().path();

    let hook_contents = fs::read_to_string(&hook_path).unwrap();
    let expected = "#!/bin/bash\ngash\n";
    assert_eq!(expected, hook_contents);
});
