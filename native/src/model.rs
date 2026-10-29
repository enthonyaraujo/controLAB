use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub type Schema = BTreeMap<String, Module>;
#[derive(Clone, Deserialize)]
pub struct Module {
    pub action: String,
    pub title: String,
    pub fields: Vec<Field>,
    /// Base name of exported figures (`MODULES[*].filename`).
    #[serde(default)]
    pub filename: Option<String>,
    /// Metric labels highlighted as cards, matched case-insensitively by substring.
    #[serde(default, rename = "primaryKeys")]
    pub primary_keys: Vec<String>,
}
#[derive(Clone, Deserialize)]
pub struct Field {
    pub name: String,
    pub label: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub value: Value,
    #[serde(default)]
    pub help: String,
    #[serde(default)]
    pub placeholder: String,
    #[serde(default)]
    pub options: Vec<[String; 2]>,
    #[serde(default)]
    pub when: BTreeMap<String, Vec<String>>,
    pub min: Option<String>,
    pub max: Option<String>,
}
impl Field {
    pub fn visible(&self, values: &BTreeMap<String, String>) -> bool {
        self.when
            .iter()
            .all(|(key, allowed)| values.get(key).is_some_and(|v| allowed.contains(v)))
    }
}
impl Module {
    pub fn defaults(&self) -> BTreeMap<String, String> {
        self.fields
            .iter()
            .map(|f| {
                let value = f
                    .value
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| f.value.to_string());
                // Number inputs displayed the pt-BR decimal comma in the previous renderer.
                let value = if f.kind == "number" {
                    value.replace('.', ",")
                } else {
                    value
                };
                (f.name.clone(), value)
            })
            .collect()
    }
    pub fn payload(&self, values: &BTreeMap<String, String>, light: bool) -> Result<Value, String> {
        let mut request = json!({"action":self.action,"theme":if light{"light"}else{"dark"}});
        for f in &self.fields {
            if !f.visible(values) {
                continue;
            }
            let text = values.get(&f.name).map(|s| s.trim()).unwrap_or_default();
            if text.is_empty() {
                continue;
            }
            let v = match f.kind.as_str() {
                "number" => {
                    let n = text
                        .replace(',', ".")
                        .parse::<f64>()
                        .map_err(|_| format!("{}: informe um número válido", f.label))?;
                    if !n.is_finite()
                        || f.min
                            .as_ref()
                            .and_then(|s| s.parse().ok())
                            .is_some_and(|min: f64| n < min)
                        || f.max
                            .as_ref()
                            .and_then(|s| s.parse().ok())
                            .is_some_and(|max: f64| n > max)
                    {
                        return Err(format!("{}: valor fora do intervalo permitido", f.label));
                    }
                    json!(n)
                }
                "checkbox" => json!(text == "true"),
                "hidden" if text == "true" || text == "false" => json!(text == "true"),
                _ => json!(text),
            };
            request[&f.name] = v;
        }
        Ok(request)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn schema() -> Schema {
        serde_json::from_str(include_str!("../modules.json")).unwrap()
    }
    #[test]
    fn six_modules_have_valid_defaults() {
        let s = schema();
        assert_eq!(s.len(), 6);
        for m in s.values() {
            let p = m.payload(&m.defaults(), false).unwrap();
            assert_eq!(p["action"], m.action);
            assert_eq!(p["theme"], "dark");
        }
    }
    #[test]
    fn rejects_nonfinite_and_out_of_range_parameters() {
        let m = &schema()["time"];
        for v in ["NaN", "inf", "-1", "10001", "abc"] {
            let mut values = m.defaults();
            values.insert("final_time".into(), v.into());
            assert!(m.payload(&values, false).is_err(), "{v}");
        }
    }
    #[test]
    fn hidden_branches_are_not_submitted() {
        let m = &schema()["state-space"];
        let mut values = m.defaults();
        assert!(m.payload(&values, false).unwrap().get("expr").is_none());
        values.insert("mode".into(), "transfer".into());
        let p = m.payload(&values, true).unwrap();
        assert!(p.get("A").is_none());
        assert!(p.get("expr").is_some());
        assert_eq!(p["theme"], "light");
    }
}
