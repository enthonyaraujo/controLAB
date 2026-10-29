//! Migration dispatcher: unsupported actions fail explicitly, never fall back to Python.
//! Production UI integration is gated on full parity, separately from this executable.
use crate::{Result, frequency_response, input, routh, time_response};
use serde_json::{Value, json};
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

pub fn dispatch(payload: &Value) -> Value {
    dispatch_with_control(
        payload,
        Arc::new(AtomicBool::new(false)),
        Duration::from_secs(60),
    )
}
pub fn dispatch_with_control(payload: &Value, stop: Arc<AtomicBool>, timeout: Duration) -> Value {
    let result = match crate::execution::run(stop, timeout, || request(payload)) {
        Ok(value) => value,
        Err(error) => {
            json!({"success":false,"error":if error.starts_with("TypeError: ")||error.starts_with("IndexError: "){error}else{format!("ValueError: {error}")}})
        }
    };
    if matches!(
        payload["action"].as_str(),
        Some("render_formulas" | "check_updates")
    ) {
        return result;
    }
    crate::localization::translate_result(result, payload["language"].as_str().unwrap_or("pt-BR"))
        .unwrap_or_else(|error| json!({"success":false,"error":error}))
}
pub fn request(payload: &Value) -> Result<Value> {
    if !payload.is_object() {
        return Err("A requisição deve ser um objeto JSON.".into());
    }
    if std::env::var_os("CONTROLAB_ASSETS").is_some() {
        crate::resources::assets_root()?;
    }
    match payload["action"].as_str().unwrap_or("calculate") {
        "preview" => {
            let tf = input::transfer(payload).map_err(|e| {
                if e == "O denominador não pode ser zero."
                    && payload["mode"].as_str().unwrap_or("expr") == "expr"
                {
                    "TypeError: Cannot convert complex to float".into()
                } else {
                    e
                }
            })?;
            let (expanded, factored) = input::latex(payload, &tf)?;
            Ok(
                json!({"success":true,"num":tf.num,"den":tf.den,"latex_exp":expanded,"latex_fac":factored}),
            )
        }
        "calculate" => crate::lgr::analyze(payload),
        "presets" => Ok(json!({"success":true,"presets":crate::lgr::presets()})),
        "state_space" => crate::state_space::analyze(payload),
        "state_space_presets" => {
            Ok(json!({"success":true,"presets":crate::state_space::presets()}))
        }
        "controller_design" => crate::controllers::analyze(payload),
        "controller_presets" => Ok(crate::controllers::presets()),
        "time_response" => time_response::analyze(payload),
        "time_response_presets" => Ok(json!({"success":true,"presets":time_response::presets()})),
        "frequency_response" => frequency_response::analyze(payload),
        "frequency_response_presets" => {
            Ok(json!({"success":true,"presets":frequency_response::presets()}))
        }
        "routh_hurwitz" => routh::analyze(payload),
        "routh_evaluate_k" => routh::evaluate_k(payload),
        "routh_presets" => Ok(routh::presets()),
        "render_formulas" => crate::formulas::render(payload),
        "check_updates" => Ok(crate::updates::check()),
        action => Err(format!("Ação desconhecida: {action}")),
    }
}
