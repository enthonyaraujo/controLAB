# Migração Python → Rust

Plano aprovado em 2026-10-28. Branch: `enthony/migrate-python-to-rust`.

## Contrato

Preservar os seis módulos, métodos, entradas, padrões, resultados, deduções,
traduções, gráficos PNG/SVG e fórmulas. Diferenças mínimas de ponto flutuante
são aceitas (`atol=1e-9`, `rtol=1e-7`), sem alterar decisões ou valores exibidos.
Linux, Windows e Android; ferramentas próprias também em Rust. Python só será
removido depois da equivalência comprovada, ficando no histórico Git.

## Etapas e evidências

- [x] 1. Catálogo e dados de referência reproduzíveis do Python.
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

### Checkpoint de 2026-10-29

- Corpus congelado: 158 casos, 28 presets, 215 PNG/SVG, proveniência/hashes e
  versões científicas; suíte Python ampliada validada com 108 testes.
- Crate `controlab-engine`: portas de tempo, frequência, LGR, Routh, estados e
  controladores; composição TeX e localização nativas. A aplicação ainda usa
  o motor anterior enquanto a comparação ampla está em andamento.
- LGR: todos os detalhes/textos de seis presets e caso de zeros complexos
  comparados. Estados: 14 testes incluindo referências e MIMO. Controladores:
  oito presets com parâmetros e deduções aprovados; revisão dos 71 casos em curso.
- Divergências confirmadas para tratar: normalização de coeficientes em expressões
  com frações, metadados de respostas não finitas, sinais de polos complexos,
  padrões do dispatcher e diferenças de LaTeX além dos casos verificados.
- `cargo xtask parity` distingue diferenças e imagens não verificadas; não é
  um atestado de equivalência completa. Ferramentas de distribuição em `tools.rs`
  estão sendo integradas; legado Python preservado até os critérios finais.
- Existe checkpoint Git `c9cb0ac` recebido na retomada; preservado. Esta execução
  não cria commits, push ou publicação sem autorização correspondente.

### Decisões confirmadas em 2026-10-29

- O usuário aprovou aceitar diferenças no índice/tempo de pico **somente em
  platôs comprovados**. O comparador exige curvas Python congeladas, reprodução
  Rust, grade correspondente e todos os pontos da cauda dentro de `1e-12` do
  valor estacionário (escala mínima 1); cada aceitação é registrada no relatório.
- O usuário autorizou remover a ponte antiga de console Python sem uso. Os seis
  casos históricos dessa API ficam arquivados; não são contados como migrados.
  O console `.m` atual, já Rust, e os seis módulos científicos permanecem.
- Overshoot é expresso em porcentagem: sua tolerância absoluta deriva do fator
  100 da fórmula (`100 × atol` da saída), sem mudar ganhos ou classificações.
