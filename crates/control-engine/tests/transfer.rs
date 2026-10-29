use controlab_engine::transfer::TransferFunction;

#[test]
fn expressions_preserve_coefficients_and_normalization() {
    let tf = TransferFunction::parse("G(s) = (s + 2) / (s(s + 1)(s + 4))").unwrap();
    assert_eq!(tf.num, vec![1., 2.]);
    assert_eq!(tf.den, vec![1., 5., 4., 0.]);
    let tf = TransferFunction::parse("4/(s² + 2s + 4)").unwrap();
    assert_eq!(tf.num, vec![4.]);
    assert_eq!(tf.den, vec![1., 2., 4.]);
    assert_eq!(TransferFunction::parse("1/(2,5s+1)").unwrap().den, vec![2.5, 1.]);
}

#[test]
fn rejects_nonpolynomial_and_executable_inputs() {
    for input in ["__import__('os')", "1/(s-s)", "sin(s)", "s//2", "", "s^0.5"] {
        assert!(TransferFunction::parse(input).is_err(), "{input}");
    }
}

#[test]fn exact_input_cancellation_matches_sympy_together(){
    for(expr,num,den)in[("(s+1)/(s+1)",vec![1.],vec![1.]),("(2*s+2)/(4*s+4)",vec![1.],vec![2.]),("1/(s/2+1)",vec![2.],vec![1.,2.]),("s*(s+1)/s",vec![1.,1.],vec![1.])]{let tf=TransferFunction::parse(expr).unwrap();assert_eq!(tf.num,num,"{expr}");assert_eq!(tf.den,den,"{expr}");}
}

#[test]
fn feedback_and_roots_match_classical_case() {
    let tf = TransferFunction::parse("4/(s^2+2s+4)").unwrap();
    let poles = tf.poles().unwrap();
    assert_eq!(poles.len(), 2);
    for p in poles { assert!((p.re+1.).abs()<1e-10); assert!((p.im.abs()-3f64.sqrt()).abs()<1e-10); }
    let closed = tf.feedback(&TransferFunction::new(vec![1.],vec![1.]).unwrap(), -1.).unwrap();
    assert_eq!(closed.den, vec![1.,2.,8.]);
}

#[test]
fn companion_solver_strips_exact_zero_roots_before_iteration(){
    let mut coefficients=vec![1.,0.,-9.,0.,4.,0.,0.,0.,0.,0.,0.];
    let result=controlab_engine::polynomial::roots(&coefficients).unwrap();
    assert_eq!(result.len(),10);assert_eq!(result.iter().filter(|r|r.re==0.&&r.im==0.).count(),6);
    coefficients.fill(0.);assert!(controlab_engine::polynomial::roots(&coefficients).unwrap().is_empty());
}

#[test]
fn margin_polynomials_converge_without_sampling_approximation(){
    for p in [vec![-1.,0.,6.,0.,-20.,0.,8.],vec![4.,0.,4.,0.,-186.,0.,352.,0.,-632.,0.],vec![-1.,0.,-91.,0.,146.,0.,-63.],vec![-34.,0.,72.,0.,-1686.,0.,-1904.,0.,2592.,0.]]{
        let roots=controlab_engine::polynomial::roots(&p).unwrap();
        assert_eq!(roots.len(),p.len()-1);
        for r in roots{let residual=controlab_engine::polynomial::evaluate(&p,r).norm();let scale=p.iter().map(|c|c.abs()).sum::<f64>()*(1.+r.norm()).powi((p.len()-1) as i32);assert!(residual<1e-9*scale);}
    }
}
