# Teaching shared package

Student-safe materials defaults: `AGENTS.md`, course-materials and
slos-as-truth rules, and the `materials-author` agent. `write-gherkin` is
assigned to the materials role in `catalog.yaml`. Faculty shipping checks
(`release-materials`) stay on instructor / materials roles, not the learner
pack.
Course SLO blanks live in `harnesses/teaching/templates/` and copy with
`--docs slos`. That is the published outcome file; do not keep a second
list. Course runtime/identity is [csymd/symcourse](https://github.com/csymd/symcourse).

Literacy handouts live in the **learner** package, not here.
