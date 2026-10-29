//! Migration tools. Commands never report incomplete semantic/visual parity as a pass.
use serde_json::{Value,json};
use std::{env,fs,path::PathBuf,process::ExitCode};

fn compare(expected:&Value,actual:&Value,path:&str,diffs:&mut Vec<String>,assets:&mut usize){
    if expected.get("$asset").is_some(){*assets+=1;if !actual.is_string(){diffs.push(format!("{path}: imagem/SVG ausente"));}return;}
    match(expected,actual){
        (Value::Number(a),Value::Number(b))=>{let(a,b)=(a.as_f64().unwrap(),b.as_f64().unwrap());if(a-b).abs()>1e-9+1e-7*a.abs(){diffs.push(format!("{path}: esperado {a}, recebido {b}"));}},
        (Value::Array(a),Value::Array(b))=>{if a.len()!=b.len(){diffs.push(format!("{path}: tamanhos {} ≠ {}",a.len(),b.len()));}for(i,(a,b))in a.iter().zip(b).enumerate(){compare(a,b,&format!("{path}[{i}]"),diffs,assets);}},
        (Value::Object(a),Value::Object(b))=>{for(key,value)in a{if let Some(other)=b.get(key){compare(value,other,&format!("{path}.{key}"),diffs,assets);}else{diffs.push(format!("{path}.{key}: campo ausente"));}}for key in b.keys(){if !a.contains_key(key){diffs.push(format!("{path}.{key}: campo adicional"));}}},
        _ if expected==actual=>{},
        _=>diffs.push(format!("{path}: esperado {}, recebido {}",short(expected),short(actual))),
    }
}
fn short(v:&Value)->String{let s=v.to_string();let mut out:String=s.chars().take(160).collect();if s.chars().count()>160{out.push('…');}out}

fn parity(args:&[String])->Result<bool,String>{
    let option=|flag:&str|args.iter().position(|s|s==flag).and_then(|i|args.get(i+1));
    let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/reference/python");
    let manifest:Value=serde_json::from_slice(&fs::read(root.join("manifest.json")).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    let section=option("--section");if section.is_some_and(|s|s!="details"){return Err("--section aceita apenas details; omita para comparar resposta completa.".into());}
    let mut reports=vec![];let mut matched=0;let mut failed=0;let mut assets=0;
    for entry in manifest["cases"].as_array().ok_or("Manifesto sem casos.")?{
        if option("--module").is_some_and(|s|entry["module"]!=s.as_str())||option("--case").is_some_and(|s|entry["id"]!=s.as_str()){continue;}
        let fixture:Value=serde_json::from_slice(&fs::read(root.join(entry["path"].as_str().ok_or("Caso sem caminho.")?)).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        let actual=controlab_engine::dispatch::dispatch(&fixture["input"]);let expected=&fixture["output"];
        let mut diffs=vec![];let mut pending_assets=0;
        if section.is_some(){compare(&expected["success"],&actual["success"],"$.success",&mut diffs,&mut pending_assets);if expected["success"]==true&&expected.get("details").is_none(){diffs.push("Caso não possui details: não é verificável nesta seleção.".into());}if expected["success"]==false{compare(&expected["error"],&actual["error"],"$.error",&mut diffs,&mut pending_assets);}else{compare(&expected["details"],&actual["details"],"$.details",&mut diffs,&mut pending_assets);}}
        else{compare(expected,&actual,"$",&mut diffs,&mut pending_assets);}
        assets+=pending_assets;if diffs.is_empty(){matched+=1;}else{failed+=1;}
        reports.push(json!({"case":entry["id"],"module":entry["module"],"differences":diffs,"visual_assets_unverified":pending_assets}));
    }
    if reports.is_empty(){return Err("Nenhum caso corresponde aos filtros.".into());}
    let report=json!({"scope":section.map_or("response",String::as_str),"matched":matched,"failed":failed,"visual_assets_unverified":assets,"complete_parity":section.is_none()&&failed==0&&assets==0,"cases":reports});
    println!("{}",serde_json::to_string_pretty(&report).unwrap());
    // A partial details check can succeed for its stated scope; a full run with
    // unchecked assets cannot be used as evidence of complete parity.
    Ok(failed==0&&(section.is_some()||assets==0))
}

fn main()->ExitCode{
    let args:Vec<String>=env::args().skip(1).collect();
    let result=match args.first().map(String::as_str){Some("parity")=>parity(&args[1..]),Some("version") if args.len()==1=>{println!("{}",env!("CARGO_PKG_VERSION"));Ok(true)},_=>Err("Uso: cargo xtask parity [--module nome] [--case id] [--section details]\n     cargo xtask version".into())};
    match result{Ok(true)=>ExitCode::SUCCESS,Ok(false)=>ExitCode::FAILURE,Err(e)=>{eprintln!("{e}");ExitCode::FAILURE}}
}

#[cfg(test)]mod tests{use super::*;
    #[test]fn tolerance_does_not_hide_types_or_missing_fields(){let mut d=vec![];let mut a=0;compare(&json!({"v":1.}),&json!({"v":1.+1e-8}),"$",&mut d,&mut a);assert!(d.is_empty());compare(&json!({"v":1.}),&json!({"v":"1"}),"$",&mut d,&mut a);assert_eq!(d.len(),1);compare(&json!({"flag":true}),&json!({}),"$",&mut d,&mut a);assert_eq!(d.len(),2);}
    #[test]fn visual_assets_are_explicitly_unverified(){let mut d=vec![];let mut a=0;compare(&json!({"$asset":"a.png"}),&json!("data:image/png;base64,..."),"$",&mut d,&mut a);assert_eq!(a,1);assert!(d.is_empty());}
}
