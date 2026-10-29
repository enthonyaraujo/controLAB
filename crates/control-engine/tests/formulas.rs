use controlab_engine::formulas::render;
use serde_json::json;
#[test]fn fractions_matrices_and_arrows_render_with_stable_slots(){
    let r=render(&json!({"sources":[r"\frac{1}{s+1}",r"\begin{bmatrix}1&2\\3&4\end{bmatrix}",r"x\implies y","",r"\notacommand{a}"]})).unwrap();
    for i in 0..3{assert!(r["images"][i].as_str().is_some_and(|s|s.starts_with("iVBOR")),"{i}");}
    assert!(r["images"][3].is_null());assert!(r["images"][4].is_null());assert_eq!(r["em"],48);
    assert_eq!(r["depths"].as_array().unwrap().len(),5);
}
#[test]fn rejects_non_list_and_limits_formula_count(){
    assert!(render(&json!({"sources":"invalid"})).is_err());
    assert_eq!(render(&json!({"sources":vec!["x";205]})).unwrap()["images"].as_array().unwrap().len(),200);
}
