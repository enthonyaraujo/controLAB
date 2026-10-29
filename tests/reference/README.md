# Referência Python para migração científica

`python/manifest.json` identifica o commit de origem, SHA-256 de cada módulo Python,
versão do interpretador e versões de NumPy, SciPy, SymPy, python-control, Matplotlib
e Pillow. Cada caso registra a entrada e a saída JSON completa normalizada com
`native_bridge._json_safe`; exceções têm o mesmo envelope do transporte nativo.
Os casos públicos passam por `native_presentation.dispatch_native`.

As imagens e SVGs são preservados integralmente em `python/assets/`, deduplicados
por SHA-256. Um objeto `$asset` substitui somente a string binária/textual original,
registrando tamanho, hash e codificação suficientes para reconstruí-la. Nenhum
campo científico é removido. SVGs usam `svg.hashsalt` fixo e `SOURCE_DATE_EPOCH=0`
para IDs e metadados estáveis; nenhum ID é descartado da comparação.

A cobertura inclui todos os presets publicados dos seis módulos (LGR,
Routh-Hurwitz, tempo, frequência, controladores e espaço de estados), catálogos,
cinco modos de entrada de funções de transferência, entradas inválidas, casos
especiais de Routh, avaliação pontual de K, conversão canônica, ganhos K/L,
exercícios de controladores extraídos literalmente dos testes existentes e
sintonias clássicas P/PI/PD/PID. Casos adicionais preservam parsers de polos,
matrizes e coeficientes, renderização TeX e comandos fixos de ajuda do console.
Não executa scripts do usuário nem consulta atualizações na rede.

Os casos temporais incluem **todos** os pontos das curvas de degrau, impulso e
rampa na grade efetivamente usada pelo motor. Os casos de frequência incluem a
grade completa, módulo, fase desenrolada, dB, assíntotas, resposta complexa e módulo
de malha fechada. Essas matrizes adicionais não existem na resposta pública e
foram obtidas pelas mesmas funções de cálculo usadas pelo motor.

O gerador `scripts/export_rust_reference.py`, o motor Python e o teste de
integridade `tests/migration` foram retirados com a engine Python; estão no
histórico Git (último commit com Python: `be79347`). O corpus agora é imutável:
os testes `reference_*` do `xtask` conferem os SHA-256 de cada caso e imagem e a
cobertura dos presets, e a comparação com o Rust é feita por:

```sh
cargo xtask parity
cargo xtask parity --module time --case time_preset_00
```

Regenerar exige restaurar esses arquivos de `be79347` e o ambiente científico
registrado em `python/manifest.json`.

Esta é a base de referência da fase 1. Por si só **não comprova equivalência Rust**
(papel de `cargo xtask parity`), nem cobertura
exaustiva de todas as combinações numéricas, UI, execução do interpretador `.m`,
empacotamento, rede ou execução em Windows/Android. Não inclui operações do
console que dependem do diretório pessoal (`pwd`, `cd`, `ls`, `dir`, home state),
nem execução arbitrária de expressões do console. Metadados, formatação e gráficos
podem variar com versões de biblioteca/fonte; equivalência numérica futura deve
usar tolerâncias justificadas, preservando verificações exatas de texto e formas.

Captura inicial: **158 casos** (133 respostas de sucesso e 25 respostas de erro),
**28 presets**, **215 arquivos PNG/SVG**, aproximadamente **21 MiB**. Integridade:
2 testes aprovados; suíte Python existente e integração: 108 testes aprovados
(última execução antes da retirada: 108 aprovados em 2026-10-29).
A reprodução independente de oito casos (um por módulo, TeX e entrada Unicode)
foi verificada byte a byte. A reprodução integral dos 158 casos ainda não foi
executada como segunda passagem; a geração inicial executou todos eles.
