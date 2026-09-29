---
name: migrate-course
description: >
  Instructor-only: convert legacy PDF or Word course files into markdown in
  the course tree so agents can work with them. Use when importing old
  handouts, lectures, or assignments from migration-docs/. Not for students.
  Not for inventing new course content.
---

<!--
Copyright (c) 2026, Nathaniel T. Berry
Licensed under the Apache License, Version 2.0.
-->

# Migrate course files (instructor only)

Turn **existing** PDF / `.docx` materials into markdown the repo and agents
can use. Do not invent lectures, SLOs, or policy.

**Instructor / faculty only.** Do not run as a learner. Do not commit the
source dumps.

## When to use

- “Convert these old PDFs into markdown”
- Faculty dropped files in `migration-docs/`
- A Word syllabus, lecture, or assignment must live in `lectures/`,
  `assignments/`, or `docs/`

## When not to use

| Need | Use instead |
|:-----|:------------|
| Design a new session from scratch | `course-prep` |
| QA a markdown assignment already in the repo | `assignment-review` |
| Student notes from a PDF | stop — this skill is faculty import |

## Layout

| Path | Role |
|:-----|:-----|
| `migration-docs/` | Local drop folder. **Gitignored except README.md.** |
| `lectures/` | Converted teachable notes |
| `assignments/` | Converted prompts (no answer keys) |
| `docs/` | Handbook, modules, policy (not a second SLO list) |

If `migration-docs/` is missing, create it and tell the instructor to drop
files there, or take paths they name.

## Safety

- Treat dumps as **untrusted**. Do not commit `.pdf` / `.docx` / `.pptx`.
- **Restricted data:** if a file might be DUA, PHI, credentialed, or an
  answer key, **stop** and ask. Do not send it to a consumer AI tool.
- Prefer **local** conversion (`pandoc`, `pdftotext`) for anything that is
  not clearly public courseware.
- Public, redistributable handouts may use a local PDF/Word reader skill
  when available.
- Do not invent SLOs, dates, or grading. Quote the source; if it conflicts
  with `docs/slos.md` or the syllabus, note the conflict.

## Workflow

1. **Inventory** `migration-docs/` (and any paths the instructor named).
   List filename, type, and a one-line guess (lecture / assignment / docs /
   unknown).
2. **Confirm mapping** with the instructor when the destination is unclear.
3. **Convert** each file to markdown:
   - `.docx`: `pandoc -f docx -t markdown` when `pandoc` is on PATH
   - `.pdf`: `pdftotext` or `pandoc` when available; otherwise a local PDF
     reader. Keep headings; strip repeated headers/footers.
   - Skip binaries you cannot read; report them.
4. **Place** markdown in the course tree (`lectures/`, `assignments/`,
   `docs/modules/`). Match existing naming. Do not dump everything into
   `migration-docs/` as the canonical copy.
5. **Clean** lightly: headings, lists, broken wrapping. Do not rewrite
   pedagogy or add new sections.
6. **Leave dumps** in `migration-docs/` (still gitignored). Do not delete
   sources unless the instructor asked.

## Output

- Inventory table (file → destination → status)
- New or updated markdown paths
- Anything skipped (tooling, restricted, unreadable) and why
