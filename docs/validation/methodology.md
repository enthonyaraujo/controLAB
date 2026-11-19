# Metodologia de Validação Científica e Semântica de Especificações

## 1. Quatro Categorias Independentes de Validação

Concordância numérica com o MATLAB não garante que um controlador projetado satisfaça as especificações do problema. Por isso, cada caso e configuração é avaliado separadamente em quatro categorias:

| Categoria | Critério |
|---|---|
| `NUMERICAL_PASS` | Coeficientes, polos, zeros, trajetórias LGR, ganhos críticos e respostas temporais concordam com a resolução analítica e com o MATLAB R2022b dentro das tolerâncias definidas na Seção 4. |
| `REQUIREMENTS_PASS` | A malha fechada real — sem desprezar zeros adicionais, polos não dominantes ou aproximações de 2ª ordem — é externa e internamente estável e satisfaz **todas** as restrições obrigatórias especificadas. |
| `FUNCTIONAL_PASS` | O fluxo opera ponta a ponta nos módulos Rust (`controlab-engine`, `blocklab-core`, `mscript` e interface `controlab-native`), reportando explicitamente conformidades, alertas e falhas. |
| `VISUAL_PASS` | Os gráficos gerados (LGR e resposta temporal) e o painel de conformidade da GUI refletem fielmente os valores medidos e o status real do projeto. |

**Regra de aprovação global (`OVERALL`):** uma configuração recebe `PASS` global somente quando todas as categorias aplicáveis forem aprovadas. Caso `NUMERICAL_PASS` seja verdadeiro mas `REQUIREMENTS_PASS` falhe, o status global é obrigatoriamente **`FAIL`**.

---

## 2. Semântica Explícita das Especificações

O verificador (`crates/control-engine/src/controller_synthesis.rs`) distingue formalmente cada tipo de restrição, impedindo conversões silenciosas de igualdades em desigualdades ou descarte tácito de requisitos:

| Parâmetro | Modos suportados | Comportamento |
|---|---|---|
| **Coeficiente de amortecimento ($\xi$)** | `Equal { target, tolerance }`, `AtLeast(min)`, `DidacticEstimate(est)` | Igualdade obrigatória (`±0,01` padrão) exige par complexo conjugado quando `target < 1` (rejeita polos puramente reais). Quando informado sem `Mp`, preserva igualdade automaticamente; quando pareado com `Mp`, é sinalizado como estimativa didática opcional, salvo se marcado como obrigatório. |
| **Frequência natural ($\omega_n$)** | `Equal { target, tolerance }`, `AtLeast(min)`, `OptionalEstimate(est)` | Tolerância padrão uniforme de `±2%` (piso `0,01 rad/s`). |
| **Tempo de acomodação ($t_s$)** | `AtMost(max)` e `Equal { target, tolerance }` | Medido pelo critério definitivo de entrada na banda de `±2%` ou `±5%` (último ponto fora da banda com interpolação linear subamostral). Tolerância padrão uniforme de igualdade: `max(2% × alvo, 0,02 s)` (`±0,02 s` para `1 s`; `±0,08 s` para `4 s`). |
| **Sobressinal máximo ($M_p$)** | Limite superior efetivamente medido (`Mp <= max`) | Somente aplicado como restrição obrigatória quando `Mp` é solicitado explicitamente. |
| **Polos dominantes e dominância modal** | `DesignObjective::PolePlacement`, `require_complex_dominance` | Para sistemas de ordem superior, o motor identifica explicitamente a qual polo ou par complexo conjugado $\xi$ e $\omega_n$ se referem, calcula a razão de dominância $|\operatorname{Re}(p_k)|/|\operatorname{Re}(p_{\text{dom}})|$ e a razão exata de resíduos modais de Heaviside $\sum |R_{\text{não-dom}}| / \sum |R_{\text{dom}}|$ para `T(s)/s`. |
| **Erro estacionário ($e_{ss}$)** | `require_zero_step_error` | Distingue explicitamente erro atuante no somador $e_a(\infty) = r(\infty) - H(0)y(\infty)$ do erro de rastreamento da saída $r(\infty) - y(\infty)$ em realimentação não unitária. |
| **Classificação de viabilidade** | `Feasible`, `ProvenInfeasible`, `NotFoundWithinSearchBounds` | Diferencia inviabilidade matematicamente demonstrada de ausência de solução dentro dos limites da busca numérica. |

---

## 3. Estabilidade Externa e Interna Antes de Qualquer Simplificação

Toda verificação calcula as raízes do polinômio característico **não reduzido**:

$$P(s) = D_c(s)D_g(s)D_h(s) + N_c(s)N_g(s)N_h(s) = 0$$

- Qualquer polo com $\operatorname{Re}(p) \ge -10^{-7}$ reprova imediatamente a estabilidade externa.
- Qualquer cancelamento polo-zero no semiplano direito (SPD) ou sobre o eixo imaginário $j\omega$ (em malha aberta ou fechada) reprova imediatamente a **estabilidade interna**, sem nunca depender de simplificação prévia (`minreal`).
- Cancelamentos estáveis exatos ou aproximados no SPE são permitidos com advertência técnica explícita sobre sensibilidade a variações paramétricas da planta.

---

## 4. Tolerâncias Numéricas e Origem das Referências

| Grandeza | Tolerância | Erro máximo observado | Fundamento técnico |
|---|---:|---:|---|
| Coeficientes polinomiais | `1e-10` | `4,44e-16` | Aritmética racional exata em `exact_transfer.rs` e `blocklab-core` antes da conversão `f64` |
| Polos e zeros simples | `1e-6` | `7,11e-15` | Decomposição de Schur com balanceamento da matriz companheira |
| Polos de multiplicidade $m > 1$ | `1e-4` | `3,14e-08` | Condicionamento teórico $\epsilon_{\text{mach}}^{1/m}$ ($\approx 1{,}5\times 10^{-8}$ para $m=2$) |
| Ganhos de controladores ($K_p, K_i, K_d, z, \tau_d$) | `1e-6` | `8,88e-16` | Solução analítica direta em `f64` |
| Valor estacionário $y(\infty)$ e $e_a(\infty)$ | `1e-5` | `0,00e+00` | Ganho DC exato com cancelamento de potências de $s$ na origem |
| Forma de onda temporal $y(t)$ | `1e-4` | `4,22e-15` | Exponencial matricial exata via scaling-and-squaring |
| Sobressinal percentual ($M_p$) | `0,1 pp` | `0,017 pp` | Simulação densa (2.000–4.000 pontos) comparada ao `stepinfo` do MATLAB R2022b |
| Tempo de acomodação definitivo ($t_s$) | `0,02 s` | `0,0146 s` | Última saída da faixa com interpolação linear subamostral |

### Fontes de referência utilizadas
1. **Resoluções analíticas fechadas** (fórmulas exatas para raízes, pontos de quebra, cruzamentos $j\omega$, formula de Mason e formas de onda ao degrau).
2. **MATLAB R2022b (`9.13.0.2049777`) ao vivo** — utilizado tanto para gerar referências independentes dos 19 casos LGR, das 9 questões de controladores, da referência independente da Q8 e dos controladores refinados, quanto nos 3 testes opt-in do `mscript` (`cargo test --release -p mscript -- --ignored`).
3. **Referências congeladas do motor Python legado (`tests/reference/python/`, commit `be79347`)** — verificadas por `cargo run --release -p xtask -- parity` (152/152 casos, 260 ativos visuais).
