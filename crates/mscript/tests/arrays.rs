use mscript::{DataType, Matrix, Session, Value};
fn value(source: &str) -> Matrix {
    let mut session = Session::new();
    session.execute(&format!("{source}\n")).unwrap();
    let Some(Value::Numeric(m)) = session.get("ans") else {
        panic!("missing numeric ans: {source}");
    };
    m.clone()
}
#[test]
fn nd_construction_reshape_and_output_dimensions() {
    assert_eq!(value("zeros(2,3,4,1)").shape(), &[2, 3, 4]);
    assert_eq!(value("zeros([0 3 1 2])").shape(), &[0, 3, 1, 2]);
    assert_eq!(
        value("A=reshape(1:24,2,[],4);[r,c]=size(A);[r c]").data(),
        &[2., 12.]
    );
    assert_eq!(
        value("A=ones(2,3,4);[r,c,p,q]=size(A);[r c p q]").data(),
        &[2., 3., 4., 1.]
    );
    assert_eq!(value("size(reshape(1:24,[2 3 4]))").data(), &[2., 3., 4.]);
    assert!(Session::new().execute("reshape(1:4,4)").is_err());
    let empty = value("[false(0,3)]");
    assert_eq!(empty.shape(), &[0, 3]);
    assert_eq!(empty.dtype(), DataType::Logical);
    let empty = value("[false(0,3) false(0,2)]");
    assert_eq!(empty.shape(), &[0, 5]);
    assert_eq!(empty.dtype(), DataType::Logical);
    assert_eq!(value("[false(2,0);false(3,0)]").shape(), &[5, 0]);
    assert_eq!(value("[ones(2,2,2) zeros(2,3,2)]").shape(), &[2, 5, 2]);
}
#[test]
fn partial_nd_indexing_end_context_and_column_major_order() {
    assert_eq!(value("A=reshape(1:24,2,3,4);A(:,end)").data(), &[23., 24.]);
    assert_eq!(
        value("A=reshape(1:24,2,3,4);A(:,:,2)").data(),
        &[7., 8., 9., 10., 11., 12.]
    );
    assert_eq!(value("A=reshape(1:24,2,3,4);A(1,end,end)").data(), &[23.]);
    assert_eq!(value("A=[10 20 30];B=[1 3];A(B(end))").data(), &[30.]);
    assert!(Session::new().execute("end").is_err());
    assert!(Session::new().execute("A=ones(2,3,4);A(1,1,1,2)").is_err());
}
#[test]
fn logical_masks_preserve_orientation_empty_forms_and_check_true_positions() {
    let a = value("A=reshape(1:6,2,3);A(logical([1 0 1 0 0 0]))");
    assert_eq!(a.shape(), &[1, 2]);
    assert_eq!(a.data(), &[1., 3.]);
    assert_eq!(value("A=1:6;A(logical([1 0 1;0 0 0]))").shape(), &[1, 2]);
    assert_eq!(value("A=1:3;A(false)").shape(), &[0, 0]);
    assert_eq!(value("A=ones(2,3);A(false(3,1))").shape(), &[0, 1]);
    assert_eq!(value("A=1:3;A(logical([1 0 1 0 0]))").data(), &[1., 3.]);
    assert!(
        Session::new()
            .execute("A=1:3;A(logical([0 0 0 1]))")
            .is_err()
    );
}
#[test]
fn indexed_assignment_is_atomic_and_snapshots_are_values() {
    let mut s = Session::new();
    s.execute("A=reshape(1:6,2,3);B=A;").unwrap();
    let old = s.get("A").unwrap().clone();
    assert!(s.execute("A(:,[1 3])=[9 8 7 6]").is_err());
    assert_eq!(s.get("A"), Some(&old));
    s.execute("A(:,2)=[7 8];A(1,3)=2i;").unwrap();
    assert_eq!(s.get("B"), Some(&old));
    assert_eq!(
        value("A=1:4;A([2 2 2])=[7 8 9];A").data(),
        &[1., 9., 3., 4.]
    );
    assert_eq!(value("A=[1 2];A([1 2])=A([2 1]);A").data(), &[2., 1.]);
    let old = s.get("A").unwrap().clone();
    assert!(s.execute("A(end+1)=9").is_err());
    assert_eq!(s.get("A"), Some(&old));
    assert!(s.execute("A(1)=[]").is_err());
    assert_eq!(s.get("A"), Some(&old));
    assert!(s.execute("A=[9 9];[r,c]=A(1)=9").is_err());
    assert_eq!(s.get("A"), Some(&old));
}
#[test]
fn complex_literals_storage_transposes_and_principal_values() {
    let a = value("i=9;A=[1+2i 3-4j];A'");
    assert_eq!(a.shape(), &[2, 1]);
    assert_eq!(a.data(), &[1., 3.]);
    assert_eq!(a.imaginary(), Some(&[-2., 4.][..]));
    assert_eq!(value("A=[1+2i 3-4j];A.'").imaginary(), Some(&[2., -4.][..]));
    assert!(value("complex(1,0)").is_complex());
    assert!(!value("complex(1,0)+1").is_complex());
    assert_eq!(value("sqrt([-4 0 4])").imaginary(), Some(&[2., 0., 0.][..]));
    assert_eq!(value("abs(3+4i)").data(), &[5.]);
    assert!(Session::new().execute("logical(1i)").is_err());
    assert!(Session::new().execute("tf([1i],[1 2])").is_err());
    assert!(Session::new().execute("A=ones(2,2,2);A'").is_err());
}
#[test]
fn implicit_expansion_comparisons_and_nd_reduction() {
    assert_eq!(value("[1 2]+[1;2]").data(), &[2., 3., 3., 4.]);
    assert_eq!(value("zeros(0,3,2)+ones(1,3)").shape(), &[0, 3, 2]);
    assert_eq!(
        value("sum(reshape(1:8,2,2,2),3)").data(),
        &[6., 8., 10., 12.]
    );
    let logical = value("A=1:4;(A>2)&~(A==4)");
    assert_eq!(logical.dtype(), DataType::Logical);
    assert_eq!(logical.data(), &[0., 0., 1., 0.]);
    assert_eq!(value("true+true").dtype(), DataType::Double);
    assert_eq!(value("[2 0;0 2]\\true(2,1)").dtype(), DataType::Double);
}
#[test]
fn empty_collapsed_dimensions_cannot_overflow_or_allocate_unbounded_selectors() {
    for source in [
        "A=zeros(0,1000000,1000000,1000000,1000000);[r,c]=size(A)",
        "A=zeros(0,1000000,1000000);A(:,:)",
    ] {
        assert!(Session::new().execute(source).is_err(), "accepted {source}");
    }
    assert_eq!(
        value("A=zeros(0,1000000,1000000);[r,c]=size(A);c").data(),
        &[1e12]
    );
}
#[test]
fn real_nonfinite_elementary_results_do_not_acquire_imaginary_nan() {
    for expression in [
        "exp(Inf)",
        "exp(1000)",
        "sin(Inf)",
        "cos(Inf)",
        "log(NaN)",
        "sqrt(NaN)",
    ] {
        assert!(!value(expression).is_complex(), "{expression}");
    }
}
