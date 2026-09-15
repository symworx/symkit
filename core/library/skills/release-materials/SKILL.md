---
name: release-materials
description: >
  Faculty checklist before merging course materials to the student-facing
  branch. Use when checking a drop for restricted data, secrets, or answer
  keys. Not for drafting student-facing release notes.
---

<!--
Copyright (c) 2026, PalEm Dynamics LLC
Licensed under the Apache License, Version 2.0.
-->

# Release materials

Faculty safety checklist before students pull an updated `main`. Adapt to the
target repo’s CONTRIBUTING.md. Do not draft student-facing release notes.

## 1. Intent

- Confirm which changes on `develop` (or the release branch) are meant for
  students this drop.
- Note anything deferred to a later release.

## 2. Safety scan

- [ ] No credentialed or restricted data under `data/` or elsewhere
- [ ] No secrets, tokens, or local env files
- [ ] No private answer keys in student-facing assignment paths (unless
      intentional and documented)
- [ ] Public samples only; docs still point to external paths for restricted data

## 3. Consistency

- [ ] Syllabus / calendar / module cards agree on dates and deliverables if
      this release touches them
- [ ] Links and paths in README still resolve
- [ ] Container / `uv` instructions still match `pyproject.toml` /
      `Containerfile` if present

## 4. Ship

Typical flow (confirm in repo docs):

1. Land work on `develop`.
2. Merge to `main` as documented.
3. Tag a release when the course process requires it.

## Output

Produce a concise checklist result (pass/fail items). Do not write a student
announcement or changelog unless the faculty asked for one in their own words.
