//! Restricted arithmetic grammar for transfer-function inputs. No code evaluation.
use crate::{Result, polynomial as poly};
use num_complex::Complex64;

#[derive(Clone, Debug, PartialEq)]
pub struct TransferFunction { pub num:Vec<f64>, pub den:Vec<f64> }

impl TransferFunction {
    pub fn new(num:Vec<f64>,den:Vec<f64>)->Result<Self>{
        if num.is_empty()||den.is_empty()||num.iter().chain(&den).any(|v|!v.is_finite()){return Err("Coeficientes devem ser finitos e não vazios.".into());}
        let num=poly::trim(&num);let den=poly::trim(&den);
        if den==[0.] {return Err("O denominador não pode ser zero.".into());}
        Ok(Self{num,den})
    }
    pub fn scalar(x:f64)->Self{Self{num:vec![x],den:vec![1.]}}
    pub fn parse(source:&str)->Result<Self>{
        let normalized=normalize(source)?;
        let mut p=Parser{tokens:lex(&normalized)?,position:0};
        let result=p.expression()?;
        if p.position!=p.tokens.len(){return Err("Expressão inválida.".into());}
        // Restricted grammar above validates the whole input before exact
        // symbolic cancellation, preserving cancellations done by SymPy together.
        let _validated=result;
        let (num,den)=crate::exact_transfer::parse(&normalized)?;
        Self::new(num,den)
    }
    pub fn multiply(&self,b:&Self)->Result<Self>{Self::new(poly::multiply(&self.num,&b.num),poly::multiply(&self.den,&b.den))}
    pub fn divide(&self,b:&Self)->Result<Self>{Self::new(poly::multiply(&self.num,&b.den),poly::multiply(&self.den,&b.num))}
    pub fn add(&self,b:&Self,sign:f64)->Result<Self>{Self::new(poly::add(&poly::multiply(&self.num,&b.den),&poly::multiply(&b.num,&self.den),sign),poly::multiply(&self.den,&b.den))}
    /// sign=-1: negative feedback, sign=+1: positive feedback (python-control).
    pub fn feedback(&self,b:&Self,sign:f64)->Result<Self>{Self::new(poly::multiply(&self.num,&b.den),poly::add(&poly::multiply(&self.den,&b.den),&poly::multiply(&self.num,&b.num),-sign))}
    pub fn scale(&self,k:f64)->Result<Self>{Self::new(self.num.iter().map(|x|x*k).collect(),self.den.clone())}
    pub fn poles(&self)->Result<Vec<Complex64>>{poly::roots(&self.den)}
    pub fn zeros(&self)->Result<Vec<Complex64>>{poly::roots(&self.num)}
    pub fn evaluate(&self,s:Complex64)->Complex64{poly::evaluate(&self.num,s)/poly::evaluate(&self.den,s)}
    pub fn dc_gain(&self)->f64{self.evaluate(Complex64::new(0.,0.)).re}
    pub fn is_proper(&self)->bool{self.num.len()<=self.den.len()}
    pub fn normalized(&self)->Self{let a=self.den[0];Self{num:self.num.iter().map(|v|v/a).collect(),den:self.den.iter().map(|v|v/a).collect()}}
    pub fn latex(&self)->String{format!("G(s) = \\frac{{{}}}{{{}}}",polynomial_latex(&self.num),polynomial_latex(&self.den))}
    pub fn latex_formatted(&self,float_coefficients:bool)->(String,String){format_latex(&self.num,&self.den,float_coefficients)}
}

/// Expanded/factored presentation. This intentionally distinguishes parsed
/// integer expressions from the Float coefficient construction in state-space.
pub fn format_latex(num:&[f64],den:&[f64],float_coefficients:bool)->(String,String){
    (format!("G(s) = \\frac{{{}}}{{{}}}",poly_display(num,float_coefficients),poly_display(den,float_coefficients)),format!("G(s) = \\frac{{{}}}{{{}}}",factored_display(num,float_coefficients),factored_display(den,float_coefficients)))
}
fn scalar_display(v:f64,float:bool)->String{
    if float&&v.fract()==0.{format!("{v:.1}")}else if float{crate::values::general(v,15)}else{v.to_string()}
}
fn poly_display(p:&[f64],float:bool)->String{
    let mut out=String::new();for(i,&v)in p.iter().enumerate(){if v==0.{continue;}let power=p.len()-1-i;
        if out.is_empty(){if v<0.{out.push('-');}}else{out.push_str(if v<0.{" - "}else{" + "});}
        if float||v.abs()!=1.||power==0{out.push_str(&scalar_display(v.abs(),float));if power>0{out.push(' ');}}
        if power==1{out.push('s');}else if power>1{out.push_str(&format!("s^{{{power}}}"));}
    }if out.is_empty(){"0".into()}else{out}
}
fn factored_display(p:&[f64],float:bool)->String{
    let mut p=poly::trim(p);if p.len()==1{return scalar_display(p[0],float);}
    let mut factors:Vec<Vec<f64>>=vec![];let mut zeros=0;while p.len()>1&&p.last()==Some(&0.){p.pop();zeros+=1;}
    // Rational linear factors are verified by synthetic division; no approximate
    // complex roots are substituted into an exact displayed factorization.
    'outer:while p.len()>2{
        for denominator in 1..=32{for numerator in -128..=128{
            let root=numerator as f64/denominator as f64;
            let mut q=vec![p[0]];for i in 1..p.len()-1{q.push(p[i]+q[i-1]*root);}
            let remainder=p[p.len()-1]+q[q.len()-1]*root;
            if remainder==0.{factors.push(vec![1.,-root]);p=q;continue 'outer;}
        }}break;
    }
    let mut scale=p[0];for v in &mut p{*v/=scale;}
    if p.len()>1{factors.push(p);}
    factors.sort_by(|a,b|{
        let normalized=|p:&Vec<f64>|{let scale=p.iter().map(|x|x.abs()).fold(0.,f64::max);p.iter().map(|x|x/scale).collect::<Vec<_>>()};
        a.len().cmp(&b.len()).then_with(||if float{normalized(a).partial_cmp(&normalized(b)).unwrap_or(std::cmp::Ordering::Equal)}else{a[1..].partial_cmp(&b[1..]).unwrap_or(std::cmp::Ordering::Equal)})
    });
    if !float&&scale==1.&&zeros==0&&factors.len()==1{return poly_display(&factors[0],false);}
    let integral_residual=float&&zeros>0&&scale==1.&&factors.iter().all(|p|p.iter().map(|v|v.abs()).fold(0.,f64::max)==1.);
    let mut rendered:Vec<String>=vec![];
    for factor in factors{
        let normalization=if float{factor.iter().map(|v|v.abs()).fold(0.,f64::max)}else{1.};
        scale*=normalization;let normalized:Vec<_>=factor.iter().map(|v|v/normalization).collect();
        let value=poly_display(&normalized,float&&!integral_residual);let value=if value.contains(" + ")||value.contains(" - "){format!("\\left({value}\\right)")}else{value};rendered.push(value);
    }
    let mut out=String::new();if scale!=1.||float{out.push_str(&scalar_display(scale,float));}
    if zeros>0{if !out.is_empty(){out.push(' ');}if zeros==1{out.push('s');}else{out.push_str(&format!("s^{{{zeros}}}"));}}
    let mut i=0;while i<rendered.len(){let value=&rendered[i];let count=rendered[i..].iter().take_while(|v|*v==value).count();if !out.is_empty(){out.push(' ');}out.push_str(value);if count>1{out.push_str(&format!("^{{{count}}}"));}i+=count;}
    if out.is_empty(){"1".into()}else{out}
}

pub fn polynomial_latex(p:&[f64])->String{
    let mut out=String::new();
    for (i,&c) in p.iter().enumerate(){
        if c==0. {continue;}
        let degree=p.len()-1-i;
        if !out.is_empty(){out.push_str(if c<0.{" - "}else{" + "});}else if c<0.{out.push('-');}
        let v=c.abs();if degree==0||v!=1.{out.push_str(&format!("{v}"));if degree>0{out.push(' ');}}
        if degree==1{out.push('s');}else if degree>1{out.push_str(&format!("s^{{{degree}}}"));}
    }
    if out.is_empty(){"0".into()}else{out}
}

pub fn normalize(source:&str)->Result<String>{
    let mut source=source.trim();
    if let Some((label,expr))=source.split_once('=').or_else(||source.split_once(':')){
        let label=label.replace(' ',"").to_uppercase();
        let mut rest=label.as_str();while let Some(t)=rest.strip_prefix("G(S)").or_else(||rest.strip_prefix("H(S)")).or_else(||rest.strip_prefix("L(S)")){rest=t;}
        if !rest.is_empty(){return Err("Rótulo da função de transferência inválido.".into());}source=expr.trim();
    }
    if source.is_empty(){return Err("A expressão não pode estar vazia.".into());}
    let chars:Vec<char>=source.chars().collect();let mut out=String::new();let mut i=0;
    while i<chars.len(){
        if "⁰¹²³⁴⁵⁶⁷⁸⁹⁻".contains(chars[i]){
            out.push_str("^(");while i<chars.len(){let ch=chars[i];if let Some(pos)="⁰¹²³⁴⁵⁶⁷⁸⁹".chars().position(|x|x==ch){out.push(char::from(b'0'+pos as u8));}else if ch=='⁻'{out.push('-');}else{break;}i+=1;}out.push(')');continue;
        }
        let ch=match chars[i]{'−'|'–'=>'-','×'|'·'=>'*','÷'=>'/','['=>'(',']'=>')','S'=>'s',',' if i>0&&i+1<chars.len()&&chars[i-1].is_ascii_digit()&&chars[i+1].is_ascii_digit()=>'.',ch=>ch};
        out.push(ch);i+=1;
    }
    Ok(out)
}

#[derive(Clone,Debug,PartialEq)]enum Token{Number(f64),S,Plus,Minus,Mul,Div,Pow,Left,Right}
fn lex(s:&str)->Result<Vec<Token>>{
    let b=s.as_bytes();let mut i=0;let mut result=vec![];
    while i<b.len(){let c=b[i];if c.is_ascii_whitespace(){i+=1;continue;}
        if c.is_ascii_digit()||c==b'.'{let start=i;i+=1;while i<b.len()&&(b[i].is_ascii_digit()||b[i]==b'.'){i+=1;}
            if i<b.len()&&(b[i]==b'e'||b[i]==b'E'){i+=1;if i<b.len()&&(b[i]==b'+'||b[i]==b'-'){i+=1;}while i<b.len()&&b[i].is_ascii_digit(){i+=1;}}
            let v=s[start..i].parse::<f64>().map_err(|_|"Número inválido.")?;if !v.is_finite(){return Err("Número não finito.".into());}result.push(Token::Number(v));continue;
        }
        let t=match c{b's'=>Token::S,b'+'=>Token::Plus,b'-'=>Token::Minus,b'*'=>{if i+1<b.len()&&b[i+1]==b'*'{i+=1;Token::Pow}else{Token::Mul}},b'/'=>{if i+1<b.len()&&b[i+1]==b'/'{return Err("Use / para divisão; o operador // não é suportado.".into());}Token::Div},b'^'=>Token::Pow,b'('=>Token::Left,b')'=>Token::Right,_=>return Err("A expressão contém símbolos não suportados. Use números, s, parênteses e os operadores +, -, *, / ou ^.".into())};result.push(t);i+=1;
    }Ok(result)
}
struct Parser{tokens:Vec<Token>,position:usize}
impl Parser{
    fn peek(&self)->Option<&Token>{self.tokens.get(self.position)}
    fn take(&mut self)->Option<Token>{let t=self.peek().cloned();self.position+=usize::from(t.is_some());t}
    fn expression(&mut self)->Result<TransferFunction>{let mut v=self.product()?;loop{let sign=match self.peek(){Some(Token::Plus)=>1.,Some(Token::Minus)=>-1.,_=>break};self.take();v=v.add(&self.product()?,sign)?;}Ok(v)}
    fn product(&mut self)->Result<TransferFunction>{let mut v=self.unary()?;loop{match self.peek(){Some(Token::Mul)=>{self.take();v=v.multiply(&self.unary()?)?},Some(Token::Div)=>{self.take();v=v.divide(&self.unary()?)?},Some(Token::Number(_)|Token::S|Token::Left)=>v=v.multiply(&self.unary()?)?,_=>break}}Ok(v)}
    fn unary(&mut self)->Result<TransferFunction>{match self.peek(){Some(Token::Plus)=>{self.take();self.unary()},Some(Token::Minus)=>{self.take();self.unary()?.scale(-1.)},_=>self.power()}}
    fn power(&mut self)->Result<TransferFunction>{let mut v=self.atom()?;if self.peek()==Some(&Token::Pow){self.take();let exp=self.unary()?;if exp.num.len()!=1||exp.den.len()!=1{return Err("O expoente deve ser inteiro.".into());}let exp=exp.num[0]/exp.den[0];if exp.fract()!=0.||exp.abs()>4096.{return Err("O expoente deve ser inteiro e respeitar o limite de recursos.".into());}let invert=exp<0.;let base=v;v=TransferFunction::scalar(1.);for _ in 0..exp.abs() as usize{v=v.multiply(&base)?;}if invert{v=TransferFunction::scalar(1.).divide(&v)?;}}Ok(v)}
    fn atom(&mut self)->Result<TransferFunction>{match self.take(){Some(Token::Number(x))=>Ok(TransferFunction::scalar(x)),Some(Token::S)=>TransferFunction::new(vec![1.,0.],vec![1.]),Some(Token::Left)=>{let v=self.expression()?;if self.take()!=Some(Token::Right){return Err("Parênteses não balanceados.".into());}Ok(v)},_=>Err("Expressão inválida.".into())}}
}
