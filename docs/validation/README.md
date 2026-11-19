# Validação Científica e Numérica do ControLAB

Este diretório reúne a documentação pública de validação científica, numérica, funcional, visual e de desempenho do motor computacional nativo em Rust do **ControLAB**.

A validação cobre o escopo específico dos módulos de controle linear contínuo SISO, diagramas de blocos e interpretador `.m` avaliados frente a:
- Derivações analíticas fechadas.
- Execuções independentes em **MATLAB R2022b** (`9.13.0.2049777`, Control System Toolbox) para os casos testados.
- Referências congeladas do motor Python legado (`be79347`, 152 casos de paridade).
- Problemas acadêmicos de Sistemas de Controle II (UFERSA — 19 casos de Lugar Geométrico das Raízes e 9 questões de Projeto de Controladores).

> **Escopo das afirmações:** Os resultados documentados aplicam-se exclusivamente às funcionalidades, modelos e casos efetivamente validados nesta suíte. Não se declara equivalência universal com todo o ecossistema MATLAB/Simulink.

---

## Navegação

| Documento | Conteúdo |
|---|---|
| [`methodology.md`](methodology.md) | Classificação nas 4 categorias independentes, semântica explícita de especificações ($\xi$, $\omega_n$, $t_s$, $M_p$, $e_{ss}$), estabilidade interna e tolerâncias numéricas |
| [`academic-validation.md`](academic-validation.md) | Resultados consolidados dos 19 casos de LGR e das 9 questões de Projeto de Controladores, incluindo a referência independente da Q8 e análises de robustez |
| [`synthesis-comparison.md`](synthesis-comparison.md) | Comparação entre síntese clássica via LGR (`lgr_classic`) e síntese por restrições (`temporal_refined` / `auto_simplest`), provas de inviabilidade matemática e evidências visuais |
| [`benchmarks.md`](benchmarks.md) | Metodologia, ambiente e resultados de desempenho Rust (`--release`) vs motor Python legado (`be79347`) |
| [`final-acceptance.md`](final-acceptance.md) | Síntese de aceitação, cobertura real dos testes, limitações matemáticas conhecidas e análise de riscos |
| [`validation-matrix.csv`](validation-matrix.csv) | Matriz completa das 42 configurações auditadas com status separado nas 4 categorias |

---

## Como Reproduzir a Validação

Toda a suíte permanente de validação em Rust é autossuficiente e **não requer Python nem MATLAB** em tempo de execução:

```bash
# 1. Formatação e análise estática
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# 2. Suíte completa do workspace (inclui validação acadêmica, BlockLab, mscript e GUI)
cargo test --workspace

# 3. Suítes acadêmicas e de conformidade específicas
cargo test -p controlab-engine --test academic_lgr_validation
cargo test -p controlab-engine --test academic_controllers_validation
cargo test -p controlab-engine --test academic_requirements_compliance
cargo test -p blocklab-core --test academic_diagrams
cargo test -p mscript --test academic_validation

# 4. Compilação release, paridade congelada (152 casos) e auditoria de fontes
cargo build --workspace --release
cargo run --release -p xtask -- parity
cargo run --release -p xtask -- check-sources

# 5. (Opcional — requer MATLAB local com Control System Toolbox)
cargo test --release -p mscript -- --ignored
```
