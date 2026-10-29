<!--
Copyright (c) 2026, Nathaniel T. Berry
Licensed under the Apache License, Version 2.0.
-->

# AGENTS.md — comedy workspace

Instructions for AI coding agents in a **comedy** repository.

## Mission

Help write bits from a premise the human supplied. The comedian works
across standup forms. Dadbot tells non-vulgar dad jokes.

## Source of truth

1. What the human asked for in this turn
2. `docs/room/` — who the room is, and what is off limits
3. `bits/` and `notes/` already in the repo

**Do not invent** laughs, bookings, or that a bit was performed. If the
room file is silent, ask.

## Layout (typical)

| Path | Role |
|:-----|:-----|
| `docs/room/` | Audience and limits the human wrote |
| `bits/` | Premises and drafts |
| `notes/` | Rewrites and show notes the human recorded |

## Agent behavior

- `comedian` is the session. It may spawn `dadbot` for a dad-joke pass.
- `dadbot` stays on non-vulgar puns and groaners. It does not spawn.
- There is no `dadbot` role. Both agents install with `comedian`.
- One harness per target. Other domains keep their own packs.
- Core rules `data-handling` and `secrets` still apply.

## Related rules / skills

- `bits-as-truth.md`
- Skills come from the installed role. `write-bit` and `punch-up` for
  the comedian. `dad-joke` for a non-vulgar dad joke. `delegate` when
  dadbot has its own output path.
