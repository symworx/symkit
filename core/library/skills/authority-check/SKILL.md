---
name: authority-check
description: >
  Check legal propositions in a draft against authorities the repo
  holds. Never invent a case, statute, pin cite, or holding. Triggers:
  authority check, legal citations, authority-check, /authority-check.
---

<!--
Copyright (c) 2026, Nathaniel T. Berry
Licensed under the Apache License, Version 2.0.
-->

# Authority check

Legal propositions against the authorities on disk. Not citation style,
and not a rewrite.

## Hard rules

- **Never invent** a case name, reporter, pin cite, statute section, or holding.
- If the repo does not contain the authority, mark **UNVERIFIED**.
- A cite that does not support the sentence is a **MISMATCH**.

## Steps

1. Read the draft and the authority files the draft cites.
2. For each legal proposition, record the authority it points at, or that it has none.
3. Where you can read the authority, say whether it supports that sentence.
4. Leave the draft unchanged.

## Do not

- Pad the draft with authorities you remember.
- Use this skill for non-legal literature. That is `check-citations`.
- Mark the draft ready to file.

## Output

```markdown
## Scope
Draft and authority files read. What you could not open.

## Blockers
1. Proposition / location — issue — what is missing

## Unverified
Cites that need the human's copy of the authority.

## Summary
| Location | Proposition | Status | Note |
|----------|-------------|--------|------|
| … | … | SUPPORTED / MISSING / MISMATCH / UNVERIFIED | … |

## Verdict
**PASS** | **PASS WITH GAPS** | **FAIL** — one sentence.
```
