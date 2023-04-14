use std::ops::Not;

mod common;

#[test]
fn test_echo() {
    let input = r#"
    { "src": "c1", "dest": "n1", "body": { "msg_id": 1, "type": "init", "node_id": "n1", "node_ids": ["n1", "n2"] } }
    { "src": "c1", "dest": "n1", "body": { "type": "echo", "msg_id": 1, "echo": "Please echo 35" } }
    "#;
    let output: String = r#"
    {"src":"n1","dest":"c1","body":{"msg_id":null,"in_reply_to":1,"type":"init_ok"}}
    {"src":"n1","dest":"c1","body":{"msg_id":0,"in_reply_to":1,"type":"echo_ok","echo":"Please echo 35"}}
    "#
    .lines()
    .filter_map(|x| x.trim().is_empty().not().then_some(format!("{}\n", x.trim())))
    .collect();
    common::run_test(input, &output);
}
