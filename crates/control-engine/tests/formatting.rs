use controlab_engine::transfer::format_latex;

#[test]
fn irreducible_primitive_integer_factors_keep_their_leading_coefficient() {
    let (_, factored) = format_latex(&[-7.2], &[5., 1.], false);
    assert_eq!(factored, r"G(s) = \frac{-7.2}{5 s + 1}");
}
#[test]
fn expanded_and_factored_reference_forms() {
    let (a, b) = format_latex(&[1., 2.], &[1., 5., 4., 0.], false);
    assert_eq!(a, r"G(s) = \frac{s + 2}{s^{3} + 5 s^{2} + 4 s}");
    assert_eq!(
        b,
        r"G(s) = \frac{s + 2}{s \left(s + 1\right) \left(s + 4\right)}"
    );
    let (a, b) = format_latex(&[4.], &[1., 2., 4.], true);
    assert_eq!(a, r"G(s) = \frac{4.0}{1.0 s^{2} + 2.0 s + 4.0}");
    assert_eq!(
        b,
        r"G(s) = \frac{4.0}{4.0 \left(0.25 s^{2} + 0.5 s + 1.0\right)}"
    );
}
