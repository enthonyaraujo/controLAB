# Editor .m na página inicial — 2026-10-20

## Escopo implementado

Página inicial com toolbar, pasta navegável e busca, editor com múltiplas abas, realce de sintaxe e linhas, confirmação ao descartar alterações, salvamento, console persistente, variáveis e figuras SVG. F5 executa o script; Ctrl+S/Ctrl+O/Ctrl+N salvam/abrem/criam documentos. Os módulos científicos continuam acessíveis pela navegação existente.

O núcleo e o gerenciamento de documentos foram copiados do openMAT; fontes e licença estão em `vendor/openmat/`. O adaptador próprio comunica-se por JSON com um processo Rust persistente; o Python gerencia seu ciclo de vida e o Electron usa IPC. A Web usa a mesma ação `script_request`, com identificadores isolados por sessão. O backend rejeita origens de navegador diferentes da origem local e hosts fora de localhost/IP, antes de permitir ações de arquivo.

A origem openMAT já tinha alterações locais: elas foram preservadas. Os 33 arquivos copiados foram comparados byte a byte com a origem, e seus hashes foram registrados em `vendor/openmat/SOURCE_MANIFEST.json`. Não há dependência de execução do diretório original.

## Validação observada

- `CARGO_NET_OFFLINE=true npm test`: verificações JS e 76 testes Python aprovados; compilação Rust usando o cache já disponível.
- `cargo test --offline --locked --manifest-path vendor/openmat/Cargo.toml --target-dir build/openmat`: 79 testes dos crates copiados e do gerenciador de documentos aprovados; três testes diferenciais MATLAB ignorados por desenho.
- `npm run test:studio:browser`: abertura, edição, salvamento real de arquivos temporários, F5, console com variáveis persistentes, plot, erro com localização, limpeza de variáveis, proteção de aba alterada, temas claro/escuro, navegação científica e larguras 360/768/1024/1440 px aprovados no Chromium instalado. Não foram baixados browsers.
- Electron nativo iniciado e inspecionado via CDP local: ponte IPC, script matricial (`19 22 / 43 50`), comando posterior `det(A)` e figura de `plot` aprovados. As verificações de UI do Electron usaram eventos DOM; as interações de teclado e ponteiro foram verificadas no navegador.
- Sincronização Python Android, sintaxe JS, configuração de empacotamento e `git diff --check` aprovados.

## Limites e próximos passos

- Conforme escolha do usuário, esta etapa entrega Electron e Web. Portabilidade do motor Rust para execução `.m` offline no Android fica para uma etapa própria. A UI informa indisponibilidade do motor quando não há executável, preservando os cálculos científicos existentes.
- Rust/Cargo é necessário no desenvolvimento; o executável precisa acompanhar os pacotes desktop, junto das licenças. `npm run package:python` foi ajustado para compilar o motor. Não foram gerados instaladores; PyInstaller não está instalado no ambiente atual e nenhuma dependência foi instalada para esse fim. Diálogos nativos de arquivos ainda requerem verificação manual.
- O interpretador tem o mesmo subconjunto de linguagem do openMAT. Não há cobertura integral de MATLAB nem de arquivos `.mat`. Variáveis não são persistidas após encerrar o processo.
- `rustfmt` não está disponível; os crates copiados mantêm sua formatação original.
