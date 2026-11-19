# Referências Científicas e Dados Reproduzíveis

Este diretório contém o subconjunto mínimo e reproduzível das referências numéricas independentes e dados experimentais de benchmark utilizados na validação científica do motor em Rust do **ControLAB**.

Estes arquivos têm finalidade exclusivamente científica e documental e **não constituem dependências de execução ou compilação** do ControLAB.

---

## Conteúdo

| Arquivo | Descrição |
|---|---|
| [`generate_matlab_reference.m`](generate_matlab_reference.m) | Script independente de geração das referências MATLAB R2022b para os 19 casos de LGR e as 10 configurações clássicas de projeto de controladores (Q1–Q9) |
| [`matlab_r2022b_reference.json`](matlab_r2022b_reference.json) | Saída JSON gerada pelo MATLAB R2022b (`9.13.0.2049777`) com polos, zeros, centroide, assíntotas, pontos de quebra, ângulos de partida/chegada, amostras de raízes em 13 ganhos e métricas `stepinfo` |
| [`generate_matlab_etapa10_reference.m`](generate_matlab_etapa10_reference.m) | Script independente de verificação da referência analítica da Questão 8 ($G_c(s) = -1 - 0{,}2/s$, perturbações de $\pm 10\%$ e rejeição a perturbação de carga) e dos controladores refinados por métricas temporais |
| [`matlab_r2022b_etapa10_reference.json`](matlab_r2022b_etapa10_reference.json) | Saída JSON do MATLAB R2022b correspondente a `generate_matlab_etapa10_reference.m` |
| [`generate_matlab_etapa11_reference.m`](generate_matlab_etapa11_reference.m) | Script independente de auditoria dos coeficientes de amortecimento $\xi$, resíduos modais de Heaviside e robustez paramétrica dos controladores sob restrições estritas (`DampingSpec::Equal`) e de limite mínimo (`DampingSpec::AtLeast`) |
| [`matlab_r2022b_etapa11_reference.json`](matlab_r2022b_etapa11_reference.json) | Saída JSON do MATLAB R2022b correspondente a `generate_matlab_etapa11_reference.m` |
| [`benchmark-results.json`](benchmark-results.json) | Dados consolidados dos 6 cenários de benchmark comparando o binário nativo Rust (`--release`) com o motor Python legado (`be79347`), incluindo metadados do ambiente experimental |

---

## Instruções para Reprodução Independente no MATLAB

Caso disponha do **MATLAB** (R2022b ou compatível) com **Control System Toolbox**, execute a partir deste diretório (`docs/validation/references/`):

```bash
matlab -batch "run('generate_matlab_reference.m'); run('generate_matlab_etapa10_reference.m'); run('generate_matlab_etapa11_reference.m');"
```

Cada script regenera o respectivo arquivo `matlab_r2022b_*.json` no diretório de trabalho atual utilizando exclusivamente funções nativas do MATLAB (`tf`, `pole`, `zero`, `rlocus`, `feedback`, `minreal`, `step`, `stepinfo`, `dcgain`, `residue`, `jsonencode`).
