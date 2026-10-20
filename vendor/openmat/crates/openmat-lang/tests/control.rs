use openmat_lang::{Session, Value};

#[test]
fn motor_pid_script_persists_models_and_numeric_samples() {
    let source = include_str!("../../../examples/control_motor.m");
    let mut session = Session::new();
    session.execute(source).unwrap();
    let Some(Value::TransferFunction(g)) = session.get("G") else {
        panic!("planta ausente")
    };
    assert_eq!(g.denominator(), &[1., 3., 10.]);
    let Some(Value::Numeric(y)) = session.get("y") else {
        panic!("resposta ausente")
    };
    assert_eq!((y.rows(), y.cols()), (401, 1));
    assert_eq!(y.data()[0], 0.);
    assert!(y.data().iter().all(|x| x.is_finite()));
    session.execute("dcgain(T)").unwrap();
    assert_eq!(
        session.get("ans"),
        Some(&Value::Numeric(openmat_lang::Matrix::scalar(1.)))
    );
}

#[test]
fn step_returns_a_column_and_accepts_column_time() {
    let mut session = Session::new();
    session
        .execute("G=tf([2],[1 2]); t=linspace(0,2,21); y=step(G,t'); gain=dcgain(G);")
        .unwrap();
    let Some(Value::Numeric(y)) = session.get("y") else {
        panic!()
    };
    assert_eq!((y.rows(), y.cols()), (21, 1));
    assert!((y.data()[20] - (1. - (-4f64).exp())).abs() < 1e-12);
    session.execute("R=feedback(G,1,1); dcgain(R)").unwrap();
    assert_eq!(
        session.get("ans"),
        Some(&Value::Numeric(openmat_lang::Matrix::scalar(f64::INFINITY)))
    );
}

#[test]
fn invalid_control_calls_report_locations_and_session_recovers() {
    let mut session = Session::new();
    for source in [
        "tf('z')",
        "tf([],1)",
        "tf([1 2;3 4],1)",
        "tf(1,0)",
        "step(tf('s'),[0 1])",
        "step(tf(1,[1 1]),[0])",
        "step(tf(1,[1 1]),[0 1 3])",
        "pid(1,NaN)",
        "feedback(tf(1,1),1,0)",
        "tf('s').*2",
        "tf('s')+[1 2]",
        "2^tf('s')",
        "tf(1,1)(1)",
    ] {
        let error = session.execute(source).unwrap_err();
        assert!(
            error.span.line > 0 && error.span.column > 0,
            "{source}: {error}"
        );
    }
    let error = session
        .execute("G=tf(1,[1 1]);\ny=step(G,[0 0]);")
        .unwrap_err();
    assert_eq!(error.span.line, 2);
    assert!(matches!(session.get("G"), Some(Value::TransferFunction(_))));
    session.execute("y=step(G,0:0.1:1);").unwrap();
    assert!(matches!(session.get("y"), Some(Value::Numeric(_))));
}

#[test]
fn control_functions_respect_workspace_shadowing_and_clear() {
    let mut session = Session::new();
    session.execute("tf=[4 5]; tf(2)").unwrap();
    assert_eq!(
        session.get("ans"),
        Some(&Value::Numeric(openmat_lang::Matrix::scalar(5.)))
    );
    session.clear();
    session
        .execute("s=tf('s'); G=-(1/(s+1)); H=tf(G); dcgain(H)")
        .unwrap();
    assert_eq!(
        session.get("ans"),
        Some(&Value::Numeric(openmat_lang::Matrix::scalar(-1.)))
    );
    session.clear();
    assert!(session.workspace().is_empty());
}
