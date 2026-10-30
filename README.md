
<h1>
  <img src="assets/icons/app-icon.png" width="32" height="32" align="absmiddle" alt="ControLAB Logo">
  ControLAB
</h1>


Software para análise, simulação e projeto de **Sistemas de Controle**, desenvolvido em **Rust** com interface nativa em **egui/eframe**.

---

## Funcionalidades

- **Análise Linear:** Resposta no tempo, estabilidade (Routh-Hurwitz), Lugar Geométrico das Raízes e resposta em frequência (Bode e Nyquist).
- **Controle & Espaço de Estados:** Projeto de controladores e modelagem em espaço de estados.
- **Diagrama de Blocos:** Edição visual e redução de diagramas de blocos.
- **Studio `.m`:** Interpretador e editor de scripts integrado (`mscript`).
- **Exportação & Visualização:** Gráficos interativos, renderização de fórmulas e exportação para PNG/SVG.

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
