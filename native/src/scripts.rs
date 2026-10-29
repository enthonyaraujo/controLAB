//! Studio `.m` scripts and documents, run in-process by the mscript interpreter.
//! Requests and replies keep the JSON shape of the other scientific actions.
use mscript::{ExecutionOptions, Session, Value, documents};
use serde_json::{Value as Json, json};
use std::{
    collections::BTreeMap,
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    time::Duration,
};

/// Persistent `.m` sessions, keyed by the request's `session` token.
#[derive(Default)]
pub struct Scripts {
    sessions: BTreeMap<String, Session>,
}

impl Scripts {
    pub fn request(&mut self, request: &Json) -> Json {
        match catch_unwind(AssertUnwindSafe(|| handle(request, &mut self.sessions))) {
            Ok(Ok(reply)) => reply,
            Ok(Err(error)) => json!({"success":false,"error":error}),
            Err(_) => {
                // An interrupted execution may leave the workspace half-updated.
                self.sessions
                    .remove(request["session"].as_str().unwrap_or("default"));
                json!({"success":false,"error":"O motor .m falhou e a sessão foi reiniciada. Execute novamente."})
            }
        }
    }
}

/// Same folder the Python engine uses for the home page: Documentos/ControLAB.
fn default_workspace() -> PathBuf {
    #[cfg(target_os = "android")]
    let folder =
        crate::android::workspace_dir().unwrap_or_else(|| std::env::temp_dir().join("ControLAB"));
    #[cfg(not(target_os = "android"))]
    let folder = {
        let home = dirs::home_dir().unwrap_or_else(std::env::temp_dir);
        let (pt, en) = (
            home.join("Documentos/ControLAB"),
            home.join("Documents/ControLAB"),
        );
        if !pt.is_dir() && en.is_dir() { en } else { pt }
    };
    let _ = std::fs::create_dir_all(&folder);
    folder
}

fn display_value(value: &Value, rows: usize, cols: usize) -> String {
    match value {
        Value::Text(text) => text.chars().take(4096).collect(),
        Value::TransferFunction(model) => model.to_string(),
        Value::Numeric(matrix) => {
            if matrix.data().is_empty() {
                return format!("[] {:?}", matrix.shape());
            }
            let mut lines = Vec::new();
            for row in 0..matrix.rows().min(rows) {
                let mut line = Vec::new();
                for col in 0..matrix.cols().min(cols) {
                    line.push(format!("{}", matrix.element(row + col * matrix.rows())));
                }
                if matrix.cols() > cols {
                    line.push("…".into());
                }
                lines.push(line.join("  "));
            }
            if matrix.rows() > rows || matrix.cols() > cols || matrix.shape().len() > 2 {
                lines.push("… (exibição parcial)".into());
            }
            lines.join("\n")
        }
    }
}
fn workspace(session: &Session) -> Vec<Json> {
    session.workspace().iter().map(|(name, value)| {
        let (kind, dimensions) = match value {
            Value::Numeric(m) => (m.class(), m.shape().iter().map(usize::to_string).collect::<Vec<_>>().join(" × ")),
            Value::Text(t) => ("texto", format!("{} caracteres", t.chars().count())),
            Value::TransferFunction(_) => ("tf", "1 × 1 SISO".into()),
        };
        json!({"name":name,"type":kind,"dimensions":dimensions,"value":display_value(value,2,4).replace('\n'," ; ")})
    }).collect()
}
fn path_arg(request: &Json) -> Result<PathBuf, String> {
    let path = request["path"].as_str().ok_or("Caminho ausente.")?;
    if path.is_empty() {
        return Err("Caminho vazio.".into());
    }
    Ok(PathBuf::from(path))
}
fn handle(request: &Json, sessions: &mut BTreeMap<String, Session>) -> Result<Json, String> {
    let operation = request["operation"].as_str().unwrap_or("state");
    let token = request["session"].as_str().unwrap_or("default");
    if token.len() > 128 {
        return Err("Identificador de sessão inválido.".into());
    }
    if operation == "close" {
        sessions.remove(token);
        return Ok(json!({"success":true}));
    }
    if sessions.len() >= 32 && !sessions.contains_key(token) {
        return Err("Limite de sessões atingido. Feche uma sessão antes de continuar.".into());
    }
    let session = sessions.entry(token.to_owned()).or_default();
    match operation {
        "state" => Ok(json!({"success":true,"workspace":workspace(session)})),
        "list" => {
            let path = match request["path"].as_str() {
                None | Some("") => default_workspace(),
                Some(_) => path_arg(request)?,
            }
            .canonicalize()
            .map_err(|e| e.to_string())?;
            let files=documents::list_folder(&path).map_err(|e|e.to_string())?.iter().take(2000).map(|entry| json!({
                "name":entry.path.file_name().unwrap_or_default().to_string_lossy(),
                "path":entry.path,"directory":entry.directory,"type":if entry.directory{"pasta"}else{"script"}
            })).collect::<Vec<_>>();
            Ok(json!({"success":true,"folder":path,"parent":path.parent(),"files":files}))
        }
        "read" => {
            let doc =
                documents::Document::open(1, &path_arg(request)?).map_err(|e| e.to_string())?;
            Ok(json!({"success":true,"source":doc.source,"path":doc.path,"name":doc.title()}))
        }
        "save" => {
            let source = request["source"].as_str().ok_or("Código ausente.")?;
            let mut doc = documents::Document::untitled(1, source.to_owned());
            let path = path_arg(request)?;
            // Existing files require an explicit overwrite action in the client.
            if path.exists() && !request["overwrite"].as_bool().unwrap_or(false) {
                return Ok(
                    json!({"success":false,"exists":true,"error":"O arquivo já existe. Confirme a substituição."}),
                );
            }
            doc.save(&path).map_err(|e| e.to_string())?;
            Ok(json!({"success":true,"path":doc.path,"name":doc.title(),"dirty":doc.dirty()}))
        }
        "clear" | "execute" => {
            let source = request["source"].as_str().unwrap_or("");
            if source.len() > documents::MAX_SCRIPT_BYTES {
                return Err("Script excede 1 MiB.".into());
            }
            let mut options = ExecutionOptions::default();
            options.resources.wall_time = Some(Duration::from_secs(8));
            options.resources.allocation_bytes = 64 * 1024 * 1024;
            options.max_workspace_bytes = 32 * 1024 * 1024;
            options.max_events = 2000;
            let result = if operation == "clear" {
                session.clear_with_events(&options, |_| Ok(()))
            } else {
                session.execute_with_events(source, &options, |_| Ok(()))
            };
            let output = result
                .execution
                .outputs
                .iter()
                .map(|out| {
                    let value = display_value(&out.value, 30, 10);
                    match &out.name {
                        Some(name) => format!("{name} =\n{value}"),
                        None => value,
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            let plots=result.execution.plots.iter().map(|event| {
                let figure=event.figure();
                json!({"title":figure.title,"xLabel":figure.x_axis.label,"yLabel":figure.y_axis.label,"series":figure.series().iter().map(|series|json!({"name":series.name(),"points":series.points(),"color":series.style().color})).collect::<Vec<_>>()})
            }).collect::<Vec<_>>();
            let error=result.diagnostic.as_ref().map(|e|json!({"message":e.message,"line":e.span.line,"column":e.span.column,"identifier":e.identifier}));
            Ok(
                json!({"success":result.diagnostic.is_none(),"output":output,"diagnostic":error,"workspace":workspace(session),"plots":plots}),
            )
        }
        _ => Err("Operação de script desconhecida.".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn request(scripts: &mut Scripts, operation: &str, data: Json) -> Json {
        let mut request = json!({"action":"script_request","session":"test","operation":operation});
        request
            .as_object_mut()
            .unwrap()
            .extend(data.as_object().unwrap().clone());
        scripts.request(&request)
    }

    fn names(reply: &Json) -> BTreeSet<String> {
        reply["workspace"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["name"].as_str().unwrap().to_owned())
            .collect()
    }

    #[test]
    fn matrix_script_and_console_share_the_workspace() {
        let mut scripts = Scripts::default();
        let source = "A=[1 2;3 4]; B=[5 6;7 8]; C=A*B; disp(C);";
        let reply = request(&mut scripts, "execute", json!({"source":source}));
        assert_eq!(reply["success"], true, "{reply}");
        let output = reply["output"].as_str().unwrap();
        assert!(
            output.contains("19  22") && output.contains("43  50"),
            "{output}"
        );
        assert_eq!(names(&reply), ["A", "B", "C"].map(String::from).into());
        let reply = request(&mut scripts, "execute", json!({"source":"det(A)"}));
        assert!(reply["output"].as_str().unwrap().contains("-2"), "{reply}");
        assert_eq!(
            request(&mut scripts, "clear", json!({}))["workspace"],
            json!([])
        );
    }

    #[test]
    fn sessions_are_isolated_and_failures_keep_completed_assignments() {
        let mut scripts = Scripts::default();
        let source = "before=42; missing_function(1)";
        let reply = request(&mut scripts, "execute", json!({"source":source}));
        assert_eq!(reply["success"], false);
        assert!(reply["diagnostic"]["line"].as_u64().unwrap() >= 1);
        assert!(names(&reply).contains("before"));
        let other = scripts.request(&json!({"session":"other","operation":"state"}));
        assert_eq!(other["workspace"], json!([]));
        let reply = request(&mut scripts, "execute", json!({"source":"before"}));
        assert!(reply["output"].as_str().unwrap().contains("42"));
    }

    #[test]
    fn plots_and_control_models_return_structured_points() {
        let mut scripts = Scripts::default();
        let source = "x=0:0.1:1; plot(x,x.^2);";
        let reply = request(&mut scripts, "execute", json!({"source":source}));
        assert_eq!(reply["success"], true, "{reply}");
        let points = reply["plots"][0]["series"][0]["points"].as_array().unwrap();
        assert_eq!(points.last().unwrap(), &json!([1.0, 1.0]));
        let source = "G=tf([1],[1 2 1]); step(G);";
        let reply = request(&mut scripts, "execute", json!({"source":source}));
        assert_eq!(reply["success"], true, "{reply}");
        assert!(
            reply["plots"][0]["series"][0]["points"]
                .as_array()
                .unwrap()
                .len()
                > 2
        );
    }

    #[test]
    fn file_roundtrip_and_failed_saves_preserve_existing_data() {
        let temp = std::env::temp_dir().join(format!("controlab-scripts-{}", std::process::id()));
        std::fs::create_dir_all(&temp).unwrap();
        let file = temp.join("matrizes.m");
        let mut scripts = Scripts::default();
        let save =
            |scripts: &mut Scripts, path: &std::path::Path, source: &str, overwrite: bool| {
                request(
                    scripts,
                    "save",
                    json!({"path":path,"source":source,"overwrite":overwrite}),
                )
            };
        assert_eq!(
            save(&mut scripts, &file, "A=[1 2;3 4];", false)["success"],
            true
        );
        let read = request(&mut scripts, "read", json!({"path":file}));
        assert_eq!(read["source"], "A=[1 2;3 4];");
        let collision = save(&mut scripts, &file, "new", false);
        assert_eq!(
            (collision["success"].clone(), collision["exists"].clone()),
            (json!(false), json!(true))
        );
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "A=[1 2;3 4];");
        assert_eq!(save(&mut scripts, &file, "changed", true)["success"], true);
        let listing = request(&mut scripts, "list", json!({"path":temp}));
        assert_eq!(listing["files"][0]["name"], "matrizes.m");
        let failed = save(&mut scripts, &temp.join("missing/a.m"), "new", false);
        assert_eq!(failed["success"], false);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "changed");
        let other = request(&mut scripts, "read", json!({"path":temp.join("other.txt")}));
        assert_eq!(other["success"], false);
        std::fs::remove_dir_all(&temp).unwrap();
    }

    #[test]
    fn resource_limits_do_not_break_the_next_request() {
        let mut scripts = Scripts::default();
        let huge = request(
            &mut scripts,
            "execute",
            json!({"source":"huge=zeros(10000000,1);"}),
        );
        assert_eq!(huge["success"], false);
        let next = request(&mut scripts, "execute", json!({"source":"2+2"}));
        assert_eq!(next["success"], true, "{next}");
        let oversized = "x".repeat(documents::MAX_SCRIPT_BYTES + 1);
        assert_eq!(
            request(&mut scripts, "execute", json!({"source":oversized}))["success"],
            false
        );
    }

    #[test]
    fn listing_without_a_folder_uses_the_default_workspace() {
        let mut scripts = Scripts::default();
        let reply = request(&mut scripts, "list", json!({}));
        assert_eq!(reply["success"], true, "{reply}");
        assert_eq!(
            reply["folder"],
            json!(default_workspace().canonicalize().unwrap())
        );
    }
}
