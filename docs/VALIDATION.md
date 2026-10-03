# TRACE 0.1.0 acceptance record

Validated on Windows x64, 2026-10-04 (Asia/Taipei), in a path containing Chinese characters and spaces. Runtime implementation revision: `0f8948614ccd28236577e44598d16bd0b62ebd37`. Later delivery commits contain documentation, screenshots and a process-restart QA script only.

## Phase status

**Phase 0: passed.** `node scripts/phase-gate.mjs` opened a real Git fixture through the Rust core, resolved `src/parser.ts:3:16`, obtained the expected blame SHA and actual removed guard patch, and built 14 nodes / 14 edges with original provenance. It asserted that pasted test output is not independently executed and that no `causes` edge exists.

**Required Phase 1 showcase: passed.** The production frontend and packaged Tauri executable exercised repository path entry, actual Node stack ingestion, exact source inspection, blame, commit, patch, provenance, failing-test ingestion, context preview, local model inference, hypothesis inspection and local case reopening. A separate complete process shutdown/restart verified that source snapshots and the AI hypothesis survive; focus-path and frame expansion also worked. Windows executable launch succeeded. Optional test execution, bisect and static symbol mapping remain deferred.

## Tests

| Check | Final result | Evidence / scope |
|---|---|---|
| Rust tests | 41 passed, 0 failed | 29 core; 4 AI boundaries; 3 executable resolution; 1 Windows file URL; 4 status parser |
| Frontend Vitest | 2 passed, 0 failed | Explicit epistemic labels and blame attribution wording |
| Playwright golden path + invalid repository | 2 passed, 0 failed | Latest final frontend; real Rust core, Git and SQLite; 33.8 seconds including setup |
| Playwright actual local inference | 1 passed, 0 failed | Separate serial run; `qwen2.5:1.5b`; 40.5 seconds including setup |
| Native WebView2 smoke | Passed | Final runtime; real Tauri IPC, actual Git patch and local Ollama; no page errors |
| Complete process restart | Passed | Reopened source + hypothesis; focus and frame expansion; no page errors |
| TypeScript / Vite production build | Passed | 1,750 modules; final JS `index-BDuqg-QS.js` |
| npm dependency audit | 0 vulnerabilities | Full dependency audit after Vitest update; final production audit also clean |

There are **46 automated test cases** across Rust, Vitest and Playwright. Gate scripts and native smoke/restart validations are additional acceptance checks, not counted as unit tests. The three latest browser tests were run in two serial invocations, rather than claimed as one combined run.

The browser plugin was unavailable; QA used Playwright Chromium and real WebView2 CDP. The browser harness injects a test-only transport into the production bundle and calls the same Rust core, without mocked Git or model output. Native smoke bypasses that adapter and exercises actual desktop commands. Software rendering was enabled for screenshot reliability on this Windows host.

## Real fixture

The fixture generator created actual commits, ran the safe test successfully (exit 0), ran the regression test (exit 1), and captured actual crashing Node stderr. The app ingests the resulting test output as user-supplied input.

| Fixture revision | SHA |
|---|---|
| Safe behavior | `23d85cd1d2debbda6c0e433479657f8b662fcb28` |
| Removed guard / regression | `e2a7cc531eac0fdd07eebfce3fa9132c63f8da00` |
| Unrelated documentation / HEAD | `360502011967139708dbaa3aead66b8cc74260e7` |

The crash-line blame identifies the regression commit, rather than the later documentation commit. Internal Node runtime frames remain unresolved with explicit notices. Fixture files are development artifacts excluded from this project's Git history; `npm run fixture` recreates them in a new folder.

## Rendered checks

| Surface | Result |
|---|---|
| Empty state | Clear repository-first flow, no invented graph data |
| Desktop source inspector | Actual line and column; raw patch and full SHA inspectable |
| AI inspector | Violet hypothesis; cited references, uncertainty, verification steps; LOW confidence and root cause unverified |
| 1024 × 768 | Graph bounds assertion passed after resizing; inspector visible |
| 768 × 900 browser preview | Rail remains visible; inspector hidden by narrow-preview layout; native desktop minimum is 1024 pixels |
| Console / native page errors | None in final checked golden path and native flows |

Actual final screenshots: [reopened desktop showcase](design/trace-showcase.png), [1024-pixel workspace](design/trace-1024.png). The generated [design concept](design/trace-concept.png) was used as visual direction; screenshots above are the implemented application.

## Failed checks and recovery

Earlier responsive graph assertions failed because React Flow measurements were not applied to the controlled presentation state. The final implementation lets React Flow own measurement state while preserving immutable evidence; the latest bounds assertion passed. Earlier browser screenshot attempts and overloaded development-server runs stalled; final QA used the production bundle and software rendering.

Actual `qwen2.5:7b` inference exceeded the 120-second request deadline on this host. Its success check did not pass. TRACE displayed an explicit timeout, retained deterministic evidence and created no hypothesis. Final actual inference acceptance used the smaller installed `qwen2.5:1.5b` model. Model text sometimes misinterpreted provided evidence, including claiming a supplied line was missing; citation validation guarantees real references and permitted structure, **not semantic correctness**.

One final NSIS packaging attempt failed with Windows file-in-use error after the test executable was started before bundling had finished. The test process was closed before retrying packaging. The final packaging result and hashes are recorded below.

## Limits and next step

Forensics is targeted to the first resolved source location; at most eight frames receive source excerpts. Source maps and static AST symbol resolution are absent. Log input is retained as evidence without a chronological timeline. No repository test runner, bisect, or causal verification is implemented. Git observations and captured source may differ from the deployed crash revision. The local case database is unencrypted. Model availability and accuracy vary. Edge provenance is stored, but there is no dedicated edge inspector.

The installer is unsigned. Installation on a clean Windows machine has not been tested; executable launch and native workflow were tested on this development machine with installed WebView2 and Git. Next recommended work is an explicit test command in an isolated worktree, followed by regression bisect and signed clean-machine distribution validation.

## Windows artifacts

Final `npm run tauri -- build` completed successfully with one NSIS x64 bundle. The packaged executable was launched again after bundling and passed the process-restart source/hypothesis checks.

| Artifact (relative to project root) | Bytes | SHA-256 |
|---|---:|---|
| `src-tauri/target/release/TRACE.exe` | 15,318,528 | `7BE93E191F312A3287DDC2A64219CA1F85218C3CC16BD7AFA1F26625BE17555A` |
| `src-tauri/target/release/bundle/nsis/TRACE_0.1.0_x64-setup.exe` | 6,003,504 | `BEF7CDE5D0802F539ED6FD72E37FC1B28195EF32DD23D3AFAD0ADED488A51619` |

Build artifacts are available locally and ignored by Git. Repository: `https://github.com/oliverchenOVO/project-trace`; privacy was verified with `gh repo view --json isPrivate` returning `true`. Runtime source revision is recorded at the top; use `git rev-parse HEAD` for the later documentation delivery revision.
