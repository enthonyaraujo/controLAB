//! Request-local localization with the existing catalog and protected fields.
use crate::Result;
use regex::Regex;
use serde_json::Value;
use std::{collections::BTreeMap, sync::OnceLock};

const PROTECTED: &[&str] = &[
    "source",
    "code",
    "python_code",
    "command",
    "output",
    "stdout",
    "stderr",
    "path",
    "folder",
    "name",
    "filename",
    "value",
    "expression",
    "expr",
    "type",
    "action",
    "session",
    "operation",
    "image",
    "plot_image",
    "svg",
    "plot_svg",
    "base64",
    "variables",
    "workspace",
    "files",
    "recent",
];
struct Catalog {
    map: BTreeMap<String, String>,
    pattern: Regex,
}
fn catalog() -> &'static Catalog {
    static C: OnceLock<Catalog> = OnceLock::new();
    C.get_or_init(|| {
        let map: BTreeMap<String, String> =
            serde_json::from_str(include_str!("../../../assets/locales/en.json"))
                .expect("canonical translations");
        let mut keys: Vec<_> = map
            .iter()
            .filter(|(k, v)| k != v)
            .map(|(k, _)| k.as_str())
            .collect();
        keys.sort_by_key(|s| std::cmp::Reverse(s.len()));
        let pattern = Regex::new(
            &keys
                .iter()
                .map(|k| regex::escape(k))
                .collect::<Vec<_>>()
                .join("|"),
        )
        .expect("escaped translation catalog");
        Catalog { map, pattern }
    })
}
fn word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
fn translate(text: &str) -> String {
    let c = catalog();
    if let Some(s) = c.map.get(text).filter(|s| !s.is_empty()) {
        return s.clone();
    }
    let mut out = String::new();
    let mut end = 0;
    for m in c.pattern.find_iter(text) {
        if text[..m.start()].chars().next_back().is_some_and(word)
            || text[m.end()..].chars().next().is_some_and(word)
        {
            continue;
        }
        out.push_str(&text[end..m.start()]);
        out.push_str(&c.map[m.as_str()]);
        end = m.end();
    }
    out.push_str(&text[end..]);
    out
}
fn walk(value: &mut Value) -> Result<()> {
    match value {
        Value::String(s) => *s = translate(s),
        Value::Array(a) => {
            for v in a {
                walk(v)?;
            }
        }
        Value::Object(o) => {
            for (k, v) in o.iter_mut() {
                if !PROTECTED.contains(&k.as_str()) {
                    walk(v)?;
                }
            }
            static TEXT: OnceLock<Regex> = OnceLock::new();
            let re = TEXT.get_or_init(|| Regex::new(r"(<text\b[^>]*>)([^<]*)(</text>)").unwrap());
            for (svg_key, image_key) in [("svg", "image"), ("plot_svg", "plot_image")] {
                if let Some(svg) = o.get(svg_key).and_then(Value::as_str) {
                    // Only translate SVG text nodes, never paths, formulas or IDs.
                    let translated = re
                        .replace_all(svg, |caps: &regex::Captures| {
                            format!("{}{}{}", &caps[1], translate(&caps[2]), &caps[3])
                        })
                        .to_string();
                    if translated != svg {
                        let png = crate::presentation::rasterize(&translated)?;
                        o.insert(svg_key.into(), Value::String(translated));
                        if o.contains_key(image_key) {
                            o.insert(image_key.into(), Value::String(png));
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}
pub fn translate_result(mut value: Value, language: &str) -> Result<Value> {
    if language == "en" {
        walk(&mut value)?;
    }
    Ok(value)
}
