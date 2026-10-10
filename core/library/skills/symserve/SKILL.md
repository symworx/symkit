# Copyright (c) 2026, Nathaniel T. Berry
# Licensed under the Apache License, Version 2.0.

---
name: symserve
description: >
  Call the symserve MCP server for biosignal generation and analysis
  (PPG, respiration, gait, RQA, sample entropy). Use when a repo
  documents symserve or when signal generation or analysis is needed.
  Triggers: symserve, SYMSERVE_URL, generate PPG, run RQA, sample
  entropy, biosignal analysis, /symserve.
---

# Symserve

Wrap the symserve server. Do not reimplement its algorithms.

## Precondition

1. Check that SYMSERVE_URL is set in the environment.
2. If it is not set, stop and report: 'symserve is not configured
   (SYMSERVE_URL is unset).' Do not invent results, recurrence rates,
   or entropy values.

## Steps

1. Generate a signal with generate_ppg, generate_respiration, or
   generate_gait. Signals export through symworx-io as CSV.
2. Analyze with run_rqa or run_sample_entropy, passing the file path.
   Both load through symworx-io — never read files directly.
3. Compare results across the two analyses when useful.

## Do not

- Call symserve tools when SYMSERVE_URL is unset.
- Reimplement RQA, sample entropy, or signal generation locally.
- Invent numeric results the server did not return.

## Output

- Tool called and parameters used
- File path of any generated signal
- Analysis results as returned (recurrence rate, determinism,
  sample entropy, etc.)
- Any server error, verbatim
