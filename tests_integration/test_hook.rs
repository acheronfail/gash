use std::fs;

use pretty_assertions::assert_eq;

use crate::util::TestCommand;

gashtest!(adds_git_post_commit_hook, |mut tcmd: TestCommand| {
    let stdout = tcmd.args(&["hook"]).stdout();

    let expected = "\
Creating git hook at .git/hooks/post-commit
";
    assert_eq!(expected, stdout);

    let maybe_entry = fs::read_dir(tcmd.dir().join(".git").join("hooks"))
        .unwrap()
        .find(|e| e.as_ref().unwrap().path().ends_with("post-commit"));

    assert_eq!(true, maybe_entry.is_some());

    let entry = maybe_entry.unwrap();
    let hook_path = entry.unwrap().path();

    let hook_contents = fs::read_to_string(&hook_path).unwrap();
    let expected = "\
#!/bin/bash
gash
";
    assert_eq!(expected, hook_contents);
});
