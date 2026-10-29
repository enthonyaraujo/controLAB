//! Python-compatible display and strict JSON numerical conventions.
use num_complex::Complex64;
use serde_json::{Value,json};

pub fn number(v:f64)->Value{
    if v.is_nan(){Value::Null}else if v.is_infinite(){Value::String(if v>0.{"∞"}else{"-∞"}.into())}else{json!(v)}
}
pub fn general(v:f64,digits:usize)->String{
    if v.is_nan(){return "nan".into();}if v.is_infinite(){return if v>0.{"inf"}else{"-inf"}.into();}
    if v==0.{return if v.is_sign_negative(){"-0"}else{"0"}.into();}
    let s=format!("{:.*e}",digits.saturating_sub(1),v);
    let(m,e)=s.split_once('e').unwrap();let exponent=e.parse::<i32>().unwrap();
    if exponent< -4||exponent>=digits as i32{
        let m=strip_zeros(m);format!("{m}e{}{:#02}",if exponent<0{"-"}else{"+"},exponent.abs())
    }else{
        strip_zeros(&format!("{:.*}",(digits as i32-1-exponent).max(0) as usize,v))
    }
}
fn strip_zeros(s:&str)->String{if s.contains('.') {s.trim_end_matches('0').trim_end_matches('.').into()}else{s.into()}}
pub fn complex_list(values:&[Complex64])->Value{Value::Array(values.iter().map(|v|json!({"re":number(v.re),"im":number(v.im),"text":format!("{}{}{}j",general(v.re,5),if v.im.is_sign_negative(){"-"}else{"+"},general(v.im.abs(),5))})).collect())}

#[cfg(test)]mod tests{
    use super::*;
    #[test]fn python_general_format_examples(){
        for(v,s)in[(0.,"0"),(1.,"1"),(1.234567,"1.2346"),(0.00001,"1e-05"),(100000.,"1e+05"),(99999.99,"1e+05"),(-12.3,"-12.3")]{assert_eq!(general(v,5),s);}
        assert_eq!(number(f64::INFINITY),json!("∞"));assert_eq!(number(f64::NAN),Value::Null);
    }
}
