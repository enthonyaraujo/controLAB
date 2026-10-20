use openmat_numeric::{Matrix,Complex};
use openmat_numeric::execution::{ExecutionScope,ResourceLimits,CancellationToken,ResourceFailure,checkpoint};

#[test]
fn elementwise_kernels_check_cancellation_and_restore_scope(){
    let a=Matrix::filled(1,10000,1.).unwrap();let token=CancellationToken::default();
    let guard=ExecutionScope::enter(Default::default(),token.clone());let calls=std::cell::Cell::new(0);
    let out=a.map_complex(|v|{calls.set(calls.get()+1);if calls.get()==300{token.cancel();}v});
    assert_eq!(out.unwrap_err().1,Some(ResourceFailure::Cancelled));assert!(calls.get()<=512);
    drop(guard);assert!(a.map_complex(|x|x+Complex::new(1.,0.)).is_ok());
}
#[test]
fn kernels_deny_work_and_allocations_before_result_publication(){
    let a=Matrix::filled(10,10,1.).unwrap();let limits=ResourceLimits{work_units:10,..Default::default()};
    let guard=ExecutionScope::enter(limits,Default::default());assert_eq!(a.multiply(&a).unwrap_err().1,Some(ResourceFailure::WorkLimit));drop(guard);
    let limits=ResourceLimits{allocation_bytes:10,..Default::default()};let _guard=ExecutionScope::enter(limits,Default::default());assert_eq!(a.transpose().unwrap_err().1,Some(ResourceFailure::AllocationLimit));
}
#[test]
fn scatter_budget_failure_preserves_the_original_array(){
    let mut a=Matrix::new(1,3,vec![1.,2.,3.]).unwrap();let old=a.clone();let rhs=Matrix::scalar(9.);
    let _guard=ExecutionScope::enter(ResourceLimits{allocation_bytes:1,..Default::default()},Default::default());
    assert_eq!(a.scatter(&[1],&rhs).unwrap_err().1,Some(ResourceFailure::AllocationLimit));assert_eq!(a,old);
}
