use controlab_engine::formulas::render;
use serde_json::json;

#[test]
fn every_formula_supported_by_the_frozen_python_renderer_remains_supported() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("../../../tests/reference/formulas.json")).unwrap();
    let cases = data["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 583);
    for batch in cases.chunks(200) {
        let sources: Vec<_> = batch.iter().map(|c| c["source"].clone()).collect();
        let out = render(&json!({"sources":sources})).unwrap();
        assert_eq!(out["images"].as_array().unwrap().len(), batch.len());
        for (i, case) in batch.iter().enumerate() {
            if case["python_png"] == true {
                assert!(
                    out["images"][i].is_string(),
                    "Lost Python formula: {}",
                    case["source"]
                );
            }
            let depth = out["depths"][i].as_f64().unwrap();
            assert!((0. ..=1.).contains(&depth));
        }
    }
}
#[test]
fn fractions_matrices_and_arrows_render_with_stable_slots() {
    let r=render(&json!({"sources":[r"\frac{1}{s+1}",r"\begin{bmatrix}1&2\\3&4\end{bmatrix}",r"x\implies y","",r"\notacommand{a}"]})).unwrap();
    for i in 0..3 {
        assert!(
            r["images"][i]
                .as_str()
                .is_some_and(|s| s.starts_with("iVBOR")),
            "{i}"
        );
    }
    assert!(r["images"][3].is_null());
    assert!(r["images"][4].is_null());
    assert_eq!(r["em"], 48);
    assert_eq!(r["depths"].as_array().unwrap().len(), 5);
}
#[test]
fn rejects_non_list_and_limits_formula_count() {
    assert!(render(&json!({"sources":"invalid"})).is_err());
    assert_eq!(
        render(&json!({"sources":vec!["x";205]})).unwrap()["images"]
            .as_array()
            .unwrap()
            .len(),
        200
    );
}
