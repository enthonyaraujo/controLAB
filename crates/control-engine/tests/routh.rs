use controlab_engine::routh;
use serde_json::{Value, json};

#[test]
fn reference_routh_tables_and_deductions_are_preserved() {
    let base =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/reference/python/cases");
    for name in [
        "routh_preset_00",
        "routh_preset_01",
        "routh_preset_02",
        "routh_preset_03",
        "routh_preset_04",
        "routh_preset_05",
        "routh_test_degree_5_and_high_order_1",
        "routh_test_transfer_function_rational_input_0",
    ] {
        let reference: Value = serde_json::from_str(
            &std::fs::read_to_string(base.join(format!("{name}.json"))).unwrap(),
        )
        .unwrap();
        let output = routh::analyze(&reference["input"]).unwrap();
        for field in [
            "success",
            "input_str",
            "is_fraction",
            "char_poly_latex",
            "degree",
            "coeffs_latex",
            "routh_table",
            "special_cases",
            "steps",
            "cell_details",
            "first_col_signs",
            "sign_changes",
            "rhp_poles",
            "verdict",
            "summary",
            "k_range",
        ] {
            assert_eq!(output[field], reference["output"][field], "{name}: {field}");
        }
    }
}

#[test]
fn evaluates_gain_without_symbolic_epsilon_approximation() {
    for (k, verdict, count) in [
        (3., "Estável", 0),
        (6., "Marginalmente Estável", 2),
        (10., "Instável", 0),
    ] {
        let result = routh::evaluate_k(&json!({"expr":"s^3+3*s^2+2*s+K","k_val":k})).unwrap();
        assert_eq!(result["verdict"], verdict);
        assert_eq!(result["jw_count"], count);
    }
}

#[test]
fn gain_intervals_preserve_python_partition_policy() {
    // Preserve the Python policy: symbolic leading coefficients imply positive conditions. Only positive numerator roots delimit the public intervals.
    let result = routh::analyze(&json!({"expr":"(K+2)*s^2+(K+1)*s+1"})).unwrap();
    assert!(
        result["k_range"]["stable_intervals"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s.as_str() == Some("K > 0"))
    );
    assert_eq!(result["verdict"], "Dependente de K");
}

#[test]
fn catalogue_matches_python_bridge() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/reference/python/cases/routh_catalog.json");
    let reference: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(routh::presets(), reference["output"]);
}
#[test]
fn zero_gain_preserves_zero_transfer_endpoint() {
    let r = routh::evaluate_k(&json!({"expr":"s^3+3*s^2+2*s+K","k_val":0.})).unwrap();
    assert_eq!(r["roots"], json!([]));
    assert_eq!(r["verdict"], "Estável");
}
#[test]
fn decimal_coefficients_preserve_float_latex_domain() {
    let r = routh::analyze(&json!({"expr":"0.5*s^2+2*s+1"})).unwrap();
    assert_eq!(r["char_poly_latex"], "0.5 s^{2} + 2.0 s + 1.0 = 0");
    assert_eq!(r["coeffs_latex"], json!(["0.5", "2.0", "1.0"]));
    assert_eq!(
        r["cell_details"][0]["formula"],
        "\\frac{(2.0)(1.0) - (0.5)(0)}{2.0}"
    );
    assert_eq!(r["cell_details"][0]["value_latex"], "1.0");
}

#[test]
fn irrational_gain_boundaries_and_formula_remain_analytic() {
    let r = routh::analyze(&json!({"expr":"s^4+K*s^3+3*s^2+2*s+1"})).unwrap();
    assert_eq!(r["k_range"]["critical_k"][0]["k_latex"], "\\frac{2}{3}");
    assert_eq!(r["k_range"]["critical_k"][1]["k_latex"], "3 - \\sqrt{5}");
    assert_eq!(r["k_range"]["critical_k"][2]["k_latex"], "\\sqrt{5} + 3");
    assert_eq!(
        r["routh_table"][3]["cells"][0]["latex"],
        "\\frac{- K \\left(K - 6\\right) - 4}{3 K - 2}"
    );
    assert_eq!(r["k_range"]["stable_latex"], "0.7639 < K < 5.236");
}

#[test]
fn decimal_gain_expressions_preserve_float_terms() {
    let r = routh::analyze(&json!({"expr":"0.5*s^3+1.5*s^2+s+K"})).unwrap();
    assert_eq!(
        r["routh_table"][2]["cells"][0]["latex"],
        "1.0 - 0.333333333333333 K"
    );
    assert_eq!(r["k_range"]["critical_k"][0]["k_latex"], "3.0");
}
