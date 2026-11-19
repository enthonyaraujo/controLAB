# Changelog — ControLAB

Todas as mudanças relevantes e lançamentos da suíte ControLAB são documentados neste arquivo.
O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/) e este projeto segue [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [1.1.0] - 2026-11-19

Integração da validação científica, numérica, funcional, visual e de desempenho do motor nativo em Rust e da síntese de controladores orientada por restrições ([PR #1](https://github.com/enthonyaraujo/controLAB/pull/1)).

### Added

- Suíte permanente de validação acadêmica e conformidade científica em Rust (`academic_lgr_validation`, `academic_controllers_validation`, `academic_requirements_compliance`, `blocklab-core/tests/academic_diagrams` e `mscript/tests/academic_validation`), sem dependência de Python ou MATLAB em tempo de execução.
- Motor genérico de verificação e síntese por restrições em `crates/control-engine/src/controller_synthesis.rs` com modos `temporal_refined` e `auto_simplest` (seleção automática `P -> PI -> PD -> Lead -> PID`) e parametrização canônica de ganhos PID.
- Semântica explícita para restrições de coeficiente de amortecimento (`DampingSpec::{Equal, AtLeast, DidacticEstimate}`), frequência natural (`NaturalFreqSpec::{Equal, AtLeast, OptionalEstimate}`), tempo de acomodação (`AtMost` / `Equal` com tolerância padrão uniforme de `±2%`, piso `0,02 s`) e dominância modal (identificação do polo/par de referência e razão de resíduos de Heaviside).
- Classificação explícita de viabilidade (`Feasible`, `ProvenInfeasible`, `NotFoundWithinSearchBounds`) com provas algébricas de inviabilidade e diagnóstico de estruturas alternativas.
- Painel de conformidade na interface nativa (`controlab-native`) com banner de status global (`CONFORME`, `CONFORME COM ALERTAS`, `NÃO CONFORME`), comparação `Solicitado × Teórico × Medido × Diferença`, marcações `[OBRIGATÓRIA]` vs `[ESTIMATIVA OPCIONAL]`, referência modal, motivos de não conformidade e advertências técnicas.
- Documentação científica pública de validação, matriz de 42 configurações auditadas, referências independentes reproduzíveis do MATLAB R2022b e resultados consolidados de benchmarks em `docs/validation/` e `docs/validation/references/`.

### Changed

- Aprimorado o módulo `controllers` para distinguir projeto clássico por alocação de polos via LGR (`lgr_classic`) da síntese por restrições temporais/modais, preservando integralmente o contrato legado de paridade (152/152 casos).
- Aprimorada a verificação de estabilidade para avaliar o polinômio característico não reduzido antes de qualquer simplificação, detectando instabilidade interna por cancelamento polo-zero no SPD ou eixo imaginário.
- Integrada a verificação automática de paridade congelada (`cargo run --locked --release -p xtask -- parity`) ao workflow de CI em `.github/workflows/build-all.yml`.

### Fixed

- Corrigidas regressões de formatação (`rustfmt`) e de paridade congelada em `crates/control-engine/src/controllers.rs` nos casos `controller_test_q5_pid_by_root_locus_0` e `controller_test_q5_pid_custom_pi_zero_0`, restaurando 152/152 casos aprovados.
- Corrigida a classificação indevida de controladores numericamente exatos como bem-sucedidos na GUI quando a resposta temporal real ou o amortecimento violam especificações obrigatórias.
- Corrigida a conversão implícita de igualdades de amortecimento ($\xi$) em desigualdades de sobressinal ($M_p$) durante o refinamento numérico.

## [1.0.0] - 2026-10-30

- Versão inicial nativa Rust unificada do ControLAB (Linux, Windows e Android).
- Suíte completa de Engenharia de Controle: Lugar das Raízes (LGR), Critério de Routh-Hurwitz, Resposta no Tempo, Resposta em Frequência (Bode/Nyquist), Projeto de Controladores, Espaço de Estados e Diagrama de Blocos.
- Interpretador de scripts `.m` e Command Window integrados.
