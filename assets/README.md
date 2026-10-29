# Recursos compartilhados do ControLAB

Esta é a fonte canônica dos recursos de apresentação usados pelo Rust e Python.
Fontes e ícones foram preservados byte a byte durante a remoção da interface web.
O catálogo recebeu instruções de build atualizadas para Cargo/Gradle.
`manifest.json` registra SHA-256 das fontes, ícones, catálogo e licenças.

- `fonts/ibm-plex-sans`: pesos usados na interface e nos gráficos. Licença SIL OFL.
- `fonts/jetbrains-mono`: editor, console e valores científicos. Licença SIL OFL.
- `fonts/katex`: somente arquivos TTF usados pelo MathText; não contém KaTeX JS/CSS.
- `icons`: marca do aplicativo em PNG/SVG.
- `locales/en.json`: catálogo da engine científica. A UI nativa permanece em português.

O Rust incorpora as fontes da UI durante a compilação. Python resolve estes
recursos por `resource_paths.py`: checkout, pacote desktop ou pacote de dados
`controlab_assets` gerado pelo projeto Android. O pacote Linux inclui os dados
em `engine/_internal/assets`; não depende do código-fonte ou de Node.

Para atualizar um recurso, revise sua procedência/licença, recalcule a entrada
SHA-256 correspondente no manifesto e execute os testes de recursos. Os
endereços das licenças são os repositórios oficiais:

- https://github.com/IBM/plex/blob/master/LICENSE.txt
- https://github.com/JetBrains/JetBrainsMono/blob/master/OFL.txt
- https://github.com/KaTeX/KaTeX/blob/main/LICENSE
