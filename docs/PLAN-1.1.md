# Phase 1.1 plan

1. Add a separate opt-in verification service with structured executable/argument validation, approval tokens, persisted lifecycle and owned temporary worktrees. Keep the read-only Git API unchanged.
2. Capture real executions with bounded artifact output, timestamps, revision, environment and integrity hashes. Use Windows Job Objects for timeout, cancellation and application-exit process cleanup.
3. Validate good/bad endpoints and use isolated native Git bisect with individually captured test results. Recheck the boundary and retain all steps; abort inconclusive or invalid endpoints.
4. Attach native-owned verification evidence to existing cases; validate references/hashes on reopen. Recover interrupted runs without restarting execution and support validated cleanup retry.
5. Add Verification Lab approval, progress/cancel, boundary and result inspectors, and progressive graph expansion.
6. Test real repositories, failure/recovery, existing regressions, rendered workflow and native Windows IPC; package 0.2.0, document exact gate results and push private milestones.

Scope: predicate is exit 0 versus nonzero; timeout, cancellation, launch/output errors never classify a revision. No installs, inferred commands, source-map work or universal causal claim. Temporary Git administration changes are expected; the active checkout's HEAD, branch, status and nonignored file digest must match the captured state before claiming it unchanged. Arbitrary approved tests retain normal OS permissions, so this is isolation of checkout state, not a security sandbox.
