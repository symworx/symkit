# Root (user-global) install — draft

**Status:** draft / not implemented. Do not rely on this path yet.

SymKit currently installs into a project/repo directory. A **root** option would install the same harness packs into user-level locations so every Codex (and other) session sees the skills/agents without a per-repo install.

## Goal

```bash
symkit install --root --harness engineering --role engineer --adapters all
# or
symkit root install --harness engineering --role engineer
```

Result: skills appear in every Codex session via `~/.agents/skills/` (and optionally `~/.codex/skills/`), with global instructions in `~/.codex/AGENTS.md`.

## Destination mapping (draft)

| Project install writes          | Root install writes (proposed)                  |
|---------------------------------|-------------------------------------------------|
| `<dir>/.agents/skills/`         | `~/.agents/skills/` (primary)                   |
| `<dir>/.agents/agents/`         | `~/.agents/agents/` (or mapped to `~/.codex/agents/`) |
| `<dir>/.agents/rules/`          | `~/.agents/rules/`                              |
| `<dir>/.codex/skills/` (adapter)| `~/.codex/skills/`                              |
| `AGENTS-SYMKIT.md` + pointer    | Merge into `~/.codex/AGENTS.md`                 |
| `<dir>/.symkit/state.yaml`      | `~/.local/share/symkit/state.yaml` (or `~/.symkit/`) |
| target `.gitignore`             | skip                                            |
| `--scaffold` / `--docs`         | disabled or warned (no project tree)            |

Respect `CODEX_HOME`, `XDG_DATA_HOME`, and existing `SYMKIT_DATA`.

## Implementation sketch

1. **CLI** (`src/main.rs`): add `--root` flag on `install` (or a `Root` subcommand). When set, target is resolved to a user home path instead of a project dir. Refuse `--scaffold` / course `--docs` by default.
2. **Install path logic** (`src/install.rs`): when root mode is active, rewrite canonical + adapter destinations from relative-to-target to absolute user paths. Reuse existing merge/prune/state code.
3. **Adapters**: same catalog entries; only the output roots change. Optionally a "root" adapter variant that emits only global pieces.
4. **State**: write outside any project so re-install can detect prior root installs and prune/switch roles.
5. **Safety**: preview always, require `--yes` for non-TTY, never touch a project `.gitignore`, refuse self-install of the kit repo.

## Open questions

- One root tree vs multiple harnesses (warn on mix, same as project)?
- Agent format: keep Markdown under `~/.agents/agents/` or convert for Codex subagents under `~/.codex/agents/`?
- Should root mode also write a Claude/Grok global equivalent, or Codex-only for v1?
- How to uninstall / reset a root install?

## Not in this draft

No functional code yet. This file is the design note only.
