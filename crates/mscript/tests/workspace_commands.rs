use mscript::{Session, Value};

#[test]
fn clear_removes_workspace_without_erasing_prior_outputs_and_session_recovers() {
    for command in ["clear", "clear()"] {
        let mut session = Session::new();
        let output = session.execute(&format!("a=3\n{command}; b=4")).unwrap();
        assert!(session.get("a").is_none());
        assert!(session.get("ans").is_none());
        assert_eq!(output.outputs.len(), 2);
        assert!(matches!(session.get("b"),Some(Value::Numeric(m)) if m.data()==[4.]));
        session.execute("b+1").unwrap();
        assert!(matches!(session.get("ans"),Some(Value::Numeric(m)) if m.data()==[5.]));
    }
}

#[test]
fn clear_respects_shadowing_and_invalid_output_calls_do_not_mutate_workspace() {
    let mut session = Session::new();
    session.execute("a=1; clear=8;").unwrap();
    session.execute("clear").unwrap();
    assert!(session.get("a").is_some());
    assert!(session.get("clear").is_some());
    assert!(matches!(session.get("ans"),Some(Value::Numeric(m)) if m.data()==[8.]));
    let mut session = Session::new();
    session.execute("a=1;").unwrap();
    assert!(session.execute("x=clear()").is_err());
    assert!(session.get("a").is_some());
    assert!(session.execute("clear('a')").is_err());
    assert!(session.get("a").is_some());
    assert!(session.execute("clear(); a=[").is_err());
    assert!(session.get("a").is_some());
}
