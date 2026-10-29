//! Bounded JSON-lines transport for differential migration tests.
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};

const MAX_REQUEST: usize = 8 * 1024 * 1024;
const MAX_RESPONSE: usize = 32 * 1024 * 1024;

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    loop {
        let mut raw = Vec::new();
        let mut oversize = false;
        loop {
            let available = input.fill_buf()?;
            if available.is_empty() {
                break;
            }
            let end = available
                .iter()
                .position(|&c| c == b'\n')
                .map(|i| i + 1)
                .unwrap_or(available.len());
            let last = available[end - 1] == b'\n';
            if raw.len() + end > MAX_REQUEST {
                oversize = true;
            }
            if !oversize {
                raw.extend_from_slice(&available[..end]);
            }
            input.consume(end);
            if last {
                break;
            }
        }
        if raw.is_empty() && !oversize {
            break;
        }
        let reply = if oversize {
            json!({"success":false,"error":"Requisição excede o limite de 8 MiB."})
        } else {
            match serde_json::from_slice::<Value>(&raw) {
                Ok(value) => controlab_engine::dispatch::dispatch(&value),
                Err(error) => json!({"success":false,"error":format!("JSON inválido: {error}")}),
            }
        };
        let mut response = serde_json::to_vec(&reply)?;
        if response.len() > MAX_RESPONSE {
            response = serde_json::to_vec(
                &json!({"success":false,"error":"Resposta excede o limite de 32 MiB."}),
            )?;
        }
        output.write_all(&response)?;
        output.write_all(b"\n")?;
        output.flush()?;
    }
    Ok(())
}
