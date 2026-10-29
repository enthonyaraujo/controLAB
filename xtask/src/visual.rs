//! Visual check of the frozen Matplotlib figures. Rust redraws each figure with
//! its own renderer, so geometry, fonts and line widths are not compared. Every
//! non-numeric label of the Python SVG (title, axes, legend, annotation) and
//! every non-neutral color must be present in the Rust SVG, with nothing extra;
//! PNGs must decode with the same transport encoding.
use base64::{Engine, engine::general_purpose::STANDARD};
use regex::Regex;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    sync::OnceLock,
};

pub const CRITERION: &str = "SVG: textos não numéricos e cores não neutras iguais ao Python; PNG: imagem válida na mesma codificação. Geometria, fontes e espessuras não são comparadas.";

pub fn check(root: &Path, expected: &Value, actual: &Value, path: &str, diffs: &mut Vec<String>) {
    let Some(actual) = actual.as_str() else {
        diffs.push(format!("{path}: imagem/SVG ausente"));
        return;
    };
    let asset = expected["$asset"].as_str().unwrap_or("");
    if asset.ends_with(".svg") {
        match fs::read_to_string(root.join(asset)) {
            Ok(reference) => compare_svg(&reference, actual, path, diffs),
            Err(e) => diffs.push(format!("{path}: referência {asset} ilegível: {e}")),
        }
        return;
    }
    let encoding = expected["encoding"].as_str().unwrap_or("");
    let data = match encoding.strip_prefix("data:") {
        Some(_) => actual.strip_prefix(&format!("{encoding},")),
        None => Some(actual),
    };
    let png = data.and_then(|d| STANDARD.decode(d).ok());
    if !png.is_some_and(|b| b.starts_with(b"\x89PNG\r\n\x1a\n")) {
        diffs.push(format!(
            "{path}: PNG inválido ou fora da codificação {encoding}"
        ));
    }
}

fn compare_svg(reference: &str, actual: &str, path: &str, diffs: &mut Vec<String>) {
    let mut missing = labels(reference);
    let mut extra = labels(actual);
    for key in missing.keys().cloned().collect::<Vec<_>>() {
        let both = missing[&key].min(extra.get(&key).copied().unwrap_or(0));
        take(&mut missing, &key, both);
        take(&mut extra, &key, both);
    }
    // Matplotlib writes each line of a multi-line annotation as its own text.
    for key in extra.keys().cloned().collect::<Vec<_>>() {
        while extra.get(&key).copied().unwrap_or(0) > 0 {
            let parts: Vec<String> = missing.keys().cloned().collect();
            let pair = parts.iter().enumerate().find_map(|(i, a)| {
                parts[i + 1..]
                    .iter()
                    .find(|b| sorted(&format!("{a}{b}")) == key)
                    .map(|b| (a.clone(), b.clone()))
            });
            let Some((a, b)) = pair else { break };
            take(&mut missing, &a, 1);
            take(&mut missing, &b, 1);
            take(&mut extra, &key, 1);
        }
    }
    for (side, left) in [("ausente", &missing), ("adicional", &extra)] {
        for (key, count) in left {
            diffs.push(format!(
                "{path}: texto {side} ({count}×) com caracteres «{key}»"
            ));
        }
    }
    let (expected, got) = (colors(reference), colors(actual));
    for color in expected.difference(&got) {
        diffs.push(format!("{path}: cor {color} ausente"));
    }
    for color in got.difference(&expected) {
        diffs.push(format!("{path}: cor {color} adicional"));
    }
}

fn take(map: &mut BTreeMap<String, usize>, key: &str, n: usize) {
    if let Some(v) = map.get_mut(key) {
        *v -= n;
        if *v == 0 {
            map.remove(key);
        }
    }
}

/// Order-insensitive key: Matplotlib mathtext emits glyphs out of reading order.
fn sorted(s: &str) -> String {
    let mut chars: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).collect();
    chars.sort_unstable();
    chars.into_iter().collect()
}

fn labels(svg: &str) -> BTreeMap<String, usize> {
    static TEXT: OnceLock<[Regex; 3]> = OnceLock::new();
    let [text, tag, number] = TEXT.get_or_init(|| {
        [
            Regex::new(r"(?s)<text\b[^>]*>(.*?)</text>").unwrap(),
            Regex::new(r"<[^>]+>").unwrap(),
            Regex::new(r"^([-+]?(\d+([.,]\d*)?|[.,]\d+)(e[-+]?\d+)?|10[-+]?\d+)?$").unwrap(),
        ]
    });
    let mut out = BTreeMap::new();
    for capture in text.captures_iter(svg) {
        let plain = unescape(&tag.replace_all(&capture[1], ""))
            // A literal "j\omega" in a Python annotation means jω.
            .replace(r"\omega", "ω")
            .replace(['$', '\\', '_'], "")
            .replace('−', "-")
            .replace('∘', "°");
        let compact: String = plain.chars().filter(|c| !c.is_whitespace()).collect();
        if !number.is_match(&compact) {
            *out.entry(sorted(&plain)).or_insert(0) += 1;
        }
    }
    out
}

fn unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

fn colors(svg: &str) -> BTreeSet<String> {
    static COLOR: OnceLock<Regex> = OnceLock::new();
    const NEUTRAL: [&str; 19] = [
        "#111827", "#CBD5E1", "#64748B", "#334155", "#E5E7EB", "#9CA3AF", "#6B7280", "#D1D5DB",
        "#F3F4F6", "#374151", "#1F2937", "#4B5563", "#E2E8F0", "#94A3B8", "#475569", "#1E293B",
        "#0F172A", "#F8FAFC", "#F1F5F9",
    ];
    let color = COLOR.get_or_init(|| {
        Regex::new(r#"(?:stroke|fill)\s*[:=]\s*"?\s*(#[0-9a-fA-F]{6})\b"#).unwrap()
    });
    color
        .captures_iter(svg)
        .map(|c| c[1].to_ascii_uppercase())
        .filter(|c| {
            let gray = c[1..3] == c[3..5] && c[3..5] == c[5..7];
            !gray && !NEUTRAL.contains(&c.as_str())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_ignore_ticks_markup_and_glyph_order() {
        let python = r#"<text>−1.5</text><text>10−2</text><text>E i x o ( ) σ</text><text>Break: -0.55</text><text>(K=0.6)</text>"#;
        let rust = r#"<text>−1.5</text><text>Eixo (σ)</text><text>Break: -0.55<tspan>(K=0.6)</tspan></text>"#;
        let mut d = vec![];
        compare_svg(python, rust, "$", &mut d);
        assert!(d.is_empty(), "{d:?}");
        compare_svg(python, r#"<text>Eixo (σ)</text>"#, "$", &mut d);
        assert_eq!(d.len(), 2, "{d:?}");
    }

    #[test]
    fn non_neutral_colors_must_match() {
        let mut d = vec![];
        compare_svg(
            r##"<path stroke="#F59E0B"/><path style="fill: #64748b"/>"##,
            r##"<path stroke="#000000"/>"##,
            "$",
            &mut d,
        );
        assert_eq!(d, ["$: cor #F59E0B ausente"]);
    }
}
