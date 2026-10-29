//! Exact continuous LTI simulation via an augmented matrix exponential.
use crate::{Result, transfer::TransferFunction};
use nalgebra::{DMatrix,DVector};

#[derive(Clone,Copy,Debug)]pub enum Input{Step,Impulse,Ramp,Parabolic}

/// Controllable realization used by SciPy's tf2ss (descending coefficients).
pub fn realization(model:&TransferFunction)->Result<(DMatrix<f64>,DVector<f64>,DVector<f64>,f64)>{
    if !model.is_proper(){return Err("A função de transferência deve ser própria (grau do numerador ≤ grau do denominador) para análise temporal.".into());}
    let model=model.normalized();let n=model.den.len()-1;
    let mut a=DMatrix::zeros(n,n);let mut b=DVector::zeros(n);let mut c=DVector::zeros(n);
    let mut num=vec![0.;n+1];num[n+1-model.num.len()..].copy_from_slice(&model.num);let d=num[0];
    if n>0{b[0]=1.;}
    for j in 0..n{a[(0,j)]=-model.den[j+1];c[j]=num[j+1]-d*model.den[j+1];if j>0{a[(j,j-1)]=1.;}}
    Ok((a,b,c,d))
}

pub fn simulate(model:&TransferFunction,times:&[f64],input:Input)->Result<Vec<f64>>{
    if times.len()<2||times[0]!=0.||times.iter().any(|x|!x.is_finite())||times.windows(2).any(|w|w[1]<=w[0]){return Err("O vetor temporal deve ser crescente, finito e começar em zero.".into());}
    let(a,b,c,d)=realization(model)?;let n=a.nrows();
    // Include u and its first two derivatives. Polynomial inputs are propagated
    // exactly; impulse is represented as x(0+)=B, without a Dirac sample at t=0.
    let dim=n+3;let mut generator=DMatrix::zeros(dim,dim);
    generator.view_mut((0,0),(n,n)).copy_from(&a);
    for i in 0..n{generator[(i,n)]=b[i];}
    generator[(n,n+1)]=1.;generator[(n+1,n+2)]=1.;
    let mut x=DVector::zeros(dim);
    match input{Input::Step=>x[n]=1.,Input::Impulse=>for i in 0..n{x[i]=b[i]},Input::Ramp=>x[n+1]=1.,Input::Parabolic=>x[n+2]=1.}
    let mut out=Vec::with_capacity(times.len());let mut last_dt=f64::NAN;let mut transition=DMatrix::zeros(dim,dim);
    for (i,&time) in times.iter().enumerate(){
        if i>0{let dt=time-times[i-1];if !last_dt.is_finite()||(dt-last_dt).abs()>1e-12*dt{transition=(&generator*dt).exp();last_dt=dt;}x=&transition*x;}
        let value=(0..n).map(|j|c[j]*x[j]).sum::<f64>()+d*x[n];
        if !value.is_finite(){return Err("Resposta temporal não finita.".into());}out.push(value);
    }
    Ok(out)
}

pub fn time_vector(model:&TransferFunction,final_time:Option<f64>,points:usize)->Result<Vec<f64>>{
    let points=if points==0{900}else{points}.clamp(200,4000);
    let end=match final_time{Some(v) if (0.05..=10000.).contains(&v)=>v,Some(_)=>return Err("O tempo final deve estar entre 0,05 s e 10000 s.".into()),None=>{
        let rates:Vec<f64>=model.poles()?.iter().filter(|p|p.re.abs()>1e-8).map(|p|p.re.abs()).collect();
        if rates.is_empty(){10.}else{(8./rates.iter().copied().fold(f64::INFINITY,f64::min)).clamp(1.,120.)}
    }};
    Ok((0..points).map(|i|end*i as f64/(points-1) as f64).collect())
}
