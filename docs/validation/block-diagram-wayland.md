# Travamento das janelas no Wayland

## Reprodução e causa

Ao executar `target/debug/controlab --diagram examples/referencia.json` com
Glow e VSync padrão, a thread principal ficou bloqueada em `eglSwapBuffers`,
`dri2_wl_swap_buffers_with_damage` e `wl_display_dispatch_queue`.
O diagnóstico foi capturado com GDB, interrompendo a execução após 18 segundos.
Isso impede o processamento de eventos das duas janelas e provoca o aviso do GNOME.
Os testes headless do editor não exercitam essa sincronização nativa.

## Correção e verificação

Em `native/src/main.rs`, `GlowConfiguration.vsync = false` evita que a troca de
buffers da viewport imediata bloqueie o event loop compartilhado. O agendamento
de repaints continua sendo feito pelo egui; não foi adicionado repaint contínuo.

Na mesma sessão Wayland, a execução corrigida foi interrompida após 15 segundos:
a thread principal estava em `epoll_wait` no event loop winit/calloop, sem espera
em `eglSwapBuffers`. A execução com `CONTROLAB_SCREENSHOT` e o mesmo diagrama
gerou `/tmp/controlab-diagram-fixed.png` e encerrou com código 0 antes do timeout
de 20 segundos. A captura é temporária e não é recurso distribuído.

Build debug e Clippy da interface aprovados. Formatação e diff verificados.
Limite: esta evidência cobre o Linux/Wayland local; não valida Windows,
Android, pacote instalado ou todos os gestos e diálogos interativos.
