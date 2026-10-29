use mscript::{Session, Value};
fn data<'a>(session: &'a Session, name: &str) -> &'a [f64] {
    let Some(Value::Numeric(m)) = session.get(name) else {
        panic!("missing numeric {name}");
    };
    m.data()
}

#[test]
fn step_outputs_share_exact_samples_and_never_emit_a_figure() {
    let mut session = Session::new();
    let execution = session
        .execute("G=tf(2,[1 2]); requested=0:0.1:2; [y,t]=step(G,requested);")
        .unwrap();
    assert!(execution.plots.is_empty());
    assert!(execution.outputs.is_empty());
    assert_eq!(data(&session, "t"), data(&session, "requested"));
    session.execute("single=step(G,requested);").unwrap();
    assert_eq!(data(&session, "y"), data(&session, "single"));
    let execution = session
        .execute("[automatic_y, automatic_t]=step(G);")
        .unwrap();
    assert!(execution.plots.is_empty());
    session.execute("same=step(G,automatic_t);").unwrap();
    assert_eq!(data(&session, "automatic_y"), data(&session, "same"));
    assert!(matches!(session.get("t"),Some(Value::Numeric(m)) if m.cols()==1));
}

#[test]
fn size_outputs_support_spaces_ignored_slots_and_preserve_matrix_literals() {
    let mut session = Session::new();
    let execution = session
        .execute("A=[1 2 3;4 5 6]; [r c extra]=size(A)\n[~,keep]=size(A); [copy]=A;")
        .unwrap();
    assert_eq!(data(&session, "r"), [2.]);
    assert_eq!(data(&session, "c"), [3.]);
    assert_eq!(data(&session, "extra"), [1.]);
    assert_eq!(data(&session, "keep"), [3.]);
    assert_eq!(data(&session, "copy"), data(&session, "A"));
    assert_eq!(execution.outputs.len(), 3);
    assert!(session.get("ans").is_none());
    session.execute("[r,c]=size(zeros(0,3));").unwrap();
    assert_eq!(data(&session, "r"), [0.]);
    assert_eq!(data(&session, "c"), [3.]);
}

#[test]
fn discarded_outputs_still_request_correct_arity_and_fail_atomically() {
    let mut session = Session::new();
    session.execute("old=7; y=8; t=9;").unwrap();
    assert!(session.execute("[y,t,x]=step(tf(1,[1 1]));").is_err());
    assert_eq!(data(&session, "y"), [8.]);
    assert_eq!(data(&session, "t"), [9.]);
    assert!(session.execute("[~,~]=sin(1)").is_err());
    assert!(session.execute("[y,t]=size([1 2],1)").is_err());
    assert!(session.execute("step=[1 2]; [y,t]=step(1)").is_err());
    assert!(session.execute("old=0; [a;b]=size([1 2])").is_err());
    assert_eq!(data(&session, "old"), [7.]);
    session.execute("[a,b]=size([1 2]);").unwrap();
    assert_eq!(data(&session, "a"), [1.]);
}

#[test]
fn output_list_syntax_matches_reference_rows_keywords_and_trailing_comma() {
    let mut session = Session::new();
    session.execute("[\nr,c,]=size([1 2]);").unwrap();
    assert_eq!(data(&session, "r"), [1.]);
    assert_eq!(data(&session, "c"), [2.]);
    session.execute("[r, ...\nc]=size([1;2]);").unwrap();
    assert_eq!(data(&session, "r"), [2.]);
    for invalid in [
        "[r\nc]=size([1 2]);",
        "[r,\nc]=size([1 2]);",
        "[r,c,\n]=size([1 2]);",
        "[r,,c]=size([1 2]);",
        "[catch,c]=size([1 2]);",
        "catch=3;",
    ] {
        assert!(session.execute(invalid).is_err(), "{invalid}");
    }
    session.execute("saved=4;").unwrap();
    assert!(session.execute("saved=0; x=catch;").is_err());
    assert_eq!(data(&session, "saved"), [4.]);
}
