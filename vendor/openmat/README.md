# Núcleo openMAT usado pelo ControLAB

Snapshot copiado em 2026-10-20 da árvore de trabalho de `/home/enthony/GitHub/openmat`, cuja revisão-base é `470cd4c3b02367e6b8c7c25b5a648f414d1b25db`. A origem já continha alterações locais, incorporadas nesta cópia. O projeto original foi consultado em modo somente leitura. `SOURCE_MANIFEST.json` registra os hashes das fontes copiadas.

- `crates/openmat-{numeric,control,graphics,lang}`: fontes, manifests e testes copiados sem modificações.
- `documents.rs`: leitura, listagem e salvamento de `.m` do openMAT Studio, sem modificações.
- `crates/controlab-runtime`: adaptador próprio do ControLAB para JSON por linha e sessões persistentes. As prévias limitadas de variáveis seguem a implementação de `openmat-studio/src/worker.rs`.
- `Cargo.toml`/`Cargo.lock`: workspace independente. O adaptador usa `serde_json` para o transporte; não utiliza o frontend egui do Studio.
- `LICENSE`: licença MIT do openMAT. Os créditos devem acompanhar a distribuição.

O ControLAB compila e executa esta cópia sem consultar a pasta original. Alterações futuras do openMAT não são sincronizadas automaticamente.

O suporte a MATLAB é incremental: arrays, aritmética, funções numéricas e o subconjunto de controle e gráficos implementado no openMAT. Não há compatibilidade integral com MATLAB, funções de usuário ou controle de fluxo.

Validação dos crates e do adaptador:

```bash
cargo test --locked --manifest-path vendor/openmat/Cargo.toml --target-dir build/openmat
```

Os três testes diferenciais marcados como ignorados exigem MATLAB e não integram a validação padrão.
