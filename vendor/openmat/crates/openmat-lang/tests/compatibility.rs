use openmat_lang::{Session, Value};

fn evaluate(source: &str) -> openmat_lang::Matrix {
    let mut s = Session::new();
    s.execute(source)
        .unwrap_or_else(|e| panic!("{source}: {e}"));
    let Some(Value::Numeric(m)) = s.get("ans") else {
        panic!("expected numeric ans: {source}");
    };
    m.clone()
}
fn check(source: &str, rows: usize, cols: usize, data: &[f64]) {
    let m = evaluate(source);
    assert_eq!((m.rows(), m.cols()), (rows, cols), "{source}");
    assert_eq!(m.data(), data, "{source}");
}

#[test]
fn operator_precedence_and_associativity() {
    check("2+3*4", 1, 1, &[14.]);
    check("(2+3)*4", 1, 1, &[20.]);
    check("-2^2", 1, 1, &[-4.]);
    check("(-2)^2", 1, 1, &[4.]);
    check("2^3^2", 1, 1, &[64.]);
    check("2^-2", 1, 1, &[0.25]);
    check("10-3-2", 1, 1, &[5.]);
    check("1:2+3", 1, 5, &[1., 2., 3., 4., 5.]);
}
#[test]
fn numeric_literals_and_operators() {
    check(".5 + 1e-2 + 2E+1", 1, 1, &[20.51]);
    check("2.*3 + 4./2 + 2.\\4", 1, 1, &[10.]);
    check("[1 2;3 4].^2", 2, 2, &[1., 9., 4., 16.]);
}
#[test]
fn matrix_arithmetic_and_scalar_expansion() {
    check("A=[1 2;3 4]; A*A", 2, 2, &[7., 15., 10., 22.]);
    check("A=[1 2;3 4]; A.*A", 2, 2, &[1., 9., 4., 16.]);
    check("A=[1 2;3 4]; A+10", 2, 2, &[11., 13., 12., 14.]);
    check("10-[1 2]", 1, 2, &[9., 8.]);
    check("[1 2;3 4]/2", 2, 2, &[0.5, 1.5, 1., 2.]);
    check("2*[1 2]", 1, 2, &[2., 4.]);
    check("[1 2;3 4]'", 2, 2, &[1., 2., 3., 4.]);
    check("[1 2;3 4].'", 2, 2, &[1., 2., 3., 4.]);
}
#[test]
fn matrix_literal_whitespace_blocks_and_newlines() {
    check("[1 -2 +3]", 1, 3, &[1., -2., 3.]);
    check("[1 - 2]", 1, 1, &[-1.]);
    check("[1+2,4]", 1, 2, &[3., 4.]);
    check("[1 2\n3 4]", 2, 2, &[1., 3., 2., 4.]);
    check("[1:3;4:6]", 2, 3, &[1., 4., 2., 5., 3., 6.]);
    check("A=[1;2]; [A A]", 2, 2, &[1., 2., 1., 2.]);
    check("[[],1,2]", 1, 2, &[1., 2.]);
}
#[test]
fn ranges_and_empty_shapes() {
    check("3:-1:1", 1, 3, &[3., 2., 1.]);
    check("1:0", 1, 0, &[]);
    check("[]", 0, 0, &[]);
    check("size(zeros(0,3))", 1, 2, &[0., 3.]);
    check("linspace(1,3,0)", 1, 0, &[]);
    check("linspace(1,3,1)", 1, 1, &[3.]);
    check("linspace(1,3,3)", 1, 3, &[1., 2., 3.]);
    check("numel(0:0.1:0.3)", 1, 1, &[4.]);
}
#[test]
fn indexing_is_one_based_and_column_major() {
    check("A=[1 2;3 4]; A(2)", 1, 1, &[3.]);
    check("A=[1 2;3 4]; A(2,1)", 1, 1, &[3.]);
    check("A=[1 2;3 4]; A(:)", 4, 1, &[1., 3., 2., 4.]);
    check("A=[1 2;3 4]; A(:,2)", 2, 1, &[2., 4.]);
    check("A=[1 2;3 4]; A([2 1],:)", 2, 2, &[3., 1., 4., 2.]);
    check("A=[1 2;3 4]; A([1 3])", 1, 2, &[1., 2.]);
    check("A=[1;2;3]; A([3 1])", 2, 1, &[3., 1.]);
    check("A=[1 2 3]; A([3;1])", 1, 2, &[3., 1.]);
    check("A=[1 2]; A([])", 0, 0, &[]);
}
#[test]
fn reductions_and_dimensions() {
    check("sum([1 2;3 4])", 1, 2, &[4., 6.]);
    check("sum([1 2;3 4],2)", 2, 1, &[3., 7.]);
    check("sum([1 2 3])", 1, 1, &[6.]);
    check("sum([1;2;3])", 1, 1, &[6.]);
    check("mean([1 2;3 4])", 1, 2, &[2., 3.]);
    check("sum([])", 1, 1, &[0.]);
    assert!(evaluate("mean([])").data()[0].is_nan());
    check("sum(zeros(0,3))", 1, 3, &[0., 0., 0.]);
    check("size([1 2;3 4],3)", 1, 1, &[1.]);
    check("numel([1 2;3 4])", 1, 1, &[4.]);
    check("length(zeros(0,3))", 1, 1, &[0.]);
}
#[test]
fn constructors_and_reshape() {
    check("zeros(2)", 2, 2, &[0.; 4]);
    check("ones([2 3])", 2, 3, &[1.; 6]);
    check("eye(2,3)", 2, 3, &[1., 0., 0., 1., 0., 0.]);
    check("reshape(1:6,2,3)", 2, 3, &[1., 2., 3., 4., 5., 6.]);
}
#[test]
fn linear_systems_and_matrix_power() {
    check("[0 2;1 3]\\[4;7]", 2, 1, &[1., 2.]);
    check("[4 7]/[0 1;2 3]", 1, 2, &[1., 2.]);
    check("[1 2;3 4]^2", 2, 2, &[7., 15., 10., 22.]);
    check("[2 0;0 4]^-1", 2, 2, &[0.5, 0., 0., 0.25]);
    check("[1 2;3 4]^0", 2, 2, &[1., 0., 0., 1.]);
}
#[test]
fn elementary_functions_and_constants() {
    check("sin(0)+cos(0)+sqrt(4)+abs(-2)+exp(0)+log(1)", 1, 1, &[6.]);
    let m = evaluate("pi");
    assert!((m.data()[0] - std::f64::consts::PI).abs() < 1e-15);
}
#[test]
fn determinant_uses_the_existing_session() {
    let mut session = Session::new();
    session.execute("A=[1 2;3 4];").unwrap();
    session.execute("det(A)").unwrap();
    let Some(Value::Numeric(value)) = session.get("ans") else {
        panic!("missing ans");
    };
    assert!((value.data()[0] + 2.).abs() < 1e-12);
    assert!(session.execute("det([1 2 3])").is_err());
    check("det([])", 1, 1, &[1.]);
    check("det(ones(2))", 1, 1, &[0.]);
}
#[test]
fn output_suppression_comments_strings_and_continuation() {
    let mut s = Session::new();
    let e=s.execute("% comentário\na=1; b=2\ndisp('olá ''mundo''');\ndisp(\"ok\");\na + ... % continuação\nb").unwrap();
    assert_eq!(e.outputs.len(), 4);
    assert_eq!(e.outputs[0].name.as_deref(), Some("b"));
    assert_eq!(e.outputs[1].value, Value::Text("olá 'mundo'".into()));
    assert_eq!(e.outputs[2].name, None);
    assert_eq!(
        s.get("ans"),
        Some(&Value::Numeric(openmat_lang::Matrix::scalar(3.)))
    );
}
#[test]
fn session_persistence_shadowing_and_clear() {
    let mut s = Session::new();
    s.execute("a=2;").unwrap();
    s.execute("a+3").unwrap();
    assert_eq!(
        s.get("ans"),
        Some(&Value::Numeric(openmat_lang::Matrix::scalar(5.)))
    );
    s.execute("sin=[10 20]; sin(2)").unwrap();
    assert_eq!(
        s.get("ans"),
        Some(&Value::Numeric(openmat_lang::Matrix::scalar(20.)))
    );
    s.clear();
    assert!(s.workspace().is_empty());
}
#[test]
fn diagnostics_and_invalid_inputs() {
    for source in [
        "A=[1 2;3]",
        "A=[1 2]; A(0)",
        "A=[1 2]; A(3)",
        "A=[1 2]; A(1.5)",
        "[1 2]*[1 2]",
        "zeros(-1)",
        "zeros(Inf)",
        "reshape(1:3,2,2)",
        "1:0:3",
        "missing(1)",
        "missing",
        "for i=1:3\nend",
        "a=1e;",
        "[1",
        "'unterminated",
        "sum(1,0)",
    ] {
        let result = Session::new().execute(source);
        assert!(result.is_err(), "accepted {source}");
    }
    let e = Session::new().execute("a=1;\nmissing").unwrap_err();
    assert_eq!((e.span.line, e.span.column), (2, 1));
}
#[test]
fn parsing_has_no_workspace_effects_and_runtime_preserves_prior_assignments() {
    let mut s = Session::new();
    assert!(s.execute("a=1; b=[").is_err());
    assert!(s.get("a").is_none());
    assert!(s.execute("a=1; b=missing").is_err());
    assert!(s.get("a").is_some());
    assert!(s.get("b").is_none());
}
#[test]
fn resource_limits_return_diagnostics() {
    for source in [
        "zeros(1000001)",
        "1:1000001",
        "eye(1001)",
        "linspace(0,1,1000001)",
    ] {
        assert!(Session::new().execute(source).is_err(), "{source}");
    }
    let nested = format!("{}1{}", "(".repeat(200), ")".repeat(200));
    assert!(Session::new().execute(&nested).is_err());
    let chain = std::iter::repeat_n("1", 300).collect::<Vec<_>>().join("+");
    assert!(Session::new().execute(&chain).is_err());
}
