//! Input modes exposed by the existing scientific dispatcher.
use crate::{Result, polynomial, transfer::TransferFunction};
use num_complex::Complex64;
use serde_json::Value;

pub fn coefficients(source: &str) -> Result<Vec<f64>> {
    let source = source.replace(['[', ']', ','], " ");
    if source.trim().is_empty() {
        return Err("A lista de coeficientes não pode estar vazia.".into());
    }
    source
        .split_whitespace()
        .map(|s| {
            s.parse::<f64>()
                .map_err(|_| format!("could not convert string to float: '{s}'"))
        })
        .collect()
}

pub fn complex(source: &str) -> Result<Complex64> {
    let mut s = source.trim().replace('i', "j");
    if s.starts_with('(') && s.ends_with(')') {
        s = s[1..s.len() - 1].to_owned();
    }
    if !s.ends_with('j') {
        return s
            .parse::<f64>()
            .map(|r| Complex64::new(r, 0.))
            .map_err(|_| format!("Número complexo inválido: {source}"));
    }
    s.pop();
    let split = s
        .char_indices()
        .filter(|&(i, c)| {
            i > 0 && (c == '+' || c == '-') && !matches!(s.as_bytes()[i - 1], b'e' | b'E')
        })
        .map(|(i, _)| i)
        .next_back();
    let (re, im) = if let Some(i) = split {
        (&s[..i], &s[i..])
    } else {
        ("0", s.as_str())
    };
    let im = match im {
        "" | "+" => 1.,
        "-" => -1.,
        x => x
            .parse()
            .map_err(|_| format!("Número complexo inválido: {source}"))?,
    };
    Ok(Complex64::new(
        re.parse()
            .map_err(|_| format!("Número complexo inválido: {source}"))?,
        im,
    ))
}

pub fn complex_list(source: &str) -> Result<Vec<Complex64>> {
    source
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(complex)
        .collect()
}

pub fn transfer(payload: &Value) -> Result<TransferFunction> {
    let text = |key: &str, default: &str| payload[key].as_str().unwrap_or(default).to_owned();
    match payload["mode"].as_str().unwrap_or("expr") {
        "expr" => TransferFunction::parse(&text("expr", "")),
        "parts" => TransferFunction::parse(&format!(
            "({})/({})",
            text("numerator", "1"),
            text("denominator", "s(s + 1)")
        )),
        "coeffs" => TransferFunction::new(
            coefficients(&text("num", "1, 2"))?,
            coefficients(&text("den", "1, 5, 4, 0"))?,
        ),
        "zpk" => {
            let z = complex_list(&text("zeros", ""))?;
            let p = complex_list(&text("poles", ""))?;
            let k = match &payload["k"] {
                Value::String(s) => s.parse().map_err(|_| "Ganho inválido.")?,
                v => v.as_f64().unwrap_or(1.),
            };
            // Preserve the existing Python implementation: coefficient imaginary
            // components are discarded, even for non-conjugate user input.
            TransferFunction::new(
                polynomial::from_roots(&z, k).iter().map(|v| v.re).collect(),
                polynomial::from_roots(&p, 1.)
                    .iter()
                    .map(|v| v.re)
                    .collect(),
            )
        }
        "preset" => {
            let entries = [
                (
                    "Exemplo 1: 3ª Ordem com Breakaway e Cruzamento jω (Ogata)",
                    "(s+2)/(s(s+1)(s+4))",
                ),
                (
                    "Exemplo 2: Polos Complexos Conjugados e Ângulo de Partida",
                    "1/(s(s^2+2s+2))",
                ),
                (
                    "Exemplo 3: Zero de Fase Não-Mínima (Semiplano Direito)",
                    "(s-2)/(s(s+1)(s+3))",
                ),
                (
                    "Exemplo 4: Sistema com Break-in e Breakaway",
                    "(s+2)/(s(s+1))",
                ),
                (
                    "Exemplo 5: 4ª Ordem com 4 Assíntotas",
                    "1/(s(s+1)(s+2)(s+3))",
                ),
                (
                    "Exemplo 6: Sistema de 2ª Ordem com Amortecimento",
                    "1/(s^2+2s+5)",
                ),
            ];
            let key = payload["preset_key"].as_str().unwrap_or("");
            let expr = entries
                .iter()
                .find(|(name, _)| *name == key)
                .unwrap_or(&entries[0])
                .1;
            let index = entries
                .iter()
                .position(|(_, value)| *value == expr)
                .unwrap_or(0);
            let (num, den) = match index {
                0 => (vec![1., 2.], vec![1., 5., 4., 0.]),
                1 => (vec![1.], vec![1., 2., 2., 0.]),
                2 => (vec![1., -2.], vec![1., 4., 3., 0.]),
                3 => (vec![1., 2.], vec![1., 1., 0.]),
                4 => (vec![1.], vec![1., 6., 11., 6., 0.]),
                _ => (vec![1.], vec![1., 2., 5.]),
            };
            TransferFunction::new(num, den)
        }
        mode => Err(format!("Modo desconhecido: {mode}")),
    }
}

pub fn latex(payload: &Value, tf: &TransferFunction) -> Result<(String, String)> {
    match payload["mode"].as_str().unwrap_or("expr") {
        "expr" => crate::transfer::expression_latex(payload["expr"].as_str().unwrap_or("")),
        "parts" => crate::transfer::expression_latex(&format!(
            "({})/({})",
            payload["numerator"].as_str().unwrap_or("1"),
            payload["denominator"].as_str().unwrap_or("s(s + 1)")
        )),
        _ => Ok(tf.latex_formatted(true)),
    }
}
