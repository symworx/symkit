# Performance harness

Combined **exercise physiology** and **biomechanics** for human-performance
repos (training, testing, movement analysis).

```bash
./cli/symkit init /path/to/new-program \
  --harness performance --role coach --scaffold
```

Roles: `coach` (`performance-coach`), `materials`.

The harness is the domain (`performance`). The persona is `coach` — not a
life coach, coding coach, or clinician. The coach package also ships
`analyst`. The coach session may spawn it for an analysis pass, or do that
work itself.

Written program and recorded trials are the source of truth. If the repo
documents [SymWorx](https://github.com/symworx/symworx) or another kernel,
wrap it — do not reimplement filters or inverse dynamics.
