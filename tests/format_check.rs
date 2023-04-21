use pretty_assertions::assert_eq;
use std::{
    env::var,
    io::Write,
    ops::Not,
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
pub fn run_test(bin: &str, input: &str, output: &str) {
    let path = build(false, bin);
    let expected_output: String = output
        .lines()
        .filter_map(|x| {
            x.trim()
                .is_empty()
                .not()
                .then_some(format!("{}\n", x.trim()))
        })
        .collect();
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
    let stdout = child.wait_with_output().unwrap().stdout;
    let output = String::from_utf8_lossy(&stdout);
    assert_eq!(output, expected_output, "{input}");
}

/// test echo node input and output.
#[test]
fn test_echo() {
    let input = r#"
    { "src": "c1", "dest": "n1", "body": { "msg_id": 1, "type": "init", "node_id": "n1", "node_ids": ["n1", "n2"] } }
    { "src": "c1", "dest": "n1", "body": { "type": "echo", "msg_id": 1, "echo": "Please echo 35" } }
    "#;
    let output = r#"
    {"src":"n1","dest":"c1","body":{"msg_id":null,"in_reply_to":1,"type":"init_ok"}}
    {"src":"n1","dest":"c1","body":{"msg_id":0,"in_reply_to":1,"type":"echo_ok","echo":"Please echo 35"}}
    "#;
    run_test("echo", input, output);
}

/// test unique id node input and output.
#[test]
fn test_unique_id() {
    let input = r#"
    { "src": "c1", "dest": "n1", "body": { "msg_id": 1, "type": "init", "node_id": "n1", "node_ids": ["n1", "n2"] } }
    { "src": "c1", "dest": "n1", "body": { "type": "generate", "msg_id": 1 } }
    "#;
    let output = r#"
    {"src":"n1","dest":"c1","body":{"msg_id":null,"in_reply_to":1,"type":"init_ok"}}
    {"src":"n1","dest":"c1","body":{"msg_id":0,"in_reply_to":1,"type":"generate_ok","id":"n1/0"}}
    "#;
    run_test("unique_ids", input, output);
}

/// test broadcast node input and output.
#[test]
fn test_broadcast() {
    let input = r#"
    { "src": "c1", "dest": "n1", "body": { "msg_id": 1, "type": "init", "node_id": "n1", "node_ids": ["n1", "n2"] } }
    { "src": "c1", "dest": "n1", "body": { "type": "read", "msg_id": 1 } }
    { "src": "c1", "dest": "n1", "body": { "type": "broadcast", "message": 1000,"msg_id": 2 } }
    { "src": "c1", "dest": "n1", "body": { "type": "topology", "topology": { "n1": ["n2", "n3"] } ,"msg_id": 3 } }
    "#;
    let output = r#"
    {"src":"n1","dest":"c1","body":{"msg_id":null,"in_reply_to":1,"type":"init_ok"}}
    {"src":"n1","dest":"c1","body":{"msg_id":0,"in_reply_to":1,"type":"read_ok","messages":[]}}
    {"src":"n1","dest":"c1","body":{"msg_id":1,"in_reply_to":2,"type":"broadcast_ok"}}
    {"src":"n1","dest":"c1","body":{"msg_id":2,"in_reply_to":3,"type":"topology_ok"}}
    "#;
    run_test("broadcast", input, output);
}

/// test g-counter node input and output.
#[test]
#[ignore = "This has race condition"]
fn test_g_counter() {
    let input = r#"
    { "src": "c1", "dest": "n1", "body": { "msg_id": 1, "type": "init", "node_id": "n1", "node_ids": ["n1", "n2"] } }
    { "src": "c1", "dest": "n1", "body": { "type": "add", "delta": 10, "msg_id": 1 } }
    { "src": "c1", "dest": "n1", "body": { "type": "read", "msg_id": 2 } }
    { "src": "seq-kv", "dest": "n1", "body": { "type": "read_ok", "msg_id": 1, "value": 10 } }
    { "src": "seq-kv", "dest": "n1", "body": { "type": "cas_ok", "msg_id": 2 } }
    { "src": "c1", "dest": "n1", "body": { "type": "read", "msg_id": 3 } }
    "#;
    let output = r#"
    {"src":"n1","dest":"c1","body":{"msg_id":null,"in_reply_to":1,"type":"init_ok"}}
    {"src":"n1","dest":"c1","body":{"msg_id":0,"in_reply_to":1,"type":"add_ok"}}
    {"src":"n1","dest":"c1","body":{"msg_id":1,"in_reply_to":2,"type":"read_ok","value":10}}
    {"src":"n1","dest":"seq-kv","body":{"msg_id":2,"in_reply_to":null,"type":"read","key":"COUNTER"}}
    {"src":"n1","dest":"seq-kv","body":{"msg_id":3,"in_reply_to":1,"type":"cas","key":"COUNTER","from":10,"to":20,"create_if_not_exists":false}}
    {"src":"n1","dest":"c1","body":{"msg_id":4,"in_reply_to":3,"type":"read_ok","value":20}}
    "#;
    run_test("g_counter", input, output);
}
