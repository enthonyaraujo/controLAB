# ControLAB — Sistemas de Controle

Interface nativa em **Rust + egui/eframe**, engine científica em **Rust** e
interpretador `.m` em **Rust** (`mscript`).

## Executar no Linux

Requer Rust/Cargo e as bibliotecas gráficas de desenvolvimento usadas pelo
eframe. Compile e execute com a engine científica Rust:

```bash
cargo build --locked --workspace
CONTROLAB_SCIENTIFIC_BACKEND=rust cargo run --locked --bin controlab
```

O workspace gera `controlab`, que inclui o interpretador `.m`, em `target/debug/`;
`controlab-engine` oferece transporte científico local por JSON-lines. Para uma
compilação otimizada, acrescente `--release` ao build e à execução. `--offline`
pode ser acrescentado quando as dependências Cargo já estiverem no cache.
Nenhuma dependência é instalada automaticamente pelos comandos de build.

## Funcionalidades

- Resposta no tempo, estabilidade/Routh-Hurwitz, lugar das raízes, Bode/Nyquist,
  controladores e espaço de estados.
- Studio `.m` com arquivos, editor, console e variáveis persistentes na sessão.
- Gráficos científicos, fórmulas, memoriais e exportação PNG/SVG.
- Temas claro/escuro/sistema e layouts de celular, tablet e desktop.
- Aplicação para fazer Redução de Blocos (diagrama de blocos)


