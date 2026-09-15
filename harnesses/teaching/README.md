# Teaching harness

Course materials, staff agents, and optional learner literacy.

## Roles

| Role | Packs | Notes |
|:-----|:------|:------|
| `instructor` / `faculty` | shared + staff + instructor | Full design + evaluation toolkit |
| `ta` | shared + staff + ta | Evaluation + engagement; prunes instructor-only skills |
| `learner` | shared + learner | Student docs + study skills |
| `materials` | shared | Student-safe materials defaults only |

This harness installs **agent packs**. Course *runtime* (uv, Containerfile,
org identity) is [csymd/symcourse](https://github.com/csymd/symcourse).
The usual teaching path is `symcourse new`, which calls this installer
(pass `--symkit` if the binary is not on `PATH`).

```bash
# Nested from symcourse (preferred for a full course repo):
#   ./cli/symcourse new … --symkit /path/to/this/checkout

# Existing course tree (no --scaffold):
./cli/symkit install /path/to/course --harness teaching --role instructor --docs slos

# Kit-only folder stubs (assignments/, lectures/, data/ — not a full course image):
./cli/symkit init /path/to/new-course --harness teaching --role instructor --scaffold --docs slos
```

Do not pass `--scaffold` on a tree `symcourse` already created. Do not install
`--role instructor` and then `--role learner` on the same tree (learner prune
strips instructor agents).

`--docs slos` copies a faculty-owned SLO blank into `docs/` or `documents/`
(detected; pass `--docs-root` if both exist). It does not overwrite unless
`--force`. That file is the published outcome list (`slos-as-truth`).
Pack `docs/` (AI policy, literacy) are the same: skip existing files unless
`--force`. `symkit show teaching` lists template ids.

Do not commit `staff`, `instructor`, or `ta` packs into student-visible trees.
Learner docs under `docs/` are safe to commit; `.agents/` usually is not.
See the installed `course-materials.md` rule for the commit list.

`--migration-docs` (or the teaching `init` prompt) creates `migration-docs/`
for PDF/Word dumps. Convert with the instructor skill `migrate-course`.
Source files stay gitignored.

Course-specific extras: copy `examples/teaching-overlay/` into
`harnesses/teaching/packages/<id>/`, register it in `catalog.yaml`, then
`--also <id>`. Keep overlays short. Overlay packs are not first-class local
paths yet — they must live in the catalog.
