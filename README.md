
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

| | |
|:---:|:---:|
| ![Studio .m](docs/screenshots/01-studio.png) | ![Central de Módulos](docs/screenshots/02-apps-hub.png) |
| **Studio .m & Command Window** | **Central de Módulos (Apps)** |
| ![Diagrama de Blocos](docs/screenshots/03-block-diagram.png) | ![Resposta no Tempo](docs/screenshots/04-time-response.png) |
| **Diagrama de Blocos** | **Resposta no Domínio do Tempo** |
| ![Análise de Estabilidade](docs/screenshots/05-stability.png) | ![Lugar Geométrico das Raízes](docs/screenshots/06-root-locus.png) |
| **Análise de Estabilidade (Routh-Hurwitz)** | **Lugar Geométrico das Raízes (LGR)** |
| ![Resposta em Frequência](docs/screenshots/07-frequency-response.png) | ![Projeto de Controladores](docs/screenshots/08-controllers.png) |
| **Resposta em Frequência (Bode & Nyquist)** | **Projeto de Controladores** |
| ![Espaço de Estados](docs/screenshots/09-state-space.png) | |
| **Espaço de Estados** | |

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
