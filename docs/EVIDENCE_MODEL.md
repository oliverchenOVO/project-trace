# Evidence model

Every node has an ID, kind, epistemic status, title, summary, creator, evidence strength, structured provenance, data and validity. Every edge has its own ID, relation, epistemic status and provenance.

| State | Meaning | Treatment |
|---|---|---|
| observed | Original stack/log/test input supplied by the user | Solid thick border, OBSERVED |
| derived | Reproducible parser, filesystem or Git result | Thin solid border, DERIVED / SOURCE VERIFIED |
| correlated | Explicit heuristic connecting candidates | Amber dashed border and connection, CORRELATION |
| hypothesis | Model interpretation requiring verification | Violet dotted border and connection, AI HYPOTHESIS |

Provenance includes source type, file and line region where applicable, commit SHA where applicable, command argument array, raw excerpt, artifact ID, SHA-256 and capture time. Source hashes cover the full file; the excerpt itself is a bounded line region. User inputs have case artifact IDs. Node and edge inspectors make these references inspectable without an external editor.

`snapshot` means captured evidence retained in the case. `stale` means source bytes changed, and `invalid` means unavailable/unreadable source. An overall stale flag also detects HEAD or worktree-status differences even when the relevant source bytes are unchanged. Snapshot integrity does not prove that current source matches the deployed revision that originally produced a crash.

Derived strength VERIFIED means the recorded mechanical observation is verifiable. It does **not** mean the root cause is verified. Observed inputs have DIRECT strength; correlations and hypotheses have INDIRECT strength. No test output pasted by a user is assigned independently executed status.

Hypothesis confidence is LOW with blame alone. MEDIUM requires cited actual patch and cited pasted test output, with no cited contradiction. HIGH is intentionally unavailable without stronger causal verification. The inspector explains this rule; no invented percentages are used.

The current implementation never emits `introduced_by` or `causes`, and never displays BISECT CONFIRMED. Supported relations include occurred_at, contains, modified_by, correlates_with, supports and contradicts.
