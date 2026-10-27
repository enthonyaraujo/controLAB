# Português e inglês

O campo **Configurações → Aparência → Idioma do aplicativo** alterna entre
Português (Brasil) e English. Português continua sendo o padrão. A preferência
`controlab-language` é local ao dispositivo; sua ausência ou valor inválido usa
português. A troca atualiza a interface sem recarregar ou apagar o editor.

O catálogo canônico é `src/renderer/locales/en.json`. As chaves são textos da
interface em português e os valores são as traduções inglesas. A adaptação da
interface traduz textos e nomes acessíveis, incluindo nós inseridos depois de
cálculos. Mantém os textos originais em WeakMap para a volta ao português.
Fórmulas renderizadas por KaTeX, código, saída de scripts, nomes de arquivos,
variáveis e rótulos definidos pelo autor de scripts não devem ser traduzidos.
Use `data-no-i18n` para conteúdo fornecido pelo usuário em novas interfaces.
Textos explicativos dentro de fórmulas são traduzidos antes do KaTeX.

Após editar o catálogo:

```sh
node scripts/build-locales.js
node scripts/sync-mobile.js
npm run test:js
npm run test:python
```

`en.js` é gerado para funcionar com CSP estrita, offline e `file://` no Electron.
O service worker armazena esse recurso. O catálogo também acompanha o Python
Android e o executável PyInstaller. O locale de cada requisição científica é
isolado com ContextVar; números e identificadores permanecem intactos. Os
rótulos Matplotlib são traduzidos antes de exportar PNG/SVG. O processo Electron
recebe a preferência para os diálogos nativos controlados pelo aplicativo.

## Evidências

- Verificações JS existentes: sintaxe, empacotamento, navegação, concorrência e renderer.
- Suíte Python completa: 97 testes aprovados; abertura de porta HTTP requer execução fora do sandbox.
- `tests/test_localization.py`: preferência padrão/inválida, preservação de dados, isolamento após erro e exportação SVG em cada idioma com valores numéricos equivalentes.
- `tests/i18n-browser.cjs`: alternância pelo seletor, persistência após recarga, restauração do português, inserção dinâmica, seis módulos, conteúdo do editor preservado e viewport móvel.

Para executar a verificação de navegador, inicie o servidor Web e informe sua URL:

```sh
CONTROLAB_TEST_URL=http://127.0.0.1:8766 node tests/i18n-browser.cjs
```

APK e instaladores desktop não foram gerados nesta tarefa. Os diálogos nativos e
o Android precisam de uma passagem manual nos respectivos dispositivos antes de release.
