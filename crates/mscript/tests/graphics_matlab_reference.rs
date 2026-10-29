//! Optional numeric/observable reference. Never launches MATLAB in normal tests.
use mscript::Session;
use std::{fs, process::Command};

#[test]
#[ignore = "requer MATLAB local com Control System Toolbox e figuras invisíveis"]
fn compare_graphics_data_and_output_semantics_against_matlab() {
    let models = [
        "tf(1,[1 1])",
        "tf(1,[1 2 1])",
        "tf(1,[1 1 1])",
        "tf(5,[1 1])",
        "feedback(5*tf(1,[1 2 1]),1)",
        "tf(10,[1 3 10])",
    ];
    let mut expected_points = Vec::new();
    let mut script = String::from(
        "assert(license('test','Control_Toolbox'));\nset(0,'DefaultFigureVisible','off');\n",
    );
    let dir =
        std::env::temp_dir().join(format!("mscript-graphics-reference-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let output_path = dir.join("points.csv");
    let quote = |s: &str| s.replace('\'', "''");
    script.push_str(&format!(
        "fid=fopen('{}','w'); assert(fid>=0);\n",
        quote(&output_path.to_string_lossy())
    ));
    for (i, model) in models.into_iter().enumerate() {
        let mut session = Session::new();
        let command = if i == 5 {
            "step(G,0:0.025:5);"
        } else {
            "step(G);"
        };
        let execution = session.execute(&format!("G={model}; {command}")).unwrap();
        let points = execution.plots[0].figure().series()[0].points().to_vec();
        let times = points
            .iter()
            .map(|p| format!("{:.17e}", p[0]))
            .collect::<Vec<_>>()
            .join(" ");
        script.push_str(&format!("close all; G={model}; t=[{times}];\nassert(isempty(findall(0,'Type','figure')));\n[y,to]=step(G,t);\nassert(isempty(findall(0,'Type','figure')));\nv=[to,y];\nfprintf(fid,'%d,%d',size(v,1),size(v,2)); fprintf(fid,',%.17g',v(:)); fprintf(fid,'\\n');\nstep(G,t); drawnow; assert(~isempty(findall(0,'Type','figure')));\n"));
        expected_points.push(points);
    }
    let execution = Session::new()
        .execute("x=0:0.1:10; y=sin(x); plot(x,y);")
        .unwrap();
    expected_points.push(execution.plots[0].figure().series()[0].points().to_vec());
    script.push_str("close all; x=0:0.1:10; y=sin(x); h=plot(x,y); v=[get(h,'XData')',get(h,'YData')']; fprintf(fid,'%d,%d',size(v,1),size(v,2)); fprintf(fid,',%.17g',v(:)); fprintf(fid,'\\n');\n");
    script.push_str("rejected=0; try, plot([0 1],[0 1 2]); catch, rejected=rejected+1; end\ntry, step(tf(1,[1 1]),[0 1 3]); catch, rejected=rejected+1; end\ntry, step(tf('s')); catch, rejected=rejected+1; end\nassert(rejected==3); close all; fclose(fid);\n");
    let script_path = dir.join("reference.m");
    fs::write(&script_path, script).unwrap();
    let launcher =
        std::env::var("CONTROLAB_MATLAB_PATH").unwrap_or_else(|_| "/usr/local/bin/matlab".into());
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
    let text = fs::read_to_string(&output_path).unwrap();
    assert_eq!(text.lines().count(), expected_points.len());
    let mut point_count = 0;
    let mut maximum_error: f64 = 0.;
    for (line, points) in text.lines().zip(expected_points) {
        let values: Vec<f64> = line.split(',').map(|v| v.parse().unwrap()).collect();
        assert_eq!(values[0] as usize, points.len());
        assert_eq!(values[1], 2.);
        assert_eq!(values.len(), points.len() * 2 + 2);
        for (i, point) in points.iter().enumerate() {
            for (axis, &actual) in point.iter().enumerate() {
                let reference = values[2 + i + axis * points.len()];
                let error = (actual - reference).abs() / reference.abs().max(1.);
                assert!(
                    error <= 2e-9,
                    "ponto {i}, eixo {axis}: {actual} != {reference}"
                );
                maximum_error = maximum_error.max(error);
            }
        }
        point_count += points.len();
    }
    println!(
        "7 curvas, {point_count} pontos; erro escalado máximo={maximum_error:.3e}; saída numérica/figura e 3 argumentos inválidos conferidos no MATLAB"
    );
    fs::remove_dir_all(dir).unwrap();
}
