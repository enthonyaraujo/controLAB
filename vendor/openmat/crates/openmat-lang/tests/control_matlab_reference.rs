//! Explicit, optional comparison; requires Control System Toolbox locally.
use openmat_lang::{Session, Value};
use std::{fs, process::Command};

#[test]
#[ignore = "requer MATLAB com Control System Toolbox; executar explicitamente"]
fn compare_control_against_local_matlab() {
    let cases = [
        "step(tf(2,[1 2]),0:0.01:5)",
        "s=tf('s'); G=10/(s^2+3*s+10); step(G,0:0.01:5)",
        "step(feedback(tf(10,[1 3 10]),1),0:0.02:8)",
        "step(feedback(tf(1,[1 2]),1,1),0:0.02:8)",
        "step(feedback(tf(2,[1 1]),tf(3,[1 4])),0:0.01:5)",
        "step(tf(1,[1 0]),0:0.01:5)",
        "step(tf(1,[1 0 0]),0:0.01:5)",
        "step(tf(1,[1 2 1]),0:0.01:5)",
        "step(tf([2 3],[1 1]),0:0.01:5)",
        "step(tf(1000,[1 1000]),0:0.05:5)",
        "step(feedback(pid(2,1)*tf(10,[1 3 10]),1),0:0.02:8)",
        "step(feedback(pid(2,1,0.2)*tf(10,[1 3 10]),1),0:0.02:8)",
        "step(tf(1,[1 1]),linspace(0,5,101)')",
        "s=tf('s'); step(s^-1,0:0.01:5)",
        "G=tf(1,[1 1]); step(G-G,0:0.01:5)",
        "step(tf([0 4],[0 2 4]),0:0.01:5)",
        "step(tf(3,1),0:0.01:5)",
        "dcgain(tf(10,[1 3 10]))",
        "dcgain(feedback(tf(10,[1 3 10]),1))",
        "dcgain(pid(2))",
        "dcgain(pid(0,3))",
        "dcgain(tf([1 0],[1 0]))",
    ];
    let dir =
        std::env::temp_dir().join(format!("openmat-control-reference-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let output_path = dir.join("values.csv");
    let script_path = dir.join("reference.m");
    let quote = |s: &str| s.replace('\'', "''");
    let mut script = format!("assert(license('test','Control_Toolbox'),'Control System Toolbox indisponível');\nfid=fopen('{}','w');\nassert(fid>=0);\n", quote(&output_path.to_string_lossy()));
    for source in cases {
        script.push_str("clearvars -except fid;\n");
        // MATLAB step without an assigned output plots instead of setting ans.
        // Explicit assignment selects the numeric API being compared.
        let (prefix, expression) = source.rsplit_once(';').unwrap_or(("", source));
        let assigned = format!("{prefix}; v={expression};");
        script.push_str(&format!("eval('{}');\nfprintf(fid,'%d,%d',size(v,1),size(v,2));\nif ~isempty(v), fprintf(fid,',%.17g',v(:)); end\nfprintf(fid,'\\n');\n", quote(&assigned)));
    }
    script.push_str("fclose(fid);\n");
    fs::write(&script_path, script).unwrap();
    let launcher =
        std::env::var("OPENMAT_MATLAB_PATH").unwrap_or_else(|_| "/usr/local/bin/matlab".into());
    let result = Command::new("timeout")
        .arg("90")
        .arg(launcher)
        .args(["-nodisplay", "-nosplash", "-batch"])
        .arg(format!("run('{}')", quote(&script_path.to_string_lossy())))
        .env("MATLAB_PREFDIR", dir.join("prefs"))
        .current_dir(&dir)
        .output()
        .expect("MATLAB não iniciou");
    assert!(
        result.status.success(),
        "MATLAB falhou: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let reference = fs::read_to_string(output_path).unwrap();
    let lines: Vec<_> = reference.lines().collect();
    assert_eq!(lines.len(), cases.len());
    let mut samples = 0;
    let mut maximum_scaled_error: f64 = 0.;
    for (source, line) in cases.into_iter().zip(lines) {
        let expected: Vec<f64> = line.split(',').map(|x| x.parse().unwrap()).collect();
        let mut session = Session::new();
        let (prefix, expression) = source.rsplit_once(';').unwrap_or(("", source));
        session
            .execute(&format!("{prefix}; ans={expression};"))
            .unwrap_or_else(|e| panic!("{source}: {e}"));
        let Some(Value::Numeric(actual)) = session.get("ans") else {
            panic!("resultado não numérico: {source}")
        };
        assert_eq!(
            (actual.rows(), actual.cols()),
            (expected[0] as usize, expected[1] as usize),
            "shape: {source}"
        );
        assert_eq!(actual.data().len(), expected.len() - 2);
        for (&a, &b) in actual.data().iter().zip(&expected[2..]) {
            samples += 1;
            let error = (a - b).abs() / b.abs().max(1.);
            assert!(
                a == b || (a.is_nan() && b.is_nan()) || error <= 2e-9,
                "{source}: openMAT={a}, MATLAB={b}"
            );
            if error.is_finite() {
                maximum_scaled_error = maximum_scaled_error.max(error);
            }
        }
    }
    println!("{} casos de controle, {samples} valores; erro relativo escalado máximo={maximum_scaled_error:.3e}", cases.len());
    fs::remove_dir_all(dir).unwrap();
}
