# Validação Acadêmica — LGR e Projeto de Controladores (UFERSA)

Esta validação reproduz integralmente as duas listas acadêmicas da disciplina de **Sistemas de Controle II (PEX0257)** da UFERSA nos módulos `controlab-engine`, `blocklab-core`, `mscript` e `controlab-native`, comparando os resultados com derivações analíticas fechadas e execuções independentes no **MATLAB R2022b**.

---

## 1. Lugar Geométrico das Raízes (`Exercicios_-_LGR.pdf` — 19 casos)

Foram validados todos os **19 casos** (Equações 1 a 18 das Questões 1 a 4 + Equação 19 paramétrica da Questão 5 com $a_2 = 0$), preservando integralmente a duplicidade de rótulo `2(i)-1` (Eq. 12) / `2(i)-2` (Eq. 13) e todos os casos matematicamente equivalentes (`1(a) = 2(d)`, `1(b) = 2(c)`, `1(c) = 2(i)-1 = 3(a)`).

| ID | Item / Eq. | $G(s)H(s)$ | Polos ($K=0$) | Zeros | Centroide / Assíntotas | Pontos de Quebra | Cruzamento $j\omega$ / $K_{\text{crit}}$ | Estabilidade ($K>0$) | Status (4 cat.) |
|---|---|---|---|---|---|---|---|---|---|
| `LGR_Q1A_EQ01` | 1(a) / (1) | $\frac{k}{s(s^2+6s+25)}$ | $0, -3\pm 4j$ | — | $-2$; $60^\circ,180^\circ,300^\circ$ | Nenhum real | $\pm 5j$ ($K=150$) | $0<K<150$ | **PASS** |
| `LGR_Q1B_EQ02` | 1(b) / (2) | $\frac{k(s+2)}{s^2+2s+3}$ | $-1\pm\sqrt{2}j$ | $-2$ | $0$; $180^\circ$ | $-3{,}7321$ ($K=5{,}4641$) | Nenhum | $K>0$ | **PASS** |
| `LGR_Q1C_EQ03` | 1(c) / (3) | $\frac{k}{s(s^2+4s+5)}$ | $0, -2\pm j$ | — | $-4/3$; $60^\circ,180^\circ,300^\circ$ | $-5/3$ ($K=50/27$), $-1$ ($K=2$) | $\pm\sqrt{5}j$ ($K=20$) | $0<K<20$ | **PASS** |
| `LGR_Q2A_EQ04` | 2(a) / (4) | $\frac{k}{s(s+4)}$ | $0, -4$ | — | $-2$; $90^\circ,270^\circ$ | $-2$ ($K=4$) | Nenhum | $K>0$ | **PASS** |
| `LGR_Q2B_EQ05` | 2(b) / (5) | $\frac{k}{s(s+1)(s+2)}$ | $0, -1, -2$ | — | $-1$; $60^\circ,180^\circ,300^\circ$ | $-0{,}4226$ ($K=0{,}3849$) | $\pm\sqrt{2}j$ ($K=6$) | $0<K<6$ | **PASS** |
| `LGR_Q2C_EQ06` | 2(c) / (6) | $\frac{k(s+2)}{s^2+2s+3}$ | $-1\pm\sqrt{2}j$ | $-2$ | $0$; $180^\circ$ | $-3{,}7321$ ($K=5{,}4641$) | Nenhum | $K>0$ | **PASS** |
| `LGR_Q2D_EQ07` | 2(d) / (7) | $\frac{k}{s(s^2+6s+25)}$ | $0, -3\pm 4j$ | — | $-2$; $60^\circ,180^\circ,300^\circ$ | Nenhum real | $\pm 5j$ ($K=150$) | $0<K<150$ | **PASS** |
| `LGR_Q2E_EQ08` | 2(e) / (8) | $\frac{k(s+2)(s+3)}{s(s+1)}$ | $0, -1$ | $-2, -3$ | Grau relativo 0 | $-0{,}6340$, $-2{,}3660$ | Nenhum | $K>0$ | **PASS** |
| `LGR_Q2F_EQ09` | 2(f) / (9) | $\frac{k(s^2+2s+8)}{s(s+10)(s+5)}$ | $0, -5, -10$ | $-1\pm\sqrt{7}j$ | $-13$; $180^\circ$ | $-1{,}6679$ ($K=6{,}2189$) | Nenhum | $K>0$ | **PASS** |
| `LGR_Q2G_EQ10` | 2(g) / (10) | $\frac{k(s^2+4)}{(s+2)^2(s+5)(s+6)}$ | $-2$ (2×), $-5, -6$ | $\pm 2j$ | $-7{,}5$; $90^\circ,270^\circ$ | $-5{,}5307$ ($K=0{,}0898$), $-2$ ($K=0$) | $\to\pm 2j$ ($K\to\infty$) | $K>0$ | **PASS** |
| `LGR_Q2H_EQ11` | 2(h) / (11) | $\frac{k}{(s+1)(s^2+4s+5)}$ | $-1, -2\pm j$ | — | $-5/3$; $60^\circ,180^\circ,300^\circ$ | Nenhum real | $\pm 3j$ ($K=40$) | $0<K<40$ | **PASS** |
| `LGR_Q2I1_EQ12` | 2(i)-1 / (12) | $\frac{k}{s(s^2+4s+5)}$ | $0, -2\pm j$ | — | $-4/3$; $60^\circ,180^\circ,300^\circ$ | $-5/3$, $-1$ | $\pm\sqrt{5}j$ ($K=20$) | $0<K<20$ | **PASS** |
| `LGR_Q2I2_EQ13` | 2(i)-2 / (13) | $\frac{k(s+2)}{(s+1)(s^2+6s+10)}$ | $-1, -3\pm j$ | $-2$ | $-2{,}5$; $90^\circ,270^\circ$ | Nenhum real | Nenhum | $K>0$ | **PASS** |
| `LGR_Q3A_EQ14` | 3(a) / (14) | $\frac{k}{s(s^2+4s+5)}$ ($H=1$) | $0, -2\pm j$ | — | $-4/3$; $60^\circ,180^\circ,300^\circ$ | $-5/3$, $-1$ | $\pm\sqrt{5}j$ ($K=20$) | $0<K<20$ | **PASS** |
| `LGR_Q3B_EQ15` | 3(b) / (15) | $\frac{k}{s(s+1)(s^2+4s+13)}$ | $0, -1, -2\pm 3j$ | — | $-1{,}25$; $45^\circ,135^\circ,225^\circ,315^\circ$ | $-0{,}4664$ ($K=2{,}8252$) | $\pm\sqrt{2{,}6}j$ ($K=37{,}44$) | $0<K<37{,}44$ | **PASS** |
| `LGR_Q3C_EQ16` | 3(c) / (16) | $\frac{k(s+1)}{s^2(s+3{,}6)}$ | $0$ (2×), $-3{,}6$ | $-1$ | $-1{,}3$; $90^\circ,270^\circ$ | $0$ ($K=0$) | Nenhum ($K>0$) | $K>0$ | **PASS** |
| `LGR_Q4A_EQ17` | 4(a) / (17) | $\frac{k(s+2)}{s^2+4s+13}$ | $-2\pm 3j$ | $-2$ | $-2$; $180^\circ$ | $-5$ ($K=6$) | Nenhum | $K>0$ | **PASS** |
| `LGR_Q4B_EQ18` | 4(b) / (18) | $\frac{k(s-2)(s-4)}{s^2+6s+25}$ | $-3\pm 4j$ | $+2, +4$ | Grau relativo 0 | $2{,}8853$ ($K=51{,}3118$) | $\pm\sqrt{16{,}5}j$ ($K=1$) | $0<K<1$ | **PASS** |
| `LGR_Q5_EQ19` | 5(a,b) / (19) | $\frac{a_1(s+1)}{s^3}$ ($a_2=0$) | $0$ (3×) | $-1$ | $+0{,}5$; $90^\circ,270^\circ$ | $0$ ($K=0$) | Nenhum ($a_1>0$) | Instável $\forall a_1>0$ | **PASS** |

- **Verificação da equação característica $D(s)+KN(s)=0$:** validada em 190.019 conjuntos de raízes com resíduo normalizado $\le 2{,}13\times 10^{-14}$ (raízes simples) e $\le 3{,}14\times 10^{-8}$ (raízes múltiplas), preservando simetria conjugada em 100% dos ganhos.
- **Reduções via Mason (`blocklab-core`):** Eqs. 14, 15 e 16 reduzidas simbolicamente de forma exata em `crates/blocklab-core/tests/academic_diagrams.rs`.

---

## 2. Projeto de Controladores (`Projeto_de_controladores.pdf` — 9 questões)

### 2.1 Auditoria da Síntese Clássica por LGR (`lgr_classic`)

Todos os casos clássicos atingem `NUMERICAL_PASS` frente ao MATLAB R2022b, porém apenas os casos que satisfazem integralmente as restrições reais do enunciado recebem `REQUIREMENTS_PASS`:

| Questão | Planta | Controlador Clássico | $M_p$ real (Rust / MATLAB) | $t_s(2\%)$ real (Rust / MATLAB) | $e_a(\infty)$ | `NUM` | `REQ` | `OVERALL` | Observação crítica |
|---|---|---|---:|---:|---:|---|---|---|---|
| **Q1** | $\frac{1}{(s+0{,}2)(s+0{,}644)}$ | PI ($K_p=0{,}4985, K_i=0{,}0998$) | `20.00%` / `20.00%` | `11.80 s` / `11.79 s` | `0` | PASS | PASS* | **PASS*** | Quase-cancelamento do 3º polo ($-0{,}200128$) pelo zero ($-0{,}200105$). Ganho DC gráfico unitário apenas reescala $K_p, K_i$ por $1/0{,}1288$. |
| **Q2** | $\frac{1}{s^2+1}$ | PD ($K_p=3, K_d=2$) | `41.26%` / `41.25%` | `3.723 s` / `3.723 s` | `0.25` | PASS | **FAIL** | **FAIL** | Zero do PD em $-1{,}5$ eleva $M_p$ para $41{,}25\%$ (viola $\le 16{,}3\%$) e $t_s \ne 4\text{ s}$. |
| **Q2 (alt)** | $\frac{1}{s^2+1}$ | PID `equal` ($3{,}737; 1{,}474; 2{,}369$) | `13.99%` / `13.97%` | `7.639 s` / `7.639 s` | `0` | PASS | **FAIL** | **FAIL** | Polo real lento em $-0{,}3685$ dobra $t_s$ para $7{,}639\text{ s}$ (viola $t_s = 4\text{ s}$). |
| **Q3** | $\frac{3}{20s(s+1)(s+5)}$ | PD ($K_p=53{,}33, K_d=33{,}33$) | `8.09%` / `8.09%` | `3.914 s` / `3.913 s` | `0` | PASS | PASS* | **PASS*** | Malha interna reduzida exatamente; alerta para razão de dominância $4 < 5$ e zero em $-1{,}6$. |
| **Q4** | $G=\frac{5}{s+0{,}1}, H=0{,}05$ | PI ($K_p=7{,}6, K_i=16$) | `28.39%` / `28.39%` | `3.764 s` / `3.759 s` | `0` | PASS | PASS* | **PASS*** | Erro no somador $e_a(\infty)=0$ atendido; alerta explícito para $y(\infty) = 1/H(0) = 20$ ($r-y = -19$). |
| **Q5** | $\frac{1}{s^3+10s^2+17s+8}$ | PID `cancel`/`equal` ($200; 100; 100$) | `25.38%` / `25.37%` | `0.841 s` / `0.841 s` | `0` | PASS | **FAIL** | **FAIL** | Envelope $4/\sigma=1$ resulta em $t_s = 0{,}841\text{ s}$ (viola igualdade literal $1{,}0 \pm 0{,}02\text{ s}$) e usa cancelamento duplo frágil $(s+1)^2$. |
| **Q6** | $\frac{1}{s(s+2)^2}$ | PD ($K_p=8, K_d=4$) | `16.30%` / `16.29%` | `4.038 s` / `4.038 s` | `0` | PASS | PASS* | **PASS*** | Alocação direta de $(\xi=0{,}5, \omega_n=2)$; alerta de cancelamento nominal em $-2$. |
| **Q7** | $\frac{5}{s(s+2)^2}$ | PD ($K_p=1{,}6, K_d=0{,}8$) | `16.30%` / `16.29%` | `4.038 s` / `4.038 s` | `0` | PASS | **FAIL** | **FAIL** | Viola limite literal $M_p \le 16{,}0\%$ devido ao conflito didático $\xi=0{,}5 \Rightarrow 16{,}303\%$. |
| **Q8** | $\frac{-7{,}2}{5s+1}$ | PI ($K_p=-1{,}25, K_i=-4{,}261$) | `33.84%` / `33.84%` | `3.966 s` / `3.965 s` | `0` | PASS | **FAIL** | **FAIL** | Zero do PI em $-3{,}409$ eleva $M_p$ para $33{,}84\%$ (viola $\le 25\%$). Perturbação rejeitada ($T_d(0)=0$). |
| **Q9** | $\frac{1}{s^2}$ | PD ($K_p=4, K_d=2$) | `29.84%` / `29.84%` | `3.753 s` / `3.753 s` | `0` | PASS | **FAIL** | **FAIL** | Zero do PD em $-2$ eleva $M_p$ para $29{,}84\%$ (viola $\le 16\%$). |

---

## 3. Caso de Referência Independente — Questão 8 ($G_c(s) = -1 - 0{,}2/s$)

Para $G(s) = \frac{-7{,}2}{5s+1}$, a solução nominal independente $K_p = -1{,}0, K_i = -0{,}2$ foi verificada (sem hardcode no algoritmo) no motor Rust, `blocklab-core`, `mscript`, módulo de resposta temporal e MATLAB R2022b:

- **Malha fechada de referência:** $T(s) = \frac{1{,}44}{s + 1{,}44}$ (redução de Mason exata no `blocklab-core`; erro máximo de forma de onda vs $1 - e^{-1{,}44t}$ inferior a $10^{-10}$ no `mscript` e $1{,}2\times 10^{-14}$ no MATLAB).
- **Métricas nominais:** $M_p = 0{,}000\%$, $t_s(2\%) = 2{,}71668\text{ s}$ (analítico $\ln(50)/1{,}44$), $y(\infty) = 1{,}0$, $e_{ss} = 0{,}0$.
- **Rejeição à perturbação constante:** $T_d(s) = \frac{-1{,}44s}{(s+0{,}2)(s+1{,}44)}$, $T_d(0) = 0{,}0$ (pico transitório $0{,}7273\text{ rad/s}$).
- **Estabilidade interna e sensibilidade ($\pm 10\%$ em $k$ e $\tau$):** polinômio característico não reduzido $5(s+0{,}2)(s+1{,}44)$ estritamente no SPE; sob variações de $\pm 10\%$ em $k$ e $\tau$, mantém estabilidade interna, $M_p \le 0{,}565\%$ e $t_s(2\%) \le 3{,}019\text{ s} < 4\text{ s}$.

---

## 4. Resolução Completa da Questão 5 com Igualdade Obrigatória $\xi = 0{,}4$

Para $G(s) = \frac{1}{s^3 + 10s^2 + 17s + 8}$ sob as quatro restrições simultâneas (**PID**, $e_{ss} = 0$, $t_s(2\%) = 1{,}00 \pm 0{,}02\text{ s}$ e **$\xi = 0{,}40 \pm 0{,}01$ obrigatório**), o motor genérico `temporal_refined` encontra automaticamente:

- **Ganhos:** $K_p = 227{,}1227,\; K_i = 56{,}8652,\; K_d = 80{,}9373$.
- **Polos e zeros:** par complexo $-3{,}4896 \pm 7{,}9956j$ ($\xi = 0{,}400000$, $\omega_n = 8{,}7239\text{ rad/s}$) + polos reais $-2{,}7491$ e $-0{,}2718$ quase cancelados pelos zeros reais $-2{,}5283$ e $-0{,}2779$.
- **Dominância modal e estabilidade interna:** todos os 4 polos no SPE; razão exata de resíduos modais de Heaviside dos polos reais frente ao par complexo igual a **`0,1056`** (~90% do resíduo transitório pertence ao modo $\xi = 0{,}4$).
- **Métricas (Rust / MATLAB R2022b):** $t_s(2\%) = 0{,}9969\text{ s}$ / $0{,}9970\text{ s}$, $e_{ss} = 0{,}00000$, $M_p = 28{,}844\%$ / $28{,}836\%$.
- **Robustez paramétrica verificada no MATLAB R2022b:** sob deslocamento de $\pm 5\%$ no polo duplo da planta em $-1$, o sistema mantém estabilidade interna com $\xi \in [0{,}395, 0{,}405]$ e $t_s \in [0{,}992\text{ s}, 1{,}004\text{ s}]$ (ainda dentro das metas); sob $\pm 10\%$ no ganho da planta, permanece estável com $\xi \in [0{,}381, 0{,}422]$ e $t_s \in [0{,}938\text{ s}, 1{,}070\text{ s}]$.
