//! Native TeX composition. Keeps the existing batch/slot/base64 contract.
use crate::{Result,presentation};
use latex_rust::{MathFont,MathStyle,Dim,SvgOptions,Color};
use serde_json::{Value,json};
use std::sync::OnceLock;

fn font()->&'static MathFont{
    static FONT:OnceLock<MathFont>=OnceLock::new();
    FONT.get_or_init(||MathFont::from_bytes(include_bytes!("../../../assets/fonts/katex/KaTeX_Main-Regular.ttf")).expect("bundled KaTeX font"))
}
fn fallback_font()->&'static MathFont{
    static FONT:OnceLock<MathFont>=OnceLock::new();FONT.get_or_init(||MathFont::stix_two_math().expect("embedded math font"))
}
fn one(source:&str,font:&MathFont)->Result<(String,f64)>{
    let source=source.trim().trim_matches('$');let inline=source.starts_with(r"\textstyle");
    let source=if inline{source.trim_start_matches(r"\textstyle")}else{source};
    let ast=latex_rust::parse(source).map_err(|e|e.to_string())?;
    let tree=latex_rust::layout_with_em_size_pt(&ast,font,if inline{MathStyle::Text}else{MathStyle::Display},&Dim::from_i64(36)).map_err(|e|e.to_string())?;
    let dim=|v:&Dim|v.as_ratio().map(|(n,d)|n as f64/d as f64).unwrap_or(0.);
    let depth=dim(&tree.depth);let height=dim(&tree.height)+depth;
    if height<=0.||dim(&tree.width)<=0.{return Err("Fórmula vazia.".into());}
    let options=SvgOptions{font_size_pt:Dim::from_i64(36),color:Color::rgb(255,255,255),display:!inline};
    let svg=latex_rust::render_svg(&tree,font,&options).map_err(|e|e.to_string())?;
    let image=presentation::rasterize(&svg)?;
    Ok((image.trim_start_matches("data:image/png;base64,").into(),depth/height))
}
pub fn render(payload:&Value)->Result<Value>{
    let sources=payload["sources"].as_array().ok_or("sources deve ser uma lista de fórmulas TeX.")?;
    let mut images=vec![];let mut depths=vec![];
    for source in sources.iter().take(200){
        let result=source.as_str().filter(|s|!s.trim().is_empty()&&s.chars().count()<=4096).and_then(|s|one(s,font()).or_else(|_|one(s,fallback_font())).ok());
        if let Some((image,depth))=result{images.push(json!(image));depths.push(json!(depth));}else{images.push(Value::Null);depths.push(json!(0));}
    }
    Ok(json!({"success":true,"images":images,"depths":depths,"em":48}))
}
