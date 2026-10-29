use controlab_engine::updates::decode_release;
use serde_json::json;
#[test]
fn release_response_and_errors_preserve_fields() {
    assert_eq!(decode_release(200,br#"{"tag_name":"v6.0.1","html_url":"https://github.com/enthonyaraujo/controlab/releases/tag/v6.0.1","published_at":"2026-10-29T00:00:00Z"}"#),json!({"success":true,"tag_name":"v6.0.1","html_url":"https://github.com/enthonyaraujo/controlab/releases/tag/v6.0.1","published_at":"2026-10-29T00:00:00Z"}));
    assert_eq!(
        decode_release(403, b"")["error"],
        "Limite de requisições da API do GitHub atingido temporariamente. Tente novamente mais tarde."
    );
    assert_eq!(
        decode_release(404, b"")["error"],
        "O GitHub retornou HTTP 404"
    );
    assert_eq!(decode_release(200, b"invalid")["success"], false);
}
