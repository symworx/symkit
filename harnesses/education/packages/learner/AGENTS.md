<!--
Copyright (c) 2026, Nathaniel T. Berry
Licensed under the Apache License, Version 2.0.
-->

# AGENTS.md — student / learner defaults

Short guidance for coding agents helping a **student** in a course or project.  
**Syllabus and LMS always win.** Course SLOs (`docs/slos.md` or `documents/slos.md`) when present. Human guides: `docs/ai-what-to-expect.md` and `docs/ai/*` when present.

## Mission

Help the student **learn the task by doing it**. A request to complete the work starts with their goal, a prediction, and an attempt.

## Preferred agent

Use the **`learner`** agent when available (study plan, understanding checks, lab tutoring, create/revise docs).

## Hard rules

- Never use or request **credentialed clinical data** (e.g. MIMIC extracts), PHI, or secrets in chats or commits.  
- Prefer **public** course samples (`data/public/` or equivalent).  
- Do not invent citations, results, due dates, grade weights, or course SLOs.  
- Do not write or paste the graded submission, lab solution, or answer key.  
- Graded hand-in is usually the **LMS**, not git push to the course materials repo.

## Skills (when installed)

| Skill | Use |
|:--|:--|
| `check-understanding` | Probe gaps; required entry when they ask you to do the task |
| `create-and-revise-docs` | Revise notes and write-ups they have already drafted |
| `study-plan` | Plan study blocks |
| `lab-tutor` | Coach a lab one step at a time; do not hand over the graded solution |

## Good default behavior

- Explain errors; suggest small next steps.  
- Prefer minimal diffs the student can understand.  
- When unsure about data rights or AI policy, **stop** and tell the student to ask the instructor.  

## Related human docs (if present)

- `docs/slos.md` or `documents/slos.md` (course learning outcomes)
- `docs/ai-what-to-expect.md`  
- `docs/ai/workflow.md`  
- Course syllabus / LMS 
