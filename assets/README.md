# Recursos compartilhados do ControLAB

Esta é a fonte canônica dos recursos de apresentação usados pela UI e pelo motor Rust.
Fontes e ícones foram preservados byte a byte durante a remoção da interface web.
O catálogo recebeu instruções de build atualizadas para Cargo/Gradle.
`manifest.json` registra SHA-256 das fontes, ícones, catálogo e licenças.

- `fonts/ibm-plex-sans`: pesos usados na interface e nos gráficos. Licença SIL OFL.
- `fonts/jetbrains-mono`: editor, console e valores científicos. Licença SIL OFL.
- `fonts/katex`: somente arquivos TTF usados pela composição TeX nativa; não contém KaTeX JS/CSS.
- `icons`: marca do aplicativo em PNG/SVG.
- `locales/en.json`: catálogo da engine científica. A UI nativa permanece em português.

O Rust incorpora as fontes da UI durante a compilação. O motor resolve estes
recursos por `crates/control-engine/src/resources.rs`: `CONTROLAB_ASSETS`,
`resources/assets` do pacote nativo ou o checkout. O pacote não depende do
código-fonte, de Python ou de Node.

Para atualizar um recurso, revise sua procedência/licença, recalcule a entrada
SHA-256 correspondente no manifesto e execute os testes de recursos. Os
endereços das licenças são os repositórios oficiais:

- https://github.com/IBM/plex/blob/master/LICENSE.txt
- https://github.com/JetBrains/JetBrainsMono/blob/master/OFL.txt
- https://github.com/KaTeX/KaTeX/blob/main/LICENSE
