//! JSON adapter for the copied openMAT core. No dependency on the original checkout.
#[path = "../../../documents.rs"]
mod documents;
use openmat_lang::{ExecutionOptions, Session, Value};
use serde_json::{json, Value as Json};
use std::{collections::BTreeMap, io::{self, BufRead, Read, Write}, path::PathBuf, time::Duration};

// The bounded previews follow openmat-studio/src/worker.rs.
fn display_value(value: &Value, rows: usize, cols: usize) -> String {
    match value {
        Value::Text(text) => text.chars().take(4096).collect(),
        Value::TransferFunction(model) => model.to_string(),
        Value::Numeric(matrix) => {
            if matrix.data().is_empty() { return format!("[] {:?}", matrix.shape()); }
            let mut lines = Vec::new();
            for row in 0..matrix.rows().min(rows) {
                let mut line = Vec::new();
                for col in 0..matrix.cols().min(cols) {
                    line.push(format!("{}", matrix.element(row + col * matrix.rows())));
                }
                if matrix.cols() > cols { line.push("…".into()); }
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
    let path=request["path"].as_str().ok_or("Caminho ausente.")?;
    if path.is_empty() { return Err("Caminho vazio.".into()); }
    Ok(PathBuf::from(path))
}
fn handle(request: &Json, sessions: &mut BTreeMap<String, Session>) -> Result<Json, String> {
    let operation=request["operation"].as_str().unwrap_or("state");
    let token=request["session"].as_str().unwrap_or("default");
    if token.len()>128 {return Err("Identificador de sessão inválido.".into());}
    if operation=="close" { sessions.remove(token); return Ok(json!({"success":true})); }
    if sessions.len()>=32 && !sessions.contains_key(token) { return Err("Limite de sessões atingido. Feche uma sessão antes de continuar.".into()); }
    let session=sessions.entry(token.to_owned()).or_default();
    match operation {
        "state" => Ok(json!({"success":true,"workspace":workspace(session)})),
        "list" => {
            let path=path_arg(request)?.canonicalize().map_err(|e|e.to_string())?;
            let files=documents::list_folder(&path).map_err(|e|e.to_string())?.iter().take(2000).map(|entry| json!({
                "name":entry.path.file_name().unwrap_or_default().to_string_lossy(),
                "path":entry.path,"directory":entry.directory,"type":if entry.directory{"pasta"}else{"script"}
            })).collect::<Vec<_>>();
            Ok(json!({"success":true,"folder":path,"parent":path.parent(),"files":files}))
        },
        "read" => {
            let doc=documents::Document::open(1,&path_arg(request)?).map_err(|e|e.to_string())?;
            Ok(json!({"success":true,"source":doc.source,"path":doc.path,"name":doc.title()}))
        },
        "save" => {
            let source=request["source"].as_str().ok_or("Código ausente.")?;
            let mut doc=documents::Document::untitled(1,source.to_owned());
            let path=path_arg(request)?;
            // Existing files require an explicit overwrite action in the client.
            if path.exists() && !request["overwrite"].as_bool().unwrap_or(false) {
                return Ok(json!({"success":false,"exists":true,"error":"O arquivo já existe. Confirme a substituição."}));
            }
            doc.save(&path).map_err(|e|e.to_string())?;
            Ok(json!({"success":true,"path":doc.path,"name":doc.title(),"dirty":doc.dirty()}))
        },
        "clear" | "execute" => {
            let source=request["source"].as_str().unwrap_or("");
            if source.len()>documents::MAX_SCRIPT_BYTES {return Err("Script excede 1 MiB.".into());}
            let mut options=ExecutionOptions::default();
            options.resources.wall_time=Some(Duration::from_secs(8));
            options.resources.allocation_bytes=64*1024*1024;
            options.max_workspace_bytes=32*1024*1024;
            options.max_events=2000;
            let result=if operation=="clear" {session.clear_with_events(&options, |_|Ok(()))} else {session.execute_with_events(source,&options, |_|Ok(()))};
            let output=result.execution.outputs.iter().map(|out| {
                let value=display_value(&out.value,30,10);
                match &out.name {Some(name)=>format!("{name} =\n{value}"),None=>value}
            }).collect::<Vec<_>>().join("\n");
            let plots=result.execution.plots.iter().map(|event| {
                let figure=event.figure();
                json!({"title":figure.title,"xLabel":figure.x_axis.label,"yLabel":figure.y_axis.label,"series":figure.series().iter().map(|series|json!({"name":series.name(),"points":series.points(),"color":series.style().color})).collect::<Vec<_>>()})
            }).collect::<Vec<_>>();
            let error=result.diagnostic.as_ref().map(|e|json!({"message":e.message,"line":e.span.line,"column":e.span.column,"identifier":e.identifier}));
            Ok(json!({"success":result.diagnostic.is_none(),"output":output,"diagnostic":error,"workspace":workspace(session),"plots":plots}))
        },
        _ => Err("Operação de script desconhecida.".into()),
    }
}
fn main() {
    let mut sessions=BTreeMap::new();
    let stdin=io::stdin();
    let mut input=stdin.lock();
    let mut stdout=io::stdout().lock();
    loop {
        // Bound the transport independently of the language source limit.
        let mut line=Vec::new();
        match input.by_ref().take(8*1024*1024+1).read_until(b'\n',&mut line) {Ok(0)|Err(_)=>break,_=>{}}
        if line.len()>8*1024*1024 {break;}
        let response=serde_json::from_slice(&line).map_err(|e|e.to_string()).and_then(|request|handle(&request,&mut sessions));
        let response=response.unwrap_or_else(|error|json!({"success":false,"error":error}));
        if writeln!(stdout,"{response}").and_then(|()|stdout.flush()).is_err(){break;}
    }
}
