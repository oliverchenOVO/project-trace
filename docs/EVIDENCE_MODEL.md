# Evidence model

Every node has an ID, kind, epistemic status, title, summary, creator, evidence strength, structured provenance, data and validity. Every edge has its own ID, relation, epistemic status and provenance.

| State | Meaning | Treatment |
|---|---|---|
| observed | Original supplied input or actual native test output; provenance distinguishes them | Solid thick border, OBSERVED |
| derived | Reproducible parser, filesystem or Git result | Thin solid border, DERIVED / SOURCE VERIFIED |
| correlated | Explicit heuristic connecting candidates | Amber dashed border and connection, CORRELATION |
| hypothesis | Model interpretation requiring verification | Violet dotted border and connection, AI HYPOTHESIS |

Provenance includes source type, file and line region where applicable, commit SHA where applicable, command argument array, raw excerpt, artifact ID, SHA-256 and capture time. Source hashes cover the full file; the excerpt itself is a bounded line region. User inputs have case artifact IDs. The node inspector and chain navigation expose original references without an external editor. Edge provenance is retained in the case data; there is no separate edge inspector panel.

`snapshot` means captured evidence retained in the case. `stale` means source bytes changed, and `invalid` means unavailable/unreadable source. An overall stale flag also detects HEAD or worktree-status differences even when the relevant source bytes are unchanged. Snapshot integrity does not prove that current source matches the deployed revision that originally produced a crash.

Derived strength VERIFIED means the recorded mechanical observation is verifiable. It does **not** mean the root cause is verified. Observed inputs have DIRECT strength; correlations and hypotheses have INDIRECT strength. No test output pasted by a user is assigned independently executed status.

Hypothesis confidence is LOW with blame alone. MEDIUM requires cited actual patch and cited pasted test output, with no cited contradiction. HIGH is intentionally unavailable without stronger causal verification. The inspector explains this rule; no invented percentages are used.

The implementation never emits `introduced_by` or `causes`. Only an intact native experimental run may display BISECT CONFIRMED, scoped to the selected predicate/range. Supported relations include occurred_at, contains, modified_by, correlates_with, supports and contradicts.

## Experimental evidence (0.2.0)

`test_execution` is observed actual process output created by the native test runner, distinct from pasted `test_result`. It retains exact requested/resolved command, arguments, cwd, full commit revision, timing, exit code, stdout/stderr artifacts and hashes, environment policy, cancellation/timeout/error status and structured integrity digest. No launch-failure/timeout/cancellation/output-limit result has a good/bad classification.

`verification_run` and `bisect_run` are native-derived lifecycle/algorithm records. Every tested step references a retained actual execution ID and matching revision/classification. A valid `regression_boundary` requires passing good and failing bad endpoints, completed Git narrowing, failing first-bad repeat and passing parent repeat. Missing or modified execution/graph/artifact evidence invalidates its display. No AI-generated JSON can supply these node kinds or deterministic graph edges.

Historical verification remains scoped to the executed revision range, even after current repository state changes. A matching suspect can show BISECT CONFIRMED, while universal incident root cause remains unverified. Raw artifacts and run hashes detect local corruption; they are not signatures against a user who rewrites the entire database and recomputes hashes.
