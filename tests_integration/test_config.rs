use pretty_assertions::assert_eq;

use crate::util::TestCommand;

// Keep it short so tests are fast (we're not testing prefixes here).
const PREFIX: &str = "42";

// Parallel

gashtest!(parallel_defaults_to_false, |mut tcmd: TestCommand| {
    let stderr = tcmd.args(&[PREFIX, "-vv"]).stderr();
    assert_eq!(true, stderr.contains("parallel false"));
});

gashtest!(parallel_toggle_via_cli, |mut tcmd: TestCommand| {
    let stderr = tcmd.args(&[PREFIX, "-vv", "--parallel"]).stderr();
    assert_eq!(true, stderr.contains("parallel true"));
});

gashtest!(parallel_toggle_via_config, |mut tcmd: TestCommand| {
    tcmd.git(&["config", "gash.parallel", "true"]);
    let stderr = tcmd.args(&[PREFIX, "-vv"]).stderr();
    assert_eq!(true, stderr.contains("parallel true"));
});

// Max variance

gashtest!(max_variance_defaults_to_3600, |mut tcmd: TestCommand| {
    let stderr = tcmd.args(&[PREFIX, "-vv"]).stderr();
    assert_eq!(true, stderr.contains("max_variance 3600"));
});

gashtest!(max_variance_toggle_via_cli, |mut tcmd: TestCommand| {
    let stderr = tcmd
        .args(&[PREFIX, "-vv", "--max-variance", "1200"])
        .stderr();
    assert_eq!(true, stderr.contains("max_variance 1200"));
});

gashtest!(max_variance_toggle_via_config, |mut tcmd: TestCommand| {
    tcmd.git(&["config", "gash.max-variance", "1200"]);
    let stderr = tcmd.args(&[PREFIX, "-vv"]).stderr();
    assert_eq!(true, stderr.contains("max_variance 1200"));
});

gashtest!(max_variance_cli_beats_config, |mut tcmd: TestCommand| {
    tcmd.git(&["config", "gash.max-variance", "2400"]);
    let stderr = tcmd
        .args(&[PREFIX, "-vv", "--max-variance", "4800"])
        .stderr();
    assert_eq!(true, stderr.contains("max_variance 4800"));
});

// Progress

gashtest!(progress_defaults_to_false, |mut tcmd: TestCommand| {
    let stderr = tcmd.args(&[PREFIX]).stderr();
    assert_eq!(false, stderr.contains("hashes "));
});

gashtest!(progress_toggle_via_cli, |mut tcmd: TestCommand| {
    let stderr = tcmd.args(&[PREFIX, "--progress"]).stderr();
    assert_eq!(true, stderr.contains("hashes "));
});

gashtest!(progress_toggle_via_config, |mut tcmd: TestCommand| {
    tcmd.git(&["config", "gash.progress", "true"]);
    let stderr = tcmd.args(&[PREFIX]).stderr();
    assert_eq!(true, stderr.contains("hashes "));
});
