# Verification engine · 0.2.0

TRACE measures a selected test predicate over an approved revision range. It does not convert blame or model interpretation into experimental proof.

## Intentional execution

Open an existing case and select **Verify hypothesis**. Enter an executable, a JSON array of arguments, a relative working directory and a timeout. Review the resolved executable, exact argument array, revision/range, isolated location and normal OS permissions, then select **Approve and run** or **Approve and bisect**. Preparing a plan does not start a process or create a Git worktree. Approval is a single-use native token tied to that exact immutable plan; source state is rechecked before accepting it.

Commands never come from README files, stack traces, pasted test output, comments or AI instructions. Direct executable invocation is the default. Shell interpreters and arbitrary batch files are rejected. Windows npm.cmd, pnpm.cmd and yarn.cmd are mapped to their installed JavaScript CLI plus a trusted absolute node.exe. Unsupported launcher layouts report an error; they never fall back to cmd.exe. An approved package-manager test can itself invoke its normal script shell. This is disclosed execution of the selected test, not a security sandbox.

No dependency preparation is automatic. If a clean revision needs dependencies, the actual process output is retained. Failed good/bad preconditions abort bisect. There is no automatic npm install, pip install or similar operation.

## Isolation and Git boundaries

Verification has a separate native service; the original read-only Git API still rejects worktree/bisect mutation requests. Service-owned operations create detached worktrees beneath TRACE's local application data. Each run has a UUID directory, repository/case ownership marker, test worktree, bisect coordinator, disabled-hooks directory and retained artifacts. Storage inside the active source repository is rejected. Git hooks, configured clean/smudge/process filters and recursive submodule operations are disabled for administration. Submodule repositories are explicitly unsupported in this milestone.

Native Git bisect runs in the owned coordinator. Each test runs in a separately rebuilt detached test worktree at the candidate's exact full commit SHA. Recreating it between evaluations prevents leftover generated/untracked files from contaminating later trials. The active checkout is never reset, checked out, cleaned or bisected. Git's worktree registration metadata necessarily changes temporarily in the shared Git directory; that is distinct from active checkout contents.

The source state records HEAD, branch, porcelain status, a digest of all nonignored tracked/untracked files and staged diff, and a file count. Symlinks are recorded by target path without following them outside the checkout. Before/after evidence is retained. External changes produce **SOURCE REPOSITORY CHANGED DURING VERIFICATION**; TRACE does not attribute them automatically or claim the checkout was preserved. Ignored files, outside symlink targets and arbitrary external filesystem effects are outside this digest and the isolation guarantee.

## Experimental model

The predicate is exit 0 = GOOD, nonzero = BAD. Cancellation, timeout, launch failure and output-limit errors have no GOOD/BAD classification. A test crash with a nonzero process exit is BAD for this predicate, with original stderr retained; this does not establish a semantic explanation.

Bisect validates two distinct full commit SHAs and ancestry. It actually runs known bad (must fail) and known good (must pass), then executes Git-selected candidates and records every decision. The first-bad result must be a real full SHA from the coordinator's final log. TRACE repeats the test at that revision and its single parent before confirming the boundary. Merge/root boundaries are reported inconclusive in this milestone. Git bisect assumes the predicate is monotonic within the range; flaky or nonmonotonic tests can violate that assumption. Endpoint and immediate-boundary repetitions are useful checks, not a universal flakiness detector.

The UI says **BISECT CONFIRMED** / **REGRESSION BOUNDARY VERIFIED**, scoped to the selected test and approved good/bad range. It continues to say **Root cause unverified**. A matching line-attribution suspect receives a scoped confirmation in its inspector; a different first-bad result remains visible as different evidence. Model hypothesis confidence is not automatically promoted to universal causal certainty.

## Capture, limits and integrity

Each execution records requested and resolved executable/arguments, exact cwd and revision, start/end times, duration, exit code, separate stdout/stderr, timeout/cancel/error status and safe environment metadata. No full environment dump is persisted. Git overrides, NODE_OPTIONS and package-manager shell override variables are removed. Normal network and filesystem permissions remain inherited.

Windows processes are created suspended, assigned to a kill-on-close Job Object, then resumed. This prevents an early-spawning child from escaping assignment. Timeout, cancellation, normal completion and app termination close the job and terminate descendants. The runner drains both streams concurrently, saves at most 16 MiB per stream, and shows a 32 KB preview per stream. Larger view output remains in the bounded artifact. Exceeding the artifact cap terminates the experiment as inconclusive, rather than classifying a truncated result as BAD.

SHA-256 covers each retained raw artifact, the structured execution and the run record. Reopening checks hashes and graph-to-run references; missing or modified test evidence invalidates the boundary display. These are integrity checks, not cryptographic authentication against an attacker able to rewrite the entire local database and recompute hashes. Cases and output artifacts are local and unencrypted; they may contain confidential test output and are not pushed to GitHub.

Cases and verification records are written together under an immediate SQLite writer transaction. A model response based on an older snapshot merges only its new hypothesis nodes and edges into the latest case, preserving concurrent experiment evidence. Polling reads a consistent bundle so an incremental update cannot appear as a missing-evidence failure. Confirmation is displayed only after cleanup and source state comparison complete.

## Lifecycle, cancellation and recovery

Runs move through awaiting approval → running → cleaning up → completed outcome. Completion includes bad confirmed, good confirmed, bisect confirmed, inconclusive, failed or cancelled. Completed steps remain available on cancellation. Git administration has its own 60-second deadline; the selected test timeout applies to each actual test.

Cleanup resets the coordinator and removes registered owned worktrees through Git. It never runs git clean against the source checkout. Ownership UUID, canonical containment, marker data, symlink status and Git common directory are validated. Failed cleanup reports the exact path and persists **WORKTREE CLEANUP REQUIRED**. Explicit cleanup retry may unlock an owned worktree. Only an explicitly retried, validated TRACE-owned unregistered partial-creation directory may be removed directly; output artifacts are retained.

On startup, running/cleaning runs become **INTERRUPTED VERIFICATION**. No command resumes automatically. Opening that case exposes the path and cleanup/discard controls. Discard retains historical evidence and requires successful owned cleanup. Artifacts and small run directories intentionally remain for provenance after the actual Git worktrees are removed.

## Acceptance

The existing three-commit showcase uses npm.cmd with argument array `["test"]`; it requires no dependency installation. Use its manifest's safe commit as known good and HEAD as known bad. The expected boundary is the actual guard-removal commit. See [validation](VALIDATION.md) for executed gate results, artifact hashes and failed checks; implementation alone is not evidence that a gate passed.
