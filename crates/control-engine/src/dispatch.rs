//! Migration dispatcher: unsupported actions fail explicitly, never fall back to Python.
//! Production UI integration is gated on full parity, separately from this executable.
use crate::{Result,input,time_response,frequency_response,routh};
use serde_json::{Value,json};

pub fn dispatch(payload:&Value)->Value{
    let result=match request(payload){Ok(value)=>value,Err(error)=>json!({"success":false,"error":format!("ValueError: {error}")})};
    if matches!(payload["action"].as_str(),Some("render_formulas"|"check_updates")){return result;}
    crate::localization::translate_result(result,payload["language"].as_str().unwrap_or("pt-BR")).unwrap_or_else(|error|json!({"success":false,"error":error}))
}
pub fn request(payload:&Value)->Result<Value>{
    if !payload.is_object(){return Err("A requisição deve ser um objeto JSON.".into());}
    match payload["action"].as_str().unwrap_or("calculate"){
        "preview"=>{let tf=input::transfer(payload)?;let (expanded,factored)=tf.latex_formatted(!matches!(payload["mode"].as_str().unwrap_or("expr"),"expr"|"parts"));Ok(json!({"success":true,"num":tf.num,"den":tf.den,"latex_exp":expanded,"latex_fac":factored}))},
        "calculate"=>crate::lgr::analyze(payload),
        "presets"=>Ok(crate::lgr::presets()),
        "state_space"=>crate::state_space::analyze(payload),
        "state_space_presets"=>Ok(json!({"success":true,"presets":crate::state_space::presets()})),
        "time_response"=>time_response::analyze(payload),
        "time_response_presets"=>Ok(json!({"success":true,"presets":time_response::presets()})),
        "frequency_response"=>frequency_response::analyze(payload),
        "frequency_response_presets"=>Ok(json!({"success":true,"presets":frequency_response::presets()})),
        "routh_hurwitz"=>routh::analyze(payload),
        "routh_evaluate_k"=>routh::evaluate_k(payload),
        "routh_presets"=>Ok(routh::presets()),
        "render_formulas"=>crate::formulas::render(payload),
        action=>Err(format!("Ação ainda não migrada para Rust: {action}")),
    }
}
