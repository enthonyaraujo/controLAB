# Como Publicar uma Nova Versão pelo GitHub Actions


## Passo a Passo

### 1. Atualizar a versão do projeto

Use o utilitário `xtask` para sincronizar a nova versão no `Cargo.toml`, `Cargo.lock` e nos arquivos de configuração Android:

```bash
cargo xtask version 6.1.1
```
*(Substitua `6.1.1` pela versão que deseja lançar).*

---

### 2. Fazer commit e push das alterações na branch

Verifique os arquivos modificados, crie o commit e envie para a sua branch no GitHub:

```bash
git status
git add -A

git commit -m "chore: prepara release v6.1.1"
git push origin main
```

---

### 3. Criar e enviar a Tag da versão (Dispara a Release)

Para disparar o job **Publicar GitHub Release** com todos os instaladores anexados, crie uma tag com o prefixo `v` e envie para o GitHub:

```bash
git tag -a v6.1.1 -m "v6.1.1"
git push origin v6.1.1
```
---

## Como reenviar uma tag caso o build falhe

Se você precisar corrigir algum erro e recriar a mesma tag (ex.: `v6.1.1`) antes da release ser publicada:

```bash
git add -A
git commit -m "fix: corrige build da release v6.1.1"
git push origin HEAD

git tag -d v6.1.1
git push origin :refs/tags/v6.1.1

git tag -a v6.1.1 -m "Release v6.1.1"
git push origin v6.1.1
```
