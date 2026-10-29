//! Bounded read-only GitHub release lookup, matching the existing settings action.
use serde_json::{Value, json};
use std::time::Duration;
fn connection_error() -> Value {
    json!({"success":false,"error":"Verifique sua conexão à internet."})
}
pub fn decode_release(status: u16, body: &[u8]) -> Value {
    if status == 403 {
        return json!({"success":false,"error":"Limite de requisições da API do GitHub atingido temporariamente. Tente novamente mais tarde."});
    }
    if status >= 400 {
        return json!({"success":false,"error":format!("O GitHub retornou HTTP {status}")});
    }
    let Ok(release) = serde_json::from_slice::<Value>(body) else {
        return connection_error();
    };
    if !release.is_object() {
        return connection_error();
    }
    json!({"success":true,"tag_name":release.get("tag_name").cloned().unwrap_or(json!("")),"html_url":release.get("html_url").cloned().unwrap_or(json!("")),"published_at":release.get("published_at").cloned().unwrap_or(json!(""))})
}
pub fn check() -> Value {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(15)))
        .https_only(true)
        .build();
    let agent = ureq::Agent::new_with_config(config);
    match agent
        .get("https://api.github.com/repos/enthonyaraujo/controlab/releases/latest")
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "ControLAB")
        .call()
    {
        Ok(mut response) => match response
            .body_mut()
            .with_config()
            .limit(1024 * 1024)
            .read_to_vec()
        {
            Ok(bytes) => decode_release(200, &bytes),
            Err(_) => connection_error(),
        },
        Err(ureq::Error::StatusCode(status)) => decode_release(status, &[]),
        Err(_) => connection_error(),
    }
}
