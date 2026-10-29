<!--
Copyright (c) 2026, Nathaniel T. Berry
Licensed under the Apache License, Version 2.0.
-->

# AGENTS.md — legal workspace

Instructions for AI coding agents in a **legal** repository.

## Mission

Help a licensed attorney work from the written matter. The lawyer owns
the legal position. A paralegal session organizes and drafts. The law
clerk indexes. These drafts do not take the place of a licensed attorney.

## Source of truth

1. The engagement and its scope
2. `docs/matter/` — facts, issues, and theory the lawyer has written
3. `record/` and authorities the repo actually holds
4. The licensed attorney who signs the work

**Do not invent** facts, dates, parties, deadlines, holdings, or that a
filing happened. If the record is silent, ask.

## Layout (typical)

| Path | Role |
|:-----|:-----|
| `docs/engagement/` | Who the work is for, and what was agreed |
| `docs/matter/` | Facts, issues, and written theory |
| `record/` | Index and copies the human chooses to keep |
| `drafts/` | Draft work product |
| `data/public/` | Tiny public or synthetic samples |

## Agent behavior

- The `lawyer` role installs `lawyer`, `paralegal`, and `law-clerk`.
  That session may spawn the paralegal and the clerk. It owns any
  legal position.
- The `paralegal` role installs `paralegal` and `law-clerk` and removes
  `lawyer`. That session may spawn the clerk. It does not spawn a lawyer.
- There is no `law-clerk` role. The clerk does not advise and does not spawn.
- A child stays on its brief. Do not ask the paralegal to spawn the clerk.
- Client identifiers and privileged documents stay out of git.
- Public or synthetic samples only under `data/public/`.

## Related rules / skills

- `matter-as-truth.md`, `not-counsel.md`
- Skills come from the installed role. Use `delegate` when a specialist
  has its own output path. `authority-check` for a legal proposition.
  `check-citations` only for non-legal literature the human supplied.
