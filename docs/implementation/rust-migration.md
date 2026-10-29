# Migração Python → Rust

Plano aprovado em 2026-10-28. Branch: `enthony/migrate-python-to-rust`.

## Contrato

Preservar os seis módulos, métodos, entradas, padrões, resultados, deduções,
traduções, gráficos PNG/SVG e fórmulas. Diferenças mínimas de ponto flutuante
são aceitas (`atol=1e-9`, `rtol=1e-7`), sem alterar decisões ou valores exibidos.
Linux, Windows e Android; ferramentas próprias também em Rust. Python só será
removido depois da equivalência comprovada, ficando no histórico Git.

## Etapas e evidências

- [ ] 1. Catálogo e dados de referência reproduzíveis do Python.
- [ ] 2. Entradas e fundamentos matemáticos Rust.
- [ ] 3. Tempo, frequência, LGR, Routh, controladores e espaço de estados.
- [ ] 4. Gráficos, composição TeX, localização e serviços auxiliares.
- [ ] 5. Integração desktop/Android, mantendo contratos e isolamento.
- [ ] 6. `cargo xtask`, instaladores e CI sem ferramentas Python próprias.
- [ ] 7. Comparações finais, retirada do Python, documentação e memória.

Base: 106 testes Python aprovados; suíte Rust aprovada (três testes MATLAB
ignorados por requererem instalação local). Nenhuma equivalência nova presumida.

## Registro de execução

- Branch criada a partir de `main` limpa. Sem autorização de commit/push/publicação.
- A implementação permanece no checkout solicitado, isolada pela branch; nenhum
  worktree adicional foi criado.
- Referências e fundamentos são implementados em frentes de arquivos distintos.
- A transição não selecionará automaticamente um módulo parcial em produção.
