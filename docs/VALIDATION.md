# TRACE 0.2.0 · Phase 1.1 acceptance record

Validated on Windows x64, 2026-10-04 (Asia/Taipei), in `F:\CodeX開發小東東\AI Bug Crime Scene`. **Phase 1.1 passed all 23 required gates.** Packaged runtime source: `adfe3db3e95af7d4d0123c6adfac3976241f5170`; the subsequent delivery changes only documentation and the native QA script, not application code. Node 24.13.0, Rust 1.99.0 MSVC, Git 2.54.0.windows.1, installed WebView2 and local Ollama were used.

## Acceptance gates

| # | Gate | Result and executed evidence |
|---:|---|---|
| 1 | Existing Phase 0/1 regressions | PASS — All original 41 Rust + 2 Vitest + 3 Playwright tests rerun and passed; Phase 0 gate rerun. |
| 2 | Explicit command approval | PASS — Browser and native approval controls; native plan displays resolved program, JSON arguments, revision, location and permissions. Rust token replay rejected. |
| 3 | Actual test execution | PASS — Packaged native npm.cmd ["test"] → installed node.exe/npm-cli.js; no mocked process. |
| 4 | Captured deterministic output | PASS — Original stdout/stderr artifacts, exact command/cwd/revision/timing/exit/environment plus hashes retained. |
| 5 | Active checkout unchanged | PASS — Before/after HEAD, branch, status and nonignored file/index digest equal; dirty evidence/ folder preserved. |
| 6 | Isolated worktree | PASS — Owned detached coordinator and rebuilt per-revision test worktrees outside source checkout. |
| 7 | Known good passes | PASS — Actual fixture safe revision exit 0 in standalone run, bisect preflight and parent confirmation. |
| 8 | Known bad fails | PASS — Actual fixture HEAD exit 1 in standalone run and bisect preflight. |
| 9 | Actual Git bisect | PASS — 32 retained native Git administration executions in confirmed run; real coordinator bisect. |
| 10 | Correct first-bad SHA | PASS — Guard-removal regression SHA found, repeated FAIL and immediate parent PASS. |
| 11 | Per-step evidence | PASS — Five real executions in bisect run, with matching SHA, execution ID, role and classification. |
| 12 | Graph verification state | PASS — Native screenshot: compact hypothesis/commit → run → good/bad → bisect → verified boundary; expandable steps. |
| 13 | Inspector revision | PASS — Native boundary inspector and matching suspect show actual full first-bad SHA. |
| 14 | Scoped confirmation badge | PASS — BISECT CONFIRMED / REGRESSION BOUNDARY VERIFIED asserted in browser and packaged desktop. |
| 15 | No universal root-cause claim | PASS — Selected test predicate/range explanation, Root cause unverified retained, no causes edges. |
| 16 | Cancellation | PASS — Rust, browser and packaged native UI cancel actual running processes, retain status/evidence and clean worktree. |
| 17 | Timeout | PASS — Rust, browser and native UI timeout is inconclusive; no good/bad classification; Windows descendants terminated. |
| 18 | Cleanup succeeds | PASS — All showcase worktrees removed; final git worktree list contains only active source checkout. |
| 19 | Interrupted/failed cleanup visible | PASS — Real native app termination produces INTERRUPTED VERIFICATION; locked-worktree test exposes required path and retry succeeds. |
| 20 | Persistence across process restart | PASS — Complete native process stop/relaunch retains same five execution IDs and boundary, source and AI; no automatic rerun. |
| 21 | Windows production package | PASS — Tauri optimized build and NSIS x64 bundle succeeded before application launch. |
| 22 | Native desktop verification | PASS — Actual WebView2/Tauri IPC saved-AI-case golden path, restart, forced-termination recovery, timeout and cancel all passed. |
| 23 | Private GitHub updated | PASS — Source pushed to existing private oliverchenOVO/project-trace; final documentation/QA delivery follows this record. |

## Automated regression results

| Suite | Passed | Execution |
|---|---:|---|
| Rust original suites | 41 | Core 29; AI boundaries 4; installed Git 3; file URL 1; status 4 |
| Rust verification | 37 | Actual temporary Git repositories and Windows processes; final full run 147.13 seconds for this suite |
| Vitest | 5 | Original epistemic tests plus structured argument/status/boundary tests |
| Playwright | 5 | One complete serial run, 1.7 minutes, including real qwen2.5:1.5b inference; zero skips/failures |

**88 automated cases passed, zero final failures.** Original 46 cases remain covered. Additional acceptance scripts are not counted as unit tests. Final native test invocation: `cargo test --offline --manifest-path src-tauri/Cargo.toml --no-default-features -- --test-threads=4`. The package test:native script uses four test threads to bound concurrent real Git/process fixtures. TypeScript/Vite production build passed (1,753 modules, `index-A8qrlhV_.js`). Production dependency audit: zero vulnerabilities. `node scripts/phase-gate.mjs` also passed with the original 14 nodes / 14 edges.

The Browser plugin was unavailable. QA used Playwright Chromium for the production-bundle adapter and Playwright WebView2 CDP for the actual packaged app. The browser adapter keeps one persistent native service and never fabricates Git, process or model results. The native desktop validations directly use Tauri IPC, without that adapter. CDP/software rendering was enabled only by QA launch environment; the ordinary packaged app does not expose this debug endpoint by default.

## Actual native golden path and retained proof

Native commands executed through UI approval: `npm.cmd` with `["test"]`, resolved to installed `C:\Program Files\nodejs\node.exe` plus `npm-cli.js`. Timeout: 60 seconds per test, working directory: owned worktree root. No installation command was run.

Case: `db32e9e1-694c-440a-9457-1453a5ccd1b4`. Confirmed run: `e1cfc220-d823-4e40-ade9-88e46b84e8b7`. Completed graph: **27 nodes / 37 edges**. The workflow began by reopening an actual saved case with source, blame, commit, patch and a real local AI hypothesis. Standalone bad and good runs preceded the fresh bisect preflight.

| Role | Actual revision | Classification |
|---|---|---|
| known_bad | `360502011967139708dbaa3aead66b8cc74260e7` | BAD |
| known_good | `23d85cd1d2debbda6c0e433479657f8b662fcb28` | GOOD |
| bisect_candidate | `e2a7cc531eac0fdd07eebfce3fa9132c63f8da00` | BAD |
| boundary_bad_confirmation | `e2a7cc531eac0fdd07eebfce3fa9132c63f8da00` | BAD |
| boundary_parent_confirmation | `23d85cd1d2debbda6c0e433479657f8b662fcb28` | GOOD |

First bad: **`e2a7cc531eac0fdd07eebfce3fa9132c63f8da00`**, matching the actual line-attribution suspect. This establishes the selected test's regression boundary within the selected range; it does not establish universal production root cause. All five executions and 32 Git administration records retain actual output and integrity hashes.

Active checkout before/after: branch `main`, HEAD `360502011967139708dbaa3aead66b8cc74260e7`, original dirty status `? evidence/`, eight nonignored files. Both file/index digests are **`ac4a647dbc0697ccc7790bb96a863969553f1350919482d2e83727338fd97b3f`**. Cleanup: **cleaned**. Temporary worktrees disappeared; retained raw artifacts remain under the run-owned application data directory. Registration metadata temporarily changes in the shared Git directory, while active checkout contents remain untouched. Ignored/external filesystem effects are outside this check.

Native process restart retained exactly the same five execution IDs and first-bad result, source snapshot and AI hypothesis; no tests reran. The original native reopen script also passed focus/expand behavior without page errors.

For actual crash recovery, an explicitly approved Node test spawned a ticking child. Only the TRACE main process was terminated (no taskkill /T), then a new process was launched with a fresh WebView2 profile and the same case database. The child's tick file stopped growing when the Job Object closed. The UI showed INTERRUPTED VERIFICATION and the exact owned path, with no automatic execution. UI cleanup removed the worktree; discard retained the historical run. That experiment had no completed test result, which was honestly shown as zero completed executions and source state not yet checked.

Packaged native UI timeout and cancellation were also executed. They produced TIMEOUT/INCONCLUSIVE and CANCELLED, respectively, with error classification instead of BAD, unchanged source state and cleaned worktrees.

## Rendered QA

| Surface | Check |
|---|---|
| Approval panel | Exact resolved program/arguments, revision, path, timeout and inherited permissions visible; intentional Approve and run control |
| Verification Lab | Compact fields; good/bad flow; readable scoped result; clean/unchanged statuses |
| Default graph | Compact latest verification path; individual trials hidden until Expand steps; full investigation and focused source path work |
| Boundary inspector | Full revision range, test predicate, intact result, before/after digest and cleanup evidence |
| Execution inspector | Actual command, cwd, revision, duration, exit and raw stdout/stderr previews with artifact paths/hashes |
| Interrupted state | Explicit interrupted warning, owned path, no completed-test fiction, cleanup/discard controls |
| Responsive original UI | Browser golden path retained 1024×768 fitting and 768-wide case/inspector visibility checks |

Screenshots were visually inspected and kept outside the repository in `%TEMP%`: `trace-native-approval.png`, `trace-native-verification-lab.png`, `trace-native-boundary.png`, `trace-native-verification-reopened.png`, `trace-native-interrupted.png`, `trace-verification-lab.png` and `trace-bisect-boundary.png`. Native proof export: `%TEMP%\trace-native-verification-proof.json`; interruption ownership export: `%TEMP%\trace-native-interruption.json`. Actual raw outputs live with the local case artifacts, not GitHub.

## Failed checks diagnosed and resolved

Early QA found a 20-second Git administration deadline inadequate during concurrent builds, premature UI success before cleanup, oversized textarea layout, an unreadable full-history graph, and startup conversion of interrupted status back into cleanup status. They were corrected to a separate 60-second Git deadline, explicit cleanup stage, bounded fields, compact verification path, and completed recovery timestamps. Checkout filter configuration was disabled for both inspection and administration after an actual required-filter fixture failed. Output/provenance and transactional case/run audits were strengthened, including a delayed AI response merge test.

An unrestricted parallel Rust run failed because a 700 ms timeout expired before the descendant wrote its readiness file, three fixtures exceeded the test helper's 60-second total wait, and the new merge test expected a nonexistent status string. The descendant test now permits 5 seconds to start, fixture waits allow 180 seconds, test concurrency is four, and the assertion uses the actual bad_confirmed state. The final full suite passed. These adjustments do not weaken actual process timeout/cancel semantics.

A first additional native-controls QA attempt could not click the saved case because the preceding recovery mode intentionally left the modal open. The script now closes a visible Verification Lab before beginning the next mode. The subsequent actual timeout/cancel checks passed. No application behavior was changed for this harness fix.

## Windows artifacts

`npm run tauri -- build` succeeded: optimized compile 3m 46s and one NSIS x64 bundle. The app was launched only after bundling completed. Installer execution/clean-machine installation was not performed.

| Local artifact | Bytes | SHA-256 |
|---|---:|---|
| `F:\CodeX開發小東東\AI Bug Crime Scene\src-tauri\target\release\TRACE.exe` | 16,915,456 | `3438E6E7C534B56B14A34B318797ECC075FBB6828A40BC53B52B11828777C82C` |
| `F:\CodeX開發小東東\AI Bug Crime Scene\src-tauri\target\release\bundle\nsis\TRACE_0.2.0_x64-setup.exe` | 7,083,446 | `8C218848C8ADDD9870159B2658DC164E72C9CF9E06F9B0AB32D3E943B26DBCF6` |

Artifacts are local and ignored by Git. The existing GitHub repository remains private (`gh repo view --json isPrivate` returned true). Runtime source and subsequent documentation/QA commits are pushed to main; the completion report records the final delivery SHA.

## Known limitations and next step

No source maps, AST/static symbol mapping or log chronology. Forensics still targets Node/TypeScript and the first resolved source. AI references are validated but semantic interpretation can be wrong; the previously observed larger-model timeout remains a limitation. No extra models were introduced.

Experiments inherit account and network permissions and are not an OS sandbox. Ignored files/external effects are not covered by active-checkout hashing. Dependency installation is never automatic; unsupported package-manager launcher layouts fail explicitly. Current submodule repositories are rejected; submodules are not recursively initialized. Merge/root regression boundaries are inconclusive, and Git bisect assumes a monotonic predicate; repeated endpoints/immediate-boundary checks do not prove absence of flakiness. Hashes detect corruption but are not signatures against full database rewriting. Cases/artifacts are local and unencrypted. The installer is unsigned and a clean-machine install remains unvalidated.

Next recommended step: validate the unsigned installer on a clean Windows machine with documented Git/WebView2 prerequisites, then add signing/distribution validation. Further predicate profiles, dependency preparation and broader root-cause work require separate milestones.

---

# Historical 0.1.0 record (Phase 0/1)

The following is the retained historical report; its deferred-feature statements and artifact hashes describe 0.1.0, not the current 0.2.0 runtime.

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
