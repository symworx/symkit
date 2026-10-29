# Authoring a harness

A harness is a named set of packages plus an optional workspace scaffold,
registered in `catalog.yaml`.

## Package layout

```text
harnesses/<name>/
  README.md
  packages/<pkg>/
    AGENTS.md                 # copied to target AGENTS-SYMKIT.md (last pack wins)
    .agents/rules/*.md
    .agents/agents/*.md
    docs/                     # merged into target docs/
  templates/                  # faculty-owned blanks; copied by --docs <id>
  workspace/                  # copied only by `symkit init --scaffold`
```

## Register

Add the harness under `harnesses:` in `catalog.yaml`:

- `status: active` (or `later` to list but refuse install)
- `packages` with `path` and `student_safe`
- `roles` mapping role → `packages:` plus `skills:` (library names)
- `prune` for leftover agents/rules when switching roles (skills are derived).
  If a role package ships extra specialist agents, list those filenames too.
- `workspace` path if you have a scaffold
- optional `templates:` map (`id: { path, dest }`) for `--docs <id>` blanks.
  Education ships `slos`; research ships `aims` and `protocol`. Product,
  creative, and performance already scaffold their truth dirs
  (`docs/prd/`, `docs/brand/`, `docs/program/`). Finance scaffolds
  `docs/engagement/`, `docs/plan/`, `books/`, and `returns/`. Legal
  scaffolds `docs/engagement/`, `docs/matter/`, `record/`, and `drafts/`.
  Comedy scaffolds `docs/room/`, `bits/`, and `notes/`.
  AI uses `config.yaml`.
  Engineering follows existing code — do not add a parallel spec template.

## Check

```bash
./cli/symkit show <name>
./cli/symkit init /tmp/symkit-try --harness <name> --role <role> --scaffold --yes
```

See `core/templates/new-harness.md` and `examples/education-overlay/`.
The installer is Rust (`src/`); you do not register packs in code.

Skill **bodies** live in `core/library/skills/<name>/SKILL.md`. Who receives
them is `catalog.yaml`: `core.always_skills` (every install) plus each
role’s `skills:` list. Do not copy a skill into more than one package.
A second name for one body is `library.skill_aliases`. `repro-check` points
at `reproducibility-check` and is deprecated. List the target on roles. The
installer accepts the old name, warns, installs the target directory, and
prunes a directory left under the alias name.

A second name for one harness is `harness_aliases`. `teaching` points at
`education` and is deprecated. Commands accept the old name, warn on
install, and record `education` in `.symkit/state.yaml`. Do not keep a
`harnesses/<alias>/` directory beside the canonical tree.

A role package may ship extra `.agents/agents/*.md` specialists (copywriter
under creative-director, for example). The role agent is the session;
specialists are optional children, not a required pipeline. Do not add a
nested supervisor — vendor tools only let the root session spawn. Keep
spawn notes short so the session can still do the work itself.

When a role may spawn a specialist, set `may_spawn: [analyst]` on that role
(performance `coach` does this). Each name is the agent filename without
`.md`, and the file lives in a package that role installs. Every other role
that does not install that package lists the name in `prune.agents`. The
installer refuses a missing file, a role that would leave the file behind,
and a role that both spawns and prunes the same name. `--pack` does not
apply `may_spawn`.

## Overlays vs new harnesses

- **Overlay:** extra package on an existing harness (`--pack` or `--also`).
  Use for one course or lab’s rules.
- **New harness:** different domain (layout, roles, skills). Use when teaching
  conventions would be the wrong default.
