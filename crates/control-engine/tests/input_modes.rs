use controlab_engine::input::transfer;
use serde_json::json;

#[test]
fn all_existing_input_modes_resolve_same_model() {
    for input in [
        json!({"expr":"(s+2)/(s(s+1)(s+4))"}),
        json!({"mode":"parts","numerator":"s+2","denominator":"s(s+1)(s+4)"}),
        json!({"mode":"coeffs","num":"1,2","den":"1,5,4,0"}),
        json!({"mode":"zpk","zeros":"-2","poles":"0,-1,-4","k":1}),
    ] {
        let tf = transfer(&input).unwrap();
        assert_eq!(tf.num, vec![1., 2.]);
        assert_eq!(tf.den, vec![1., 5., 4., 0.]);
    }
}

#[test]
fn complex_zpk_and_bad_coefficients_follow_current_rules() {
    let tf = transfer(&json!({"mode":"zpk","zeros":"","poles":"-1+2j,-1-2j","k":3})).unwrap();
    assert_eq!(tf.num, vec![3.]);
    assert_eq!(tf.den, vec![1., 2., 5.]);
    assert!(transfer(&json!({"mode":"coeffs","num":"bad","den":"1,2"})).is_err());
    assert!(transfer(&json!({"mode":"coeffs","num":"1","den":"0"})).is_err());
}
