use mscript::{Session, Value};

#[test]
fn assigned_step_returns_data_and_unassigned_step_emits_the_identical_samples() {
    let mut session = Session::new();
    let numeric = session
        .execute("G=tf(3,[1 2]); t=0:0.01:5; y=step(G,t);")
        .unwrap();
    assert!(numeric.plots.is_empty());
    let Some(Value::Numeric(y)) = session.get("y") else {
        panic!()
    };
    let expected = y.data().to_vec();
    let plot = session.execute("step(G,t);").unwrap();
    assert!(plot.outputs.is_empty());
    assert!(session.get("ans").is_none());
    let figure = plot.plots[0].figure();
    assert_eq!(figure.x_axis.label, "Tempo (s)");
    assert_eq!(figure.y_axis.label, "Amplitude");
    assert_eq!(figure.series()[0].name(), "G");
    for (point, expected) in figure.series()[0].points().iter().zip(expected) {
        assert_eq!(point[1], expected);
        assert!((point[1] - 1.5 * (1. - (-2. * point[0]).exp())).abs() < 1e-11);
    }
}

#[test]
fn automatic_step_plots_agree_with_analytic_responses() {
    type AnalyticCase<'a> = (&'a str, fn(f64) -> f64);
    let cases: [AnalyticCase<'_>; 5] = [
        ("G=tf(1,[1 1]); step(G);", |t| 1. - (-t).exp()),
        ("G=tf(1,[1 2 1]); step(G);", |t| 1. - (-t).exp() * (1. + t)),
        ("G=tf(1,[1 1 1]); step(G);", |t| {
            let w = 0.75f64.sqrt();
            1. - (-0.5 * t).exp() * ((w * t).cos() + 0.5 / w * (w * t).sin())
        }),
        ("G=tf(5,[1 1]); step(G);", |t| 5. * (1. - (-t).exp())),
        ("G=tf(1,[1 2 1]); T=feedback(5*G,1); step(T);", |t| {
            let w = 5f64.sqrt();
            5. / 6. * (1. - (-t).exp() * ((w * t).cos() + (w * t).sin() / w))
        }),
    ];
    for (source, expected) in cases {
        let mut session = Session::new();
        let execution = session.execute(source).unwrap();
        assert_eq!(execution.plots.len(), 1);
        for &[t, y] in execution.plots[0].figure().series()[0].points() {
            assert!((y - expected(t)).abs() < 2e-10, "{source}: t={t}, y={y}");
        }
    }
}

#[test]
fn plot_accepts_mixed_vector_orientation_and_keeps_data_order_and_workspace() {
    let mut session = Session::new();
    session.execute("x=[3 1 2]; y=[0.25;-4;2];").unwrap();
    let before = session.workspace().clone();
    let execution = session.execute("plot(x,y);").unwrap();
    assert_eq!(*session.workspace(), before);
    assert_eq!(
        execution.plots[0].figure().series()[0].points(),
        &[[3., 0.25], [1., -4.], [2., 2.]]
    );
    assert_eq!(execution.plots[0].figure().x_axis.label, "x");
    assert_eq!(execution.plots[0].figure().y_axis.label, "y");
    assert!(
        session.execute("plot([],[])").unwrap().plots[0]
            .figure()
            .series()[0]
            .points()
            .is_empty()
    );
}

#[test]
fn graphical_calls_respect_order_output_context_and_variable_shadowing() {
    let mut session = Session::new();
    let execution = session
        .execute("G=tf(1,[1 1]); step(G); plot([0 1],[3 4]); step(G,[0 1]);")
        .unwrap();
    assert_eq!(
        execution
            .plots
            .iter()
            .map(|e| e.figure().title.as_str())
            .collect::<Vec<_>>(),
        [
            "Resposta ao degrau",
            "Gráfico de linhas",
            "Resposta ao degrau"
        ]
    );
    let nested = session.execute("sum(step(G,[0 1]))").unwrap();
    assert!(nested.plots.is_empty());
    assert!(session.execute("y=step(G);").unwrap().plots.is_empty());
    session.execute("plot=[7 8]; step=[3 4];").unwrap();
    let shadowed = session.execute("plot(2); step(1)").unwrap();
    assert!(shadowed.plots.is_empty());
    assert_eq!(
        shadowed.outputs[0].value,
        Value::Numeric(mscript::Matrix::scalar(3.))
    );
}

#[test]
fn invalid_graphics_report_locations_and_a_later_execution_recovers() {
    let mut session = Session::new();
    for source in [
        "plot([0 1],[1])",
        "plot('x',[1])",
        "plot(ones(2),ones(2))",
        "plot([0 NaN],[0 1])",
        "plot(1)",
        "h=plot(1,1)",
        "step(tf('s'))",
        "step(tf(1,[1 1]),[0 1 3])",
        "[y,t,x]=step(tf(1,[1 1]))",
    ] {
        let error = session.execute(source).unwrap_err();
        assert!(error.span.line > 0 && error.span.column > 0);
    }
    assert!(session.execute("a=5; plot(0,0); missing").is_err());
    assert_eq!(
        session.get("a"),
        Some(&Value::Numeric(mscript::Matrix::scalar(5.)))
    );
    assert_eq!(
        session.execute("plot([0 1],[0 1]);").unwrap().plots.len(),
        1
    );
}

#[test]
fn graphics_event_and_point_budgets_return_diagnostics() {
    let mut session = Session::new();
    let too_many = "plot(0,0);".repeat(17);
    assert!(
        session
            .execute(&too_many)
            .unwrap_err()
            .message
            .contains("16 eventos")
    );
    assert!(
        session
            .execute("x=1:100001; plot(x,x)")
            .unwrap_err()
            .message
            .contains("100000 pontos")
    );
    let error = session
        .execute("x=1:100000; plot(x,x); plot(x,x); plot(0,0)")
        .unwrap_err();
    assert!(error.message.contains("200000 pontos"));
}
