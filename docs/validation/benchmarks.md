# Benchmarks de Desempenho — Motor Rust (`--release`) vs Motor Python Legado (`be79347`)

## 1. Metodologia e Ambiente

Os benchmarks comparam o motor matemático nativo em Rust (`target/release/controlab-engine`) contra o motor Python legado extraído do último commit antes da remoção do backend Python (`be79347:src/python/native_bridge.py`), sob **condições operacionais idênticas**:

- **Protocolo:** processo trabalhador persistente JSON-lines via `stdin`/`stdout` em ambos os motores, eliminando distorção por tempo de inicialização de processo a cada requisição.
- **Aquecimento e repetições:** 3 rodadas de aquecimento (*warmup*) descartadas, seguidas de 5 a 25 iterações cronometradas por carga de trabalho, registrando mediana, P95, média, mínimo, máximo, desvio padrão e pico de memória RSS.
- **Ambiente de medição:**
  - Sistema operacional: Linux `7.3.4-300.fc45.x86_64` (`glibc 2.44`)
  - CPU: 13th Gen Intel Core i5-13420H (12 threads lógicos)
  - Compilador Rust: `rustc 1.99.1` (`--release`)
  - Runtime Python legado: `Python 3.12.14` (NumPy, SciPy, SymPy, python-control, Matplotlib)

> **Escopo das conclusões:** Os fatores de aceleração e redução de memória abaixo referem-se exclusivamente às seis cargas de trabalho medidas neste hardware e protocolo. Não devem ser extrapolados como ganho universal para qualquer operação arbitrária.

---

## 2. Resultados Quantitativos

| Carga de Trabalho | Iterações | Mediana Rust (ms) | P95 Rust (ms) | RSS Pico Rust (MB) | Mediana Python (ms) | P95 Python (ms) | RSS Pico Python (MB) | Aceleração (Mediana) | Redução de RAM (RSS) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **Parsing, cancelamento simbólico e LaTeX** (5 FTs) | 25 | **8,83** | 14,51 | **6,04** | 33,25 | 36,04 | 68,83 | **3,76×** | **11,39×** |
| **Estabilidade Routh-Hurwitz + raízes + gráfico** (Eqs. 10, 15, 18) | 10 | **88,12** | 98,06 | **21,58** | 1.109,79 | 1.182,45 | 242,54 | **12,59×** | **11,24×** |
| **Geração completa de LGR** (7 passos + 10.001 ganhos + SVG/PNG, 3 sistemas) | 10 | **212,74** | 229,81 | **27,26** | 2.742,20 | 2.803,93 | 270,16 | **12,89×** | **9,91×** |
| **Resposta temporal** (degrau/rampa/impulso, 900 pts, 3 sistemas) | 12 | **221,50** | 247,22 | **20,55** | 1.371,91 | 1.515,15 | 235,79 | **6,19×** | **11,47×** |
| **Síntese de controladores via LGR + deduções + simulação** (Q1, Q4, Q6, Q8) | 10 | **257,62** | 271,59 | **98,82** | 2.321,29 | 2.391,06 | 230,93 | **9,01×** | **2,34×** |
| **Suíte completa de 19 exercícios LGR** (190.019 avaliações de raízes + 19 figuras) | 5 | **1.049,03** | 1.118,14 | **26,68** | 16.263,50 | 16.421,91 | 291,97 | **15,50×** | **10,94×** |

Complementarmente, no motor Rust em modo `--release`:
- A redução simbólica e numérica de todos os diagramas de blocos acadêmicos via fórmula de Mason (`blocklab-core`) executa em **~20 ms** no total.
- A execução das 9 questões de controladores e modelos de LGR no interpretador `.m` (`mscript`) executa em **~10 ms**.
- A síntese refinada por restrições temporais (`temporal_refined` / `auto_simplest`) executa em **< 300 ms** por questão em build `--release`.
