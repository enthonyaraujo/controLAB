//! Optional differential test. Normal cargo test never launches MATLAB.
use mscript::{Session, Value};
use std::fs;
use std::process::Command;

#[test]
#[ignore = "requer MATLAB local; executar explicitamente com --ignored"]
fn compare_against_local_matlab() {
    let cases = [
        "2+3*4",
        "-2^2",
        "2^3^2",
        "2^-2",
        "[1 -2 3]",
        "[1 - 2]",
        "[1:3;4:6]",
        "A=[1 2;3 4]; A*A",
        "A=[1 2;3 4]; A.*A",
        "A=[1 2;3 4]; A^2'",
        "A=[1 2;3 4]; A(:)",
        "A=[1 2;3 4]; A(:,2)",
        "A=[1 2;3 4]; A([1 3])",
        "A=[1;2;3]; A([3 1])",
        "A=[1 2 3]; A([3;1])",
        "A=[1 2]; A([])",
        "3:-1:1",
        "numel(0:0.1:0.3)",
        "size(1:0)",
        "size([])",
        "sum([])",
        "mean([])",
        "sum([1 2;3 4])",
        "sum([1 2;3 4],2)",
        "sum(zeros(0,3))",
        "mean([1 2;3 4])",
        "zeros(3)",
        "ones([2 3])",
        "eye(2,3)",
        "reshape(1:6,2,3)",
        "linspace(1,3,1)",
        "sin(linspace(0,pi,5))",
        "[0 2;1 3]\\[4;7]",
        "[4 7]/[0 1;2 3]",
        "[2 0;0 4]^-1",
        "det([1 2;3 4])",
        "det([])",
        "det(ones(2))",
    ];
    let dir = std::env::temp_dir().join(format!("mscript-reference-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let output_path = dir.join("values.csv");
    let script_path = dir.join("reference.m");
    let matlab_quote = |s: &str| s.replace('\'', "''");
    let mut script = format!(
        "fid=fopen('{}','w');\nassert(fid>=0);\n",
        matlab_quote(&output_path.to_string_lossy())
    );
    for source in cases {
        script.push_str("clearvars -except fid;\n");
        script.push_str(&format!("eval('{};');\nv=ans;\nfprintf(fid,'%d,%d',size(v,1),size(v,2));\nif ~isempty(v), fprintf(fid,',%.17g',v(:)); end\nfprintf(fid,'\\n');\n",matlab_quote(source)));
    }
    script.push_str("fclose(fid);\n");
    fs::write(&script_path, script).unwrap();
    let launcher =
        std::env::var("CONTROLAB_MATLAB_PATH").unwrap_or_else(|_| "/usr/local/bin/matlab".into());
    let result = Command::new("timeout")
        .arg("90")
        .arg(launcher)
        .args(["-nodisplay", "-nosplash", "-batch"])
        .arg(format!(
            "run('{}')",
            matlab_quote(&script_path.to_string_lossy())
        ))
        .env("MATLAB_PREFDIR", dir.join("prefs"))
        .current_dir(&dir)
        .output()
        .expect("não foi possível iniciar MATLAB");
    assert!(result.status.success(), "MATLAB falhou ({})", result.status);
    let reference = fs::read_to_string(output_path).unwrap();
    let lines: Vec<&str> = reference.lines().collect();
    assert_eq!(lines.len(), cases.len());
    for (source, line) in cases.into_iter().zip(lines) {
        let expected: Vec<f64> = line.split(',').map(|n| n.parse().unwrap()).collect();
        let mut session = Session::new();
        session
            .execute(source)
            .unwrap_or_else(|e| panic!("{source}: {e}"));
        let Some(Value::Numeric(actual)) = session.get("ans") else {
            panic!("sem ans: {source}");
        };
        assert_eq!(
            (actual.rows(), actual.cols()),
            (expected[0] as usize, expected[1] as usize),
            "shape: {source}"
        );
        assert_eq!(actual.data().len(), expected.len() - 2, "numel: {source}");
        for (&a, &b) in actual.data().iter().zip(&expected[2..]) {
            assert!(
                a == b || (a.is_nan() && b.is_nan()) || (a - b).abs() <= 1e-12 * b.abs().max(1.0),
                "{source}: ControLAB={a}, MATLAB={b}"
            );
        }
    }
    println!("{} casos concordam com MATLAB local", cases.len());
    fs::remove_dir_all(dir).unwrap();
}
