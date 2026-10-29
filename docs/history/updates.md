# ControLAB: melhorias de estudo e consumo

Trabalho na branch `feat/updates`, a partir de `6454b8b` (v3.0.2).

## Direção visual

A skill pessoal `frontend-design` foi aplicada ao ambiente educacional, em conjunto com as decisões do segundo cérebro. O plano complexo é o elemento principal; entradas e resolução usam alinhamento à esquerda. A organização permanece `entrada | gráfico ou passo a passo` no desktop e `entrada → resultado` no celular. O Hub mantém sua identidade aprovada.

Tokens preservados: fundo `#080c14`, painel `#0e1422`, superfície `#121929`, texto `#f8fafc`, texto secundário `#94a3b8`, ação `#2563eb`. Tipografia usa a pilha local já definida: Inter/Segoe UI/sans-serif para interface, JetBrains Mono/Consolas/monospace para expressões. Não há download de fontes no shell. KaTeX permanece embarcado.

A revisão do plano priorizou legibilidade de equações e geometria verdadeira, com escalas iguais no plano complexo, em vez de ornamentação. Opções secundárias ficam recolhidas, rótulos são curtos e os controles de toque recebem altura mínima de 48 CSS px quando o dispositivo anuncia ponteiro impreciso.

## Motor e interação

- Escalas iguais em ambos os eixos e limites imaginários simétricos. Os ângulos dos pares conjugados são desenhados nos dois semiplanos; a seta sai do polo ou aponta para o zero conforme K aumenta. O memorial conserva a dedução do representante superior para evitar duplicação.
- Amostragem original preservada: K=0 e 10.000 ganhos logarítmicos entre 0,001 e 10.000. A detecção de trocas de sinal é vetorizada; esta etapa não amplia a faixa de ganhos nem transforma a busca amostrada em prova analítica global.
- Imports científicos sob demanda, prévias serializadas com espera de 450 ms, descarte de respostas antigas, reutilização da última entrada válida e do gráfico atual, e renderização do memorial apenas quando aberto.
- O parser restrito também passa a atender a chamada direta por expressão; figuras são fechadas mesmo em falha de exportação.

## Evidências

- `npm test`: 40 testes Python, verificação de integração do renderer e teste comportamental de concorrência/prévias.
- `npm run mobile:sync`: motor e assets sincronizados com Android.
- Inspeção do LGR em 360×800, 812×375, 768×1024, 1024×768, 1366×768 e 1920×1080: sem transbordamento horizontal. Temas escuro e claro, cálculo de zeros complexos e abertura do passo a passo verificados no navegador; sem erros de console.
- Prévia em processo Python novo: mediana de cinco execuções, 1,424 s antes e 0,445 s depois (aproximadamente 69% menos tempo), neste computador. Caso: `1/(s*(s+1))`; mesmo interpretador `.venv/bin/python`, cache Matplotlib e ambiente. Base comparada: arquivos de `6454b8b`. Não representa medição de bateria.
- Regressões numéricas verificam ângulos contra raízes de malha fechada próximas às singularidades e cruzamento em K=6, |ω|=√2 para `1/(s(s+1)(s+2))`.

## Limites de validação

A inspeção responsiva usa o navegador com o mesmo renderer e a API Python local. Não substitui testes físicos de Android, consumo de bateria nem empacotamento Electron em Windows/Linux.

O build Android foi interrompido: apesar de `--offline` no Gradle, a tarefa Chaquopy iniciou resolução/instalação de requisitos Python no diretório de build, inclusive download de uma dependência transitiva. Nenhum APK desta branch foi validado. Verificar as dependências de build antes de repetir a compilação.

## Referências técnicas

- [Matplotlib: escalas iguais](https://matplotlib.org/stable/api/_as_gen/matplotlib.axes.Axes.set_aspect.html)
- [python-control: root_locus_map](https://python-control.readthedocs.io/en/stable/generated/control.root_locus_map.html)
