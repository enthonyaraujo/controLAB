use mscript::numeric::{Array, Complex, DataType, MAX_ELEMENTS};

#[test]
fn canonical_shapes_empty_dimensions_and_column_major_pages() {
    let a = Array::new_nd(vec![2, 3, 4, 1], (1..=24).map(f64::from).collect()).unwrap();
    assert_eq!(a.shape(), &[2, 3, 4]);
    assert_eq!(a.dimension(8), 1);
    assert_eq!(a.element(23), Complex::new(24., 0.));
    assert_eq!(
        Array::filled_nd(vec![0, 3, 1, 2], 0.).unwrap().shape(),
        &[0, 3, 1, 2]
    );
    assert!(Array::filled_nd(vec![0, MAX_ELEMENTS + 1], 0.).is_err());
    assert!(Array::filled_nd(vec![1; 33], 0.).is_err());
    assert!(a.reshape(vec![5, 5]).is_err());
}
#[test]
fn reshape_and_clone_share_storage_until_validated_write() {
    let original = Array::new_nd(vec![2, 2], vec![1., 2., 3., 4.]).unwrap();
    let mut b = original.reshape(vec![1, 4]).unwrap();
    assert_eq!(original.data().as_ptr(), b.data().as_ptr());
    let pointer = b.data().as_ptr();
    assert!(b.scatter(&[0, 4], &Array::scalar(9.)).is_err());
    assert_eq!(b.data().as_ptr(), pointer);
    assert_eq!(b.data(), original.data());
    b.scatter(&[1, 1], &Array::new(1, 2, vec![8., 9.]).unwrap())
        .unwrap();
    assert_eq!(b.data(), &[1., 9., 3., 4.]);
    assert_eq!(original.data(), &[1., 2., 3., 4.]);
    assert_ne!(original.data().as_ptr(), b.data().as_ptr());
}
#[test]
fn broadcast_handles_singleton_zero_and_nd_axes() {
    let a = Array::new(2, 1, vec![1., 2.]).unwrap();
    let b = Array::new(1, 3, vec![10., 20., 30.]).unwrap();
    assert_eq!(
        a.zip_complex(&b, |a, b| a + b).unwrap().data(),
        &[11., 12., 21., 22., 31., 32.]
    );
    let z = Array::filled_nd(vec![0, 3, 2], 0.).unwrap();
    assert_eq!(
        z.zip_complex(&Array::filled_nd(vec![1, 3, 1], 1.).unwrap(), |a, b| a + b)
            .unwrap()
            .shape(),
        &[0, 3, 2]
    );
    assert!(
        z.zip_complex(&Array::filled_nd(vec![2, 3, 2], 0.).unwrap(), |a, b| a + b)
            .is_err()
    );
}
#[test]
fn complex_storage_projection_normalization_and_matrix_arithmetic() {
    let z = Array::from_complex(vec![1, 1], vec![Complex::new(1., 0.)], true).unwrap();
    assert!(z.is_complex());
    assert!(
        !z.zip_complex(&Array::scalar(1.), |a, b| a + b)
            .unwrap()
            .is_complex()
    );
    let a = Array::from_complex(
        vec![2, 1],
        vec![Complex::new(1., 2.), Complex::new(3., 4.)],
        true,
    )
    .unwrap();
    let inner = a
        .transpose()
        .unwrap()
        .conjugate()
        .unwrap()
        .multiply(&a)
        .unwrap();
    assert_eq!(inner.data(), &[30.]);
    assert!(!inner.is_complex());
    assert_eq!(a.imag().unwrap().data(), &[2., 4.]);
    assert!(a.logical().is_err());
    assert!(a.solve(&a).is_err());
}
#[test]
fn logical_type_and_complex_assignment_guards_preserve_old_values() {
    let mut a = Array::new(1, 3, vec![0., 1., f64::NAN])
        .unwrap()
        .logical()
        .unwrap();
    assert_eq!(a.dtype(), DataType::Logical);
    assert_eq!(a.data(), &[0., 1., 1.]);
    let old = a.clone();
    let complex = Array::from_complex(vec![1, 1], vec![Complex::new(0., 1.)], true).unwrap();
    assert!(a.scatter(&[0], &complex).is_err());
    assert_eq!(a, old);
    a.scatter(&[0], &Array::scalar(2.)).unwrap();
    assert_eq!(a.data(), &[1., 1., 1.]);
}
#[test]
fn reductions_respect_axis_and_empty_shapes() {
    let a = Array::new_nd(vec![2, 2, 2], (1..=8).map(f64::from).collect()).unwrap();
    assert_eq!(a.sum(Some(3)).unwrap().data(), &[6., 8., 10., 12.]);
    assert_eq!(a.sum(Some(1)).unwrap().shape(), &[1, 2, 2]);
    assert_eq!(
        Array::filled_nd(vec![0, 3, 2], 0.)
            .unwrap()
            .sum(None)
            .unwrap()
            .data(),
        &[0.; 6]
    );
    assert_eq!(a.sum(Some(9)).unwrap(), a);
    assert!(a.sum(Some(0)).is_err());
}
#[test]
fn complex_division_and_sqrt_avoid_finite_overflow() {
    let z = Complex::new(1e308, 1e308) / Complex::new(1., 1.);
    assert!(z.re.is_finite());
    assert_eq!(z.im, 0.);
    let z = Complex::new(1e308, 1e308).sqrt();
    assert!(z.re.is_finite() && z.im.is_finite());
    let z = Complex::new(f64::MAX, f64::MAX).sqrt();
    assert!(z.re.is_finite() && z.im.is_finite());
    let z = Complex::new(1.7e308, 0.) / Complex::new(0.9, 0.9);
    assert!(z.re.is_finite() && z.im.is_finite());
    assert_eq!(Complex::new(-4., 0.).sqrt(), Complex::new(0., 2.));
}
