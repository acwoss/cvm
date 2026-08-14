# Hooks de ciclo de vida de ambiente

## Contexto

O `cvm` isola configuração do Claude Code por ambiente (`~/.cvm/envs/<nome>`)
e já expõe pontos de extensão simples (`.env` por ambiente, `--inherit` na
criação, export/import via `cvm.yaml`). Falta um jeito de o usuário rodar
scripts próprios em momentos do ciclo de vida de um ambiente — por exemplo,
notificar quando um ambiente é ativado, ou impedir a remoção de um ambiente
que ainda tenha processos rodando.

Esta feature adiciona **hooks globais**: scripts que o usuário coloca em
`~/.cvm/hooks/` e que o `cvm` executa automaticamente nos eventos definidos
abaixo, para qualquer ambiente.

## Objetivo

Permitir configurar scripts que rodam:

- depois que um ambiente é criado;
- antes/depois de um ambiente ser ativado (`cvm use`);
- antes/depois de um ambiente ser desativado (`cvm deactivate`);
- antes/depois de um ambiente ser removido (`cvm remove`).

## Não-objetivos (fora de escopo desta versão)

- Hooks por ambiente individual (só existe o conjunto global).
- Hooks para `cvm run`, `cvm open` ou `cvm import`.
- Um comando `cvm hooks` para listar/scaffoldar hooks.
- Incluir hooks no `cvm.yaml` exportável — hooks continuam sendo puramente
  locais à máquina, para preservar a garantia de "safe by construction" que
  hoje vale para `cvm export`/`cvm import` (nunca executa código arbitrário
  vindo de um manifesto de outra pessoa).

Qualquer um desses pontos pode virar uma extensão futura, mas não faz parte
desta entrega.

## Onde os hooks moram

Diretório global, ao lado de `envs/`:

```
~/.cvm/hooks/
  post-create
  pre-activate
  post-activate
  pre-deactivate
  post-deactivate
  pre-remove
  post-remove
```

Resolvido a partir de `cvm_home()` (o mesmo helper que já resolve
`~/.cvm/envs`), então respeita a variável `CVM_HOME` já usada hoje para
testes e overrides.

Convenção estilo git hooks: um arquivo por evento, sem parsing de config.
Se o arquivo não existir, o evento é ignorado silenciosamente. Se existir
mas (no Unix) não tiver bit de execução, o `cvm` avisa no stderr e ignora
(não bloqueia a operação em nenhum caso — nem para hooks `pre-*`, já que
"hook mal configurado" não deve impedir o uso normal da ferramenta).

No Windows, o arquivo correspondente é `<evento>.cmd` (mesmo padrão já usado
pelos shims em `bin/claude.cmd`), invocado via `cmd /C`.

## Eventos e pontos de disparo

| Evento | Onde dispara | Bloqueia a operação se falhar? |
|---|---|---|
| `post-create` | `env::create_env`, após o layout do ambiente (skills/, bin/, .env) estar pronto | Não |
| `pre-activate` | `env::resolve_activate`, antes de montar as variáveis a exportar | **Sim** |
| `post-activate` | `env::resolve_activate`, antes de retornar as variáveis | Não |
| `pre-deactivate` | `env::resolve_deactivate`, só quando há um ambiente ativo | **Sim** |
| `post-deactivate` | `env::resolve_deactivate`, só quando há um ambiente ativo | Não |
| `pre-remove` | `env::remove_env`, antes de `fs::remove_dir_all` | **Sim** |
| `post-remove` | `env::remove_env`, depois da remoção | Não |

Regra de falha: hooks `pre-*` que saem com código diferente de zero abortam
a operação (o `cvm` imprime o erro e não continua) — mesmo comportamento que
um `pre-commit` hook do git. Hooks `post-*` nunca bloqueiam: a operação
principal já aconteceu, então uma falha do hook apenas gera um aviso no
stderr; o comando `cvm` ainda retorna sucesso.

`cvm run` e `cvm open` não passam por `resolve_activate`/`resolve_deactivate`
(eles montam as variáveis diretamente para o processo filho), então não
disparam hooks — consistente com o não-objetivo acima.

### Nota sobre timing de pre/post-activate e pre/post-deactivate

A ativação/desativação "de verdade" (exportar ou remover variáveis na sessão
do shell) acontece no wrapper de shell instalado por `cvm init`, não dentro
do binário `cvm` — o binário não consegue mutar o ambiente do processo pai
que o invocou. Os hooks, porém, rodam como subprocessos com as variáveis
`CVM_ENV`/`CVM_ENV_PATH` injetadas explicitamente pelo próprio `cvm` (não
herdadas do shell). Por isso, o "pre/post" é relativo à lógica interna do
`cvm` (`resolve_activate`/`resolve_deactivate`), não ao momento exato em que
o prompt do shell muda — na prática isso não afeta o que o script consegue
observar ou fazer.

Consequência prática: `resolve_deactivate()` precisa deixar de ser infalível
(`Vec<String>`) e passar a retornar `Result<Vec<String>>`, para poder
propagar a falha de um `pre-deactivate`. `cmd_resolve_deactivate` em
`main.rs` passa a propagar esse erro. Isso já é suficiente para os wrappers
de shell abortarem corretamente, já que todos eles já fazem
`... || return $?` ao chamar `__resolve-deactivate`.

## Variáveis de ambiente injetadas no hook

Além de herdar o ambiente do processo que chamou o `cvm`, cada hook recebe:

- `CVM_HOOK_EVENT` — nome do evento (ex.: `post-create`, `pre-remove`)
- `CVM_ENV` — nome do ambiente envolvido
- `CVM_ENV_PATH` — caminho absoluto de `~/.cvm/envs/<nome>` (em
  `post-remove`, o diretório já não existe mais no disco, mas o valor
  continua sendo informativo)

## Testes

- Testes unitários em um novo módulo `src/hooks.rs`, seguindo o padrão já
  usado em `env.rs` (override de `CVM_HOME` via `tempfile::tempdir()`,
  serializado por um mutex para evitar corrida entre testes que mexem em
  variáveis de processo).
- Casos a cobrir: hook ausente é ignorado; hook presente e executável roda
  e recebe as três variáveis corretas; hook `pre-*` com saída != 0 aborta
  (propaga erro) e a operação principal não acontece (ex.: ambiente não é
  removido se `pre-remove` falhar); hook `post-*` com saída != 0 apenas
  avisa e a operação principal já efetivada permanece; hook existente sem
  permissão de execução (Unix) é ignorado com aviso, não bloqueia.
- Testes de integração em `create_env`/`resolve_activate`/
  `resolve_deactivate`/`remove_env` confirmando que os hooks certos disparam
  nos pontos certos.
