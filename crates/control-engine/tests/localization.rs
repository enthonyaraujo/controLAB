use controlab_engine::localization::translate_result;
use serde_json::json;
#[test]fn english_translation_protects_user_strings_and_numeric_results(){
    let input=json!({"title":"Resposta no Domínio do Tempo","details":{"stable":true},"source":"Resposta no Domínio do Tempo","path":"/tmp/Resposta no Domínio do Tempo","metrics":[{"label":"Estabilidade","value":"Estável"}]});
    let out=translate_result(input.clone(),"en").unwrap();
    assert_eq!(out["title"],"Time-Domain Response");assert_eq!(out["source"],input["source"]);assert_eq!(out["path"],input["path"]);assert_eq!(out["metrics"][0]["value"],"Estável");
    assert_eq!(translate_result(input.clone(),"pt-BR").unwrap(),input);
}
