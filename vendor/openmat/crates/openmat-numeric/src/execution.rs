//! Cooperative execution policy shared by the interpreter and Rust kernels.
//! Limits measure charged work/payload allocations, not process RSS.
use std::{cell::RefCell, rc::Rc, sync::{Arc,atomic::{AtomicBool,Ordering}}, time::{Duration,Instant}};

#[derive(Clone,Debug,Default)]
pub struct CancellationToken(Arc<AtomicBool>);
impl CancellationToken {
    pub fn cancel(&self){self.0.store(true,Ordering::Release);}
    pub fn is_cancelled(&self)->bool{self.0.load(Ordering::Acquire)}
}
#[derive(Clone,Debug)]
pub struct ResourceLimits {
    pub work_units:u64,
    pub allocation_bytes:u64,
    pub wall_time:Option<Duration>,
}
impl Default for ResourceLimits {
    fn default()->Self{Self{work_units:250_000_000,allocation_bytes:256*1024*1024,wall_time:None}}
}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct ResourceUsage {pub work_units:u64,pub allocation_bytes:u64}
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum ResourceFailure {Cancelled,WorkLimit,AllocationLimit,Deadline}
impl std::fmt::Display for ResourceFailure {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        f.write_str(match self{Self::Cancelled=>"execução cancelada",Self::WorkLimit=>"orçamento de trabalho da execução excedido",Self::AllocationLimit=>"orçamento de alocações da execução excedido",Self::Deadline=>"prazo da execução excedido"})
    }
}
impl std::error::Error for ResourceFailure{}
struct State {limits:ResourceLimits,token:CancellationToken,start:Instant,usage:RefCell<ResourceUsage>}
thread_local!{static ACTIVE:RefCell<Option<Rc<State>>>=const{RefCell::new(None)};}
/// Scoped and thread-local; nested executions restore their parent's context.
pub struct ExecutionScope {state:Rc<State>,previous:Option<Rc<State>>}
impl ExecutionScope {
    pub fn enter(limits:ResourceLimits,token:CancellationToken)->Self {
        let state=Rc::new(State{limits,token,start:Instant::now(),usage:RefCell::new(ResourceUsage::default())});
        let previous=ACTIVE.with(|active|active.replace(Some(state.clone())));Self{state,previous}
    }
    pub fn usage(&self)->ResourceUsage{*self.state.usage.borrow()}
}
impl Drop for ExecutionScope {fn drop(&mut self){ACTIVE.with(|active|{active.replace(self.previous.take());});}}
fn charge(work:u64,bytes:u64)->Result<(),ResourceFailure>{
    ACTIVE.with(|active|{
        let Some(state)=active.borrow().clone() else{return Ok(());};
        if state.token.is_cancelled(){return Err(ResourceFailure::Cancelled);}
        if state.limits.wall_time.is_some_and(|limit|state.start.elapsed()>=limit){return Err(ResourceFailure::Deadline);}
        let mut usage=state.usage.borrow_mut();
        let work=usage.work_units.checked_add(work).filter(|&n|n<=state.limits.work_units).ok_or(ResourceFailure::WorkLimit)?;
        let bytes=usage.allocation_bytes.checked_add(bytes).filter(|&n|n<=state.limits.allocation_bytes).ok_or(ResourceFailure::AllocationLimit)?;
        *usage=ResourceUsage{work_units:work,allocation_bytes:bytes};Ok(())
    })
}
pub fn checkpoint(work:u64)->Result<(),ResourceFailure>{charge(work,0)}
pub fn allocation(bytes:u64)->Result<(),ResourceFailure>{charge(0,bytes)}
pub fn payload(elements:usize,bytes_per_element:u64)->Result<(),ResourceFailure>{
    allocation((elements as u64).checked_mul(bytes_per_element).ok_or(ResourceFailure::AllocationLimit)?)
}
pub fn collect<T>(count:usize,mut f:impl FnMut(usize)->T)->Result<Vec<T>,ResourceFailure>{
    payload(count,std::mem::size_of::<T>() as u64)?;
    let mut out=Vec::with_capacity(count);
    for i in 0..count {if i%256==0{checkpoint((count-i).min(256) as u64)?;}out.push(f(i));}
    Ok(out)
}
