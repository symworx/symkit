# Legal harness

Matter work from a written record. The lawyer is the default session.
The paralegal is a separate role. The law clerk is a leaf, not a role.

```bash
./cli/symkit init /path/to/new-matter \
  --harness legal --role lawyer --scaffold
```

Roles: `lawyer`, `paralegal`, `materials`.

`lawyer` installs `lawyer`, `paralegal`, and `law-clerk`. That session
may spawn the paralegal and the clerk. The lawyer owns the legal
position and anything a client or a court would be told.

`paralegal` installs `paralegal` and `law-clerk`, and removes `lawyer`.
That session may spawn the clerk. It does not install a lawyer to spawn.
Switch to it when the session should be support work.

There is no `law-clerk` role. Either session may spawn the clerk to
index the record. The clerk returns the index. The session owns what ships.

The engagement, `docs/matter/`, the record, and authorities the repo
holds are the record. These agents do not take the place of a licensed
attorney.
