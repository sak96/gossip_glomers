use std::{
    env::var,
    io::Write,
    process::{Command, Stdio},
};

/// Builds the binary using cargo for testing.
fn build(release: bool, bin_name: &str) -> String {
    let mut args = vec!["build", "--bin", bin_name];
    let profile = if release {
        args.push("--release");
        "release"
    } else {
        "debug"
    };
    let status = Command::new("cargo")
        .args(&args)
        .status()
        .expect("failed to build!");
    assert!(status.success());
    format!(
        "{}/{}/{}",
        var("CARGO_TARGET_DIR").unwrap_or("target".to_string()),
        profile,
        bin_name
    )
}

/// Build and run binary with input and assert output.
pub fn run_test(input: &str, expected_output: &str) {
    let path = build(false, "echo");
    let mut child = Command::new(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let output = String::from_utf8(child.wait_with_output().unwrap().stdout).unwrap();
    assert_eq!(output, expected_output, "{input}");
}
