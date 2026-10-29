# Root (user-home) install

`symkit install --root` runs a normal install with `$HOME` as the target
(`%USERPROFILE%` on Windows). Grok, Claude, and Codex then see the harness
in every session. Project `install`, `init`, and `adapt` are unchanged.

```bash
symkit install --root --harness engineering --role engineer --yes
```

`--harness`, `--role`, `--pack`, `--also`, `--adapters`, `--adapter`,
`--adapters-all`, `--no-prune`, `--yes`, `--dry-run`, and `--force` work
as they do on a project. Do not pass a directory together with `--root`.

When you omit the adapter flags, `--root` selects `grok`, `claude`, and
`codex`. An explicit set wins, including `--adapters grok` or
`--adapters none`. A project install still defaults to `grok`.

## What is written

| Path under `$HOME` | Who reads it |
|---|---|
| `.agents/skills/`, `.agents/agents/`, `.agents/rules/` | Codex user skills live in `~/.agents/skills`. |
| `.grok/skills/`, `.grok/agents/`, `.grok/rules/` | Grok (`$GROK_HOME`, default `~/.grok`). Rules apply to every project. |
| `.grok/rules/AGENTS-SYMKIT.md` | Harness overlay (last pack that ships `AGENTS.md`). |
| `.claude/skills/`, `.claude/agents/`, `.claude/rules/` | Claude Code. Grok also scans `~/.claude/rules/` when Claude compatibility is on. |
| `.claude/AGENTS-SYMKIT.md`, `.claude/CLAUDE.md` | Claude global instructions. The pointer is `@AGENTS-SYMKIT.md`. |
| `.codex/skills/` | Same skills mirror a project install writes. |
| `.codex/AGENTS-SYMKIT.md`, `.codex/AGENTS.md` | Codex global instructions (`$CODEX_HOME`, default `~/.codex`). |
| `.symkit/state.yaml` | Harness, role, and the names this install manages. |

`~/AGENTS.md`, `~/AGENTS-SYMKIT.md`, and `~/.gitignore` are not written.
Package `docs/` are not copied into `~/docs`.

If `~/.codex/AGENTS.override.md` already exists, it is left alone. Codex
reads that file instead of `AGENTS.md`. The preview says so.

If `GROK_HOME` or `CODEX_HOME` points somewhere other than `~/.grok` or
`~/.codex` under this home, symkit warns and still writes the default
dot-directories. It does not follow those variables to a second root.

## What is refused

- `symkit init --root`
- `--root` together with a target directory
- `--scaffold`, `--docs`, `--docs-root`, `--migration-docs` (project trees)
- `--root` when `HOME` is unset (`USERPROFILE` on Windows)
- Installing when the home directory is the symkit repo

## Prune

A project install removes role leftovers and library skills the role did
not select. `--root` only removes names recorded in `.symkit/state.yaml`
by an earlier root install that this run no longer includes. A personal
skill in `~/.grok/skills/` stays, even when its folder name matches a
library skill. `--no-prune` skips that step.

Re-install merges. The same skill or agent name is overwritten. The
preview names paths that already exist and were not in the previous
managed set.

Open a new Grok, Claude, or Codex session after install.

## Uninstall

```bash
symkit uninstall --root --yes
```

This removes names recorded in `~/.symkit/state.yaml` from `~/.agents`,
`~/.grok`, `~/.claude`, and `~/.codex`, and strips the harness pointer from
`~/.claude/CLAUDE.md` and `~/.codex/AGENTS.md`. A skill symkit did not
record is left in place. `AGENTS.override.md` is left alone.
