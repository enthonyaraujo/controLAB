use serde_json::Value;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn invalid_requests_do_not_desynchronize_persistent_stream() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_controlab-engine"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"invalid\n[]\n{\"action\":\"preview\",\"expr\":\"1/(s+1)\"}\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let replies: Vec<Value> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    assert_eq!(replies.len(), 3);
    assert_eq!(replies[0]["success"], false);
    assert_eq!(replies[1]["success"], false);
    assert_eq!(replies[2]["den"], serde_json::json!([1., 1.]));
}
