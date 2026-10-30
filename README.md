
<h1>
  <img src="assets/icons/app-icon.png" width="32" height="32" align="absmiddle" alt="ControLAB Logo">
  ControLAB
</h1>

[🇧🇷 Português](README.md) | [🇺🇸 English](README_EN.md)

Software para análise, simulação e projeto de **Sistemas de Controle**, desenvolvido em **Rust** com interface nativa em **egui/eframe**.

<p align="center">
  <a href="docs/screenshots/01-studio.png">
    <img src="docs/screenshots/01-studio.png" alt="ControLAB Studio" width="100%">
  </a>
</p>

---

## Funcionalidades

- **Análise Linear:** Resposta no tempo, estabilidade (Routh-Hurwitz), Lugar Geométrico das Raízes e resposta em frequência (Bode e Nyquist).
- **Controle & Espaço de Estados:** Projeto de controladores e modelagem em espaço de estados.
- **Diagrama de Blocos:** Edição visual e redução de diagramas de blocos.
- **Studio `.m`:** Interpretador e editor de scripts integrado (`mscript`).
- **Exportação & Visualização:** Gráficos interativos, renderização de fórmulas e exportação para PNG/SVG.

## Interface e Módulos

O ControLAB possui uma interface moderna e responsiva, projetada para cálculo analítico, simulação dinâmica e projeto de controladores. Veja abaixo as telas da aplicação e de cada ferramenta (clique na imagem para abrir em alta resolução):

### 1. Studio `.m` & Linha de Comando (Command Window)
Ambiente interativo com workspace de variáveis, execução de scripts `.m` e comandos imediatos via interpretador `mscript`.

<p align="center">
  <a href="docs/screenshots/01-studio.png">
    <img src="docs/screenshots/01-studio.png" alt="Studio e Command Window" width="100%">
  </a>
</p>

### 2. Central de Módulos (Apps Hub)
Acesso rápido e centralizado a todos os módulos de análise e projeto de sistemas de controle.

<p align="center">
  <a href="docs/screenshots/02-apps-hub.png">
    <img src="docs/screenshots/02-apps-hub.png" alt="Central de Módulos - Apps Hub" width="100%">
  </a>
</p>

### 3. Diagrama de Blocos
Editor e visualizador com redução automática, cálculo pela Regra de Mason e determinação da função de transferência de malha fechada.

<p align="center">
  <a href="docs/screenshots/03-block-diagram.png">
    <img src="docs/screenshots/03-block-diagram.png" alt="Diagrama de Blocos e Regra de Mason" width="100%">
  </a>
</p>

### 4. Resposta no Domínio do Tempo
Simulação temporal para degrau, rampa e impulso, com métricas transitórias automáticas (sobressinal $\%OS$, tempos de pico $t_p$, subida $t_r$ e acomodação $t_s$) e constantes de erro estático ($K_p$, $K_v$, $K_a$).

<p align="center">
  <a href="docs/screenshots/04-time-response.png">
    <img src="docs/screenshots/04-time-response.png" alt="Resposta no Domínio do Tempo" width="100%">
  </a>
</p>

### 5. Análise de Estabilidade
Avaliação de estabilidade pelo Critério de Routh-Hurwitz paramétrico por ganho $K$, classificação de regimes (estável, marginalmente estável e instável) e mapeamento das raízes no plano complexo $s$.

<p align="center">
  <a href="docs/screenshots/05-stability.png">
    <img src="docs/screenshots/05-stability.png" alt="Análise de Estabilidade e Routh-Hurwitz" width="100%">
  </a>
</p>

### 6. Lugar Geométrico das Raízes (LGR)
Traçado analítico de Evans com identificação de polos, zeros, assíntotas, centroide ($\sigma_a$), pontos de ruptura (*break-in / break-out*), regras clássicas e memorial de cálculo passo a passo.

<p align="center">
  <a href="docs/screenshots/06-root-locus.png">
    <img src="docs/screenshots/06-root-locus.png" alt="Lugar Geométrico das Raízes (LGR)" width="100%">
  </a>
</p>

### 7. Resposta em Frequência (Bode e Nyquist)
Diagramas de Bode (magnitude e fase com curvas reais e assíntotas) e Diagrama Polar de Nyquist com ponto crítico $(-1, 0)$, margens de ganho ($MG$) e de fase ($MF$), frequências de cruzamento e estimativas temporais.

<p align="center">
  <a href="docs/screenshots/07-frequency-response.png">
    <img src="docs/screenshots/07-frequency-response.png" alt="Resposta em Frequência - Bode e Nyquist" width="100%">
  </a>
</p>

### 8. Projeto de Controladores
Síntese analítica de compensadores e controladores clássicos (Avanço de Fase / Lead, Atraso / Lag, P, PD, PI, PID), com traçado simultâneo do LGR compensado e comparação da resposta ao degrau antes e após a compensação.

<p align="center">
  <a href="docs/screenshots/08-controllers.png">
    <img src="docs/screenshots/08-controllers.png" alt="Projeto de Controladores e Compensadores" width="100%">
  </a>
</p>

### 9. Espaço de Estados
Modelagem contínua matricial $(\mathbf{A}, \mathbf{B}, \mathbf{C}, \mathbf{D})$, conversão para formas canônicas, verificação de postos de controlabilidade e observabilidade, alocação de polos por realimentação de estados $(\mathbf{A} - \mathbf{B}\mathbf{K})$ via fórmula de Ackermann e síntese de observador de Luenberger.

<p align="center">
  <a href="docs/screenshots/09-state-space.png">
    <img src="docs/screenshots/09-state-space.png" alt="Espaço de Estados e Realimentação" width="100%">
  </a>
</p>

## Como Executar

### Pré-requisitos
- [Rust & Cargo](https://www.rust-lang.org/) instalados.
- Bibliotecas gráficas para compilação com `eframe` (no Linux: drivers gráficos, Wayland/X11, libxkbcommon).

### Execução

```bash
# Clonar o repositório
git clone https://github.com/enthonyaraujo/controLAB.git
cd controLAB

# Compilar e executar
cargo run --bin controlab

# Ou em modo otimizado
cargo run --release --bin controlab
```

> **Dica:** Você também pode usar o script `./run.sh`.

## Como Contribuir

Contribuições são muito bem-vindas! Para contribuir:

1. Faça um **Fork** do repositório.
2. Crie uma branch para a sua alteração:
   ```bash
   git checkout -b feat/minha-feature
   ```
3. Faça o commit das suas mudanças:
   ```bash
   git commit -m "feat: adiciona minha feature"
   ```
4. Envie para o seu fork:
   ```bash
   git push origin feature/minha-feature
   ```
5. Abra um **Pull Request**.

Antes de enviar, verifique se os testes e o padrão de código passam:
```bash
cargo test --workspace
cargo fmt --all --check
```

## Licença

Distribuído sob a licença [MIT](LICENSE).
