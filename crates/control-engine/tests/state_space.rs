use controlab_engine::{input, state_space};
use nalgebra::DMatrix;
use serde_json::{Value, json};

fn matrix(v: &Value) -> DMatrix<f64> {
    let rows = v.as_array().unwrap();
    DMatrix::from_fn(rows.len(), rows[0].as_array().unwrap().len(), |i, j| {
        rows[i][j].as_f64().unwrap()
    })
}
fn close(a: &DMatrix<f64>, b: &DMatrix<f64>) {
    assert_eq!(a.shape(), b.shape());
    assert!((a - b).norm() < 1e-7, "{a}\n!=\n{b}");
}

#[test]
fn original_siso_structure_ackermann_and_dual_observer_match_reference() {
    let output =
        state_space::analyze(&json!({"desired_poles":"-4, -5","observer_poles":"-6, -7"})).unwrap();
    let d = &output["details"];
    assert_eq!(d["controllability_rank"], 2);
    assert_eq!(d["observability_rank"], 2);
    assert_eq!(d["state_feedback_method"], "Fórmula de Ackermann");
    assert_eq!(d["observer_method"], "Ackermann dual — Luenberger");
    close(
        &matrix(&d["state_feedback_gain"]),
        &DMatrix::from_row_slice(1, 2, &[18., 6.]),
    );
    close(
        &matrix(&d["observer_gain"]),
        &DMatrix::from_row_slice(2, 1, &[10., 10.]),
    );
    assert_eq!(d["transfer_function"]["numerator"], json!([1.]));
    assert_eq!(d["transfer_function"]["denominator"], json!([1., 3., 2.]));
    assert!(
        output["image"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,")
    );
}

#[test]
fn all_canonical_forms_preserve_the_documented_similarity_relation() {
    for form in [
        "none",
        "original",
        "controllable",
        "reachable",
        "observable",
        "diagonal",
        "jordan",
    ] {
        let out = state_space::analyze(&json!({"canonical_form":form})).unwrap();
        let d = &out["details"];
        let c = &d["canonical"];
        let a = matrix(&d["A"]);
        let b = matrix(&d["B"]);
        let cc = matrix(&d["C"]);
        let t = matrix(&c["transformation"]);
        close(&(&t * &a), &(matrix(&c["A"]) * &t));
        close(&(&t * &b), &matrix(&c["B"]));
        close(&(matrix(&c["C"]) * &t), &cc);
    }
}

#[test]
fn jordan_handles_defective_and_multiple_blocks_without_diagonal_fallback() {
    for a in [
        json!([[2, 1, 0], [0, 2, 1], [0, 0, 2]]),
        json!([[2, 0, 0], [0, 2, 1], [0, 0, 2]]),
        json!([[1, 0, 0], [0, 2, 1], [0, 0, 2]]),
    ] {
        let out = state_space::analyze(
            &json!({"A":a,"B":[[0],[0],[1]],"C":[[1,0,0]],"D":0,"canonical_form":"jordan"}),
        )
        .unwrap();
        let d = &out["details"];
        let c = &d["canonical"];
        let t = matrix(&c["transformation"]);
        close(&(&t * matrix(&d["A"])), &(matrix(&c["A"]) * &t));
        assert_eq!(c["label"], "Forma de Jordan");
    }
    assert!(
        state_space::analyze(&json!({"A":[[2,1],[0,2]],"canonical_form":"diagonal"}))
            .unwrap_err()
            .contains("Jordan")
    );
}

#[test]
fn transfer_conversion_and_shorthand_matrix_parser_match_scipy_realization() {
    let out = state_space::analyze(
        &json!({"mode":"transfer","expr":"(2s+1)/(s^2+3s+2)","canonical_form":"original"}),
    )
    .unwrap();
    assert_eq!(out["details"]["A"], json!([[-3., -2.], [1., 0.]]));
    assert_eq!(out["details"]["B"], json!([[1.], [0.]]));
    assert_eq!(out["details"]["C"], json!([[2., 1.]]));
    let out =
        state_space::analyze(&json!({"A":"0, 1; -2, -3","B":"0; 1","C":"1, 0","D":"0"})).unwrap();
    assert_eq!(out["details"]["A"], json!([[0., 1.], [-2., -3.]]));
}

#[test]
fn mimo_placement_and_observer_use_multivariable_method_and_achieve_requested_poles() {
    for request in [
        json!({"A":[[0,1],[0,0]],"B":[[1,0],[0,1]],"C":[[1,0],[0,1]],"D":[[0,0],[0,0]],"canonical_form":"original","desired_poles":[-2,-3],"observer_poles":[-4,-5]}),
        json!({"A":[[0,1,0],[0,0,1],[-1,-2,-3]],"B":[[0,1],[1,0],[1,1]],"C":[[1,0,0],[0,1,0]],"D":[[0,0],[0,0]],"canonical_form":"original","desired_poles":[-2,-3,-4],"observer_poles":[-5,-6,-7]}),
        json!({"A":[[0,1,0],[0,0,1],[-1,-2,-3]],"B":[[0,1],[1,0],[1,1]],"C":[[1,0,0],[0,1,0]],"D":[[0,0],[0,0]],"canonical_form":"original","desired_poles":["-2-1j","-2+1j",-4]}),
    ] {
        let out = state_space::analyze(&request).unwrap();
        let d = &out["details"];
        assert_eq!(d["state_feedback_method"], "Alocação multivariável");
        for (field, target) in [
            ("feedback_poles", "desired_poles"),
            ("observer_error_poles", "observer_poles"),
        ] {
            if let Some(requested) = request[target].as_array() {
                let actual = d[field].as_array().unwrap();
                for p in requested {
                    let p = if let Some(v) = p.as_f64() {
                        num_complex::Complex64::new(v, 0.)
                    } else {
                        input::complex(p.as_str().unwrap()).unwrap()
                    };
                    assert!(
                        actual
                            .iter()
                            .any(|a| (a["re"].as_f64().unwrap() - p.re).abs() < 1e-6
                                && (a["im"].as_f64().unwrap() - p.im).abs() < 1e-6),
                        "missing {p}: {actual:?}"
                    );
                }
            }
        }
        assert!(!d["transfer_function"]["siso"].as_bool().unwrap());
    }
}

#[test]
fn validation_rejects_invalid_dimensions_nonfinite_and_infeasible_designs() {
    for request in [
        json!({"A":[[1,2,3]]}),
        json!({"B":[[1,0]]}),
        json!({"D":[[0,1]]}),
        json!({"A":"nan"}),
        json!({"desired_poles":"-1"}),
        json!({"canonical_form":"unknown"}),
        json!({"A":[[-1,0],[0,-2]],"B":[[1],[0]],"desired_poles":"-3, -4"}),
        json!({"mode":"unknown"}),
    ] {
        assert!(state_space::analyze(&request).is_err(), "{request}");
    }
}

#[test]
fn diagonal_reference_eigenvector_scaling_is_preserved() {
    let out = state_space::analyze(
        &json!({"mode":"transfer","expr":"1/(s^2+3s+2)","canonical_form":"diagonal"}),
    )
    .unwrap();
    close(
        &matrix(&out["details"]["canonical"]["B"]),
        &DMatrix::from_column_slice(2, 1, &[-5f64.sqrt(), -2f64.sqrt()]),
    );
}

#[test]
fn mimo_yt_gain_matches_scipy_reference_within_convergence_tolerance() {
    let out=state_space::analyze(&json!({"A":[[0,1,0],[0,0,1],[-1,-2,-3]],"B":[[0,1],[1,0],[1,1]],"C":[[1,0,0],[0,1,0]],"D":[[0,0],[0,0]],"canonical_form":"original","desired_poles":[-2,-3,-4]})).unwrap();
    let expected = DMatrix::from_row_slice(
        2,
        3,
        &[
            -0.06688816812615428,
            2.957101855117632,
            1.1277729708172632,
            2.0939269131790157,
            -0.34014318703933366,
            -0.17880173912117828,
        ],
    );
    let actual = matrix(&out["details"]["state_feedback_gain"]);
    assert!(
        (&actual - &expected).norm() < 1e-3,
        "{actual} != {expected}"
    );
}

#[test]
fn captured_python_state_space_cases_match_scientific_and_text_fields() {
    // Formula literals/operators must be identical; only numerical tokens use
    // the same tolerance as the arrays. In particular, scipy.ss2tf's last-bit
    // eigenvalue noise must not be hardcoded into the Rust implementation.
    fn formula_tokens(text: &str) -> (Vec<String>, Vec<f64>) {
        let bytes = text.as_bytes();
        let mut literal = vec![];
        let mut numbers = vec![];
        let mut start = 0;
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i].is_ascii_digit()
                || (bytes[i] == b'.' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit())
            {
                literal.push(text[start..i].to_owned());
                let number_start = i;
                while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                    i += 1;
                }
                if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
                    let exp = i;
                    i += 1;
                    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
                        i += 1;
                    }
                    let digit_start = i;
                    while i < bytes.len() && bytes[i].is_ascii_digit() {
                        i += 1;
                    }
                    if digit_start == i {
                        i = exp;
                    }
                }
                numbers.push(text[number_start..i].parse().unwrap());
                start = i;
            } else {
                i += 1;
            }
        }
        literal.push(text[start..].to_owned());
        (literal, numbers)
    }
    fn compare(actual: &Value, expected: &Value, path: &str, errors: &mut Vec<String>) {
        if (path.contains(".latex") || path.ends_with(".source_latex"))
            && let (Some(a), Some(b)) = (actual.as_str(), expected.as_str())
        {
            let (a_text, a_numbers) = formula_tokens(a);
            let (b_text, b_numbers) = formula_tokens(b);
            if a_text == b_text
                && a_numbers.len() == b_numbers.len()
                && a_numbers
                    .iter()
                    .zip(&b_numbers)
                    .all(|(a, b)| (a - b).abs() <= 1e-8 * (1. + b.abs()))
            {
                return;
            }
        }
        if let (Some(a), Some(b)) = (actual.as_f64(), expected.as_f64()) {
            if (a - b).abs() > 1e-8 * (1. + b.abs()) {
                errors.push(format!("{path}: {a} != {b}"));
            }
            return;
        }
        match (actual, expected) {
            (Value::Object(a), Value::Object(b)) => {
                for (k, v) in b {
                    if k == "image" || k == "svg" {
                        continue;
                    }
                    match a.get(k) {
                        Some(x) => compare(x, v, &format!("{path}.{k}"), errors),
                        None => errors.push(format!("{path}.{k}: missing")),
                    }
                }
            }
            (Value::Array(a), Value::Array(b)) => {
                if a.len() != b.len() {
                    errors.push(format!("{path}: array length mismatch"));
                }
                for (i, (a, b)) in a.iter().zip(b).enumerate() {
                    compare(a, b, &format!("{path}[{i}]"), errors);
                }
            }
            _ => {
                if actual != expected {
                    errors.push(format!("{path}: {actual} != {expected}"));
                }
            }
        }
    }
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/reference/python");
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    let mut errors = vec![];
    let mut cases = 0;
    for entry in manifest["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["module"] == "state_space")
    {
        let case: Value = serde_json::from_slice(
            &std::fs::read(root.join(entry["path"].as_str().unwrap())).unwrap(),
        )
        .unwrap();
        let request = &case["input"];
        let result = match request["action"].as_str().unwrap() {
            "state_space" => state_space::analyze(request),
            "state_space_presets" => Ok(json!({"success":true,"presets":state_space::presets()})),
            _ => continue,
        };
        let actual =
            result.unwrap_or_else(|e| json!({"success":false,"error":format!("ValueError: {e}")}));
        compare(
            &actual,
            &case["output"],
            entry["id"].as_str().unwrap(),
            &mut errors,
        );
        cases += 1;
    }
    assert!(cases >= 10);
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

#[test]
fn python_literal_matrix_inputs_remain_supported_without_evaluation() {
    for text in [
        "((0, 1), (-2, -3))",
        "[[0, 1,], [-2, -3,],]",
        "[['0', '1'], ['-2', '-3']]",
        "[[False, True], [-2, -3]]",
    ] {
        close(
            &state_space::parse_matrix(&json!(text), "A").unwrap(),
            &DMatrix::from_row_slice(2, 2, &[0., 1., -2., -3.]),
        );
    }
}

#[test]
fn complex_canonical_forms_preserve_the_python_real_cast_and_complex_transform() {
    for (form, b) in [("diagonal", 5f64.sqrt() / 4.), ("jordan", 0.5)] {
        let out = state_space::analyze(&json!({"A":[[0,1],[-4,0]],"canonical_form":form})).unwrap();
        let c = &out["details"]["canonical"];
        close(&matrix(&c["A"]), &DMatrix::zeros(2, 2));
        close(&matrix(&c["B"]), &DMatrix::from_element(2, 1, b));
        close(&matrix(&c["C"]), &DMatrix::zeros(1, 2));
        assert!(c["transformation"][0][0].as_str().unwrap().ends_with('j'));
    }
}

#[test]
fn pole_plot_distinguishes_original_feedback_and_observer_with_scatter_markers() {
    let out =
        state_space::analyze(&json!({"desired_poles":"-4, -5","observer_poles":"-6, -7"})).unwrap();
    let svg = out["svg"].as_str().unwrap();
    for marker in [
        "data-pole-series=\"original\"",
        "data-pole-series=\"feedback\"",
        "data-pole-series=\"observer\"",
        "data-axis=\"real-zero\"",
        "data-axis=\"imaginary-zero\"",
    ] {
        assert!(svg.contains(marker), "missing {marker}");
    }
}

#[test]
fn jordan_recovers_repeated_poles_after_a_dense_similarity_transform() {
    let p = DMatrix::from_row_slice(3, 3, &[1., 1., 0., 0., 1., 1., 1., 0., 1.]);
    let j = DMatrix::from_row_slice(3, 3, &[2., 1., 0., 0., 2., 1., 0., 0., 2.]);
    let a = &p * j * p.clone().try_inverse().unwrap();
    let rows: Vec<Vec<_>> = (0..3)
        .map(|i| (0..3).map(|j| a[(i, j)]).collect())
        .collect();
    let out = state_space::analyze(
        &json!({"A":rows,"B":[[0],[0],[1]],"C":[[1,0,0]],"D":0,"canonical_form":"jordan"}),
    )
    .unwrap();
    let c = &out["details"]["canonical"];
    let t = matrix(&c["transformation"]);
    close(&(&t * &a), &(matrix(&c["A"]) * &t));
    close(
        &matrix(&c["A"]),
        &DMatrix::from_row_slice(3, 3, &[2., 1., 0., 0., 2., 1., 0., 0., 2.]),
    );
}

#[test]
fn static_transfer_function_keeps_zero_state_representations() {
    for form in ["original", "diagonal", "jordan"] {
        let out =
            state_space::analyze(&json!({"mode":"transfer","expr":"2","canonical_form":form}))
                .unwrap();
        let d = &out["details"];
        assert_eq!(d["A"], json!([]));
        assert_eq!(d["C"], json!([[]]));
        assert_eq!(d["D"], json!([[2.]]));
        assert_eq!(d["transfer_function"]["numerator"], json!([2.]));
        assert_eq!(d["controllability_rank"], 0);
    }
}
