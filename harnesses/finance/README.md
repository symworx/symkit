# Finance harness

Planning notes and tax work from a written record. The financial advisor
and the CPA are peers. The human chooses which persona is the session.
Both are installed either way.

```bash
./cli/symkit init /path/to/new-engagement \
  --harness finance --role financial-advisor --scaffold
```

Roles: `financial-advisor`, `cpa`, `materials`.

`financial-advisor` is the default role. The CPA persona is on disk too.
Switch personas when the question crosses from planning to a tax position,
or the other way. They do not spawn each other.

Both sessions may spawn `bookkeeper` (tie figures to source documents) and
`file-reviewer` (notes on a draft plan or draft return). The session owns
what a client would be told.

The engagement, the written plan, source documents, the books, and filed
returns are the record. These agents do not take the place of a licensed
financial advisor or a certified public accountant.
