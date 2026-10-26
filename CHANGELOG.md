# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Finance harness. `financial-advisor` and `cpa` are peer personas (alias
  `fa`). Both may spawn `bookkeeper` and `file-reviewer`. Drafts do not
  take the place of a licensed financial advisor or a certified public
  accountant. `materials` prunes the personas and both leaves.

### Changed

- Harness `teaching` is now `education`. `teaching` remains a deprecated
  alias: `list` and `show` accept it, install warns, and `.symkit/state.yaml`
  records `education`. The tree is `harnesses/education/`.

## [0.3.0] - 2026-10-26

### Added

- Role field `may_spawn`. Performance `coach` (and `performance-coach`) may
  spawn `analyst`. Research `researcher` may spawn `reviewer`. Creative
  `creative-director` (and `creative`) may spawn `copywriter` and `editor`.
  Teaching `instructor` (and `faculty`) may spawn `materials-author`.
  Those four sessions also receive the `delegate` skill. Install
  refuses a missing agent file, a role that would leave that file behind,
  and a role that both spawns and prunes the name. `materials` prunes the
  role agent and its specialist.

### Changed

- Skill `repro-check` is now `reproducibility-check`. `repro-check` remains a
  deprecated alias: listing it installs `reproducibility-check` and prunes a
  leftover `repro-check` directory.
- GitHub Flow: default branch is `worx`; releases are tags on `worx` (no `stage` / `main` promotion). 

## [0.2.1] - 2026-09-17
### Added

- Teaching instructor/faculty skill `migrate-course`: convert local PDF/Word
  dumps in `migration-docs/` to markdown. `--migration-docs` creates that
  dir (TTY ask on teaching `init` unless `--yes`). Dumps are gitignored;
  `README.md` may be tracked.

### Changed

- Copyright holder updated to Nathaniel T. Berry
- Repository URLs updated from `github.com/csymd` to `github.com/symworx`
- `docs/slos.md` on a **symcourse** tree is written by the course scaffold
  (code + title filled in). `--docs slos` still copies a blank only when
  the file is missing (kit-only teaching stubs).
- Teaching docs state the split with
  [symworx/symcourse](https://github.com/symworx/symcourse): this kit installs
  agent packs (`--docs slos`); course runtime and identity are a different
  tool. Do not pass `--scaffold` on a `symcourse` tree. Do not follow an
  instructor install with a learner install on the same tree.
- Pack `docs/` merge is copy-if-missing unless `--force` (same as `--docs`
  and `--scaffold`). `.agents/` still overwrites. Teaching `--scaffold`
  includes a short `docs/ai-what-to-expect.md` stub.
- README / install / teaching harness docs lead with `symcourse new` as
  the usual teaching path (`symkit install`, no `--scaffold`).
- Teaching shared rules and `release-materials` no longer ask for
  student-facing release notes. Learner packs stay learning-focused;
  shipping checks remain faculty-only.


## [0.2.0] - 2026-09-02

### Added

- Install writes harness `AGENTS.md` to `AGENTS-SYMKIT.md` and appends a
  pointer on `AGENTS.md`. Existing `AGENTS.md` is never replaced.
- `--docs <id>` copies catalogued faculty-owned blanks into `docs/` or
  `documents/` (copy-if-missing unless `--force`). Teaching: `slos`.
  Research: `aims`, `protocol`. `symkit show` lists ids.

### Changed


## [0.1.2] - 2026-08-22

### Added

- GitHub Release assets: prebuilt `symkit` binaries for Linux (musl x86_64
  and aarch64), macOS (x86_64 and Apple Silicon), and Windows (x86_64),
  plus `SHA256SUMS` and GitHub artifact attestations.

### Changed

- Install docs and `symkit guide` lead with GitHub Release binaries; crates.io
  remains the compile-from-source path (`cargo install --locked symkit`).

## [0.1.1] - 2026-08-17

### Added

- `symkit --help` prints a short FLOW / EXAMPLES / WRITES block.
- `symkit guide` covers cargo install vs clone, adapters, and what to commit.
- Flag help on `init`, `install`, and `adapt`.
- `write-docs` library skill; assigned to engineering `engineer` / `swe`.

### Changed

- Research packs treat an existing tree as the layout source (`study-layout`,
  `repro-check`, research `AGENTS.md`).
- Teaching `course-materials` rule states what to commit; overlay example
  ships a stub rule.
- Engineering `match-repo` forbids a parallel `src/` / test tree.

---

## [0.1.0] - 2026-08-15

### Added

- Initial release.
- Provides an initial pass at key agents for education as well as engineering, 
  creative, research, and product management. 
- A long list of common skills; some unique to one agent harness and some shared 
  across multiple agent harnesses.
  
### Notes

- Pushing a an early release ... we expect bugs, but hope to keep iterating on this
  to help provide an easy framework for newcomers (and experienced users) to set up 
      new harnesses...

---

## Version Links

[0.2.0]: https://github.com/symworx/symkit/releases/tag/v0.2.0
[0.1.2]: https://github.com/symworx/symkit/releases/tag/v0.1.2
[0.1.1]: https://github.com/symworx/symkit/releases/tag/v0.1.1
[0.1.0]: https://github.com/symworx/symkit/releases/tag/v0.1.0
