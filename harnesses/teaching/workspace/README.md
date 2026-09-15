# Course workspace

Folder stubs from [symkit](https://github.com/csymd/symkit) (`teaching`
harness, `init --scaffold`). A full course image (uv, Containerfile, org
identity) comes from [csymd/symcourse](https://github.com/csymd/symcourse)
instead — do not pass `--scaffold` on that tree.

## Layout

| Path | Purpose |
|:-----|:--------|
| `docs/` | Syllabus, module cards, calendars |
| `lectures/` | Teachable / study notes |
| `assignments/` | Graded prompts (submit on the LMS) |
| `data/public/` | Tiny public or synthetic samples |
| `AGENTS.md` | Repo agent rules (installer appends a pointer) |
| `AGENTS-SYMKIT.md` | Installed harness defaults |

## Getting started

```bash
# If this repo uses uv:
#   uv sync
# Review agent install (local, gitignored):
ls -a .agents .grok 2>/dev/null || true
```

Graded work is submitted on the **LMS**, not by pushing to this repository,
unless the instructor says otherwise.
