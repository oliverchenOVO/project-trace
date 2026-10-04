# TRACE — AI Bug Crime Scene

A local-first Windows desktop lab for investigating JavaScript / TypeScript / Node.js failures in real Git repositories. Follow incident → stack frame → source → blame → commit → actual patch, with inspectable provenance at every step.

**TRACE never invents evidence. TRACE investigates evidence.**

Observed input, deterministic derivation, correlation and AI hypotheses have separate labels and graph treatments. Blame establishes last modification, never causation. Pasted test output is explicitly user-supplied; it does not claim TRACE ran a test.

## Interface language (0.2.1)

Use the header's **繁體中文 / English** selector to change language immediately. The choice is saved locally and restored on restart; the initial default follows the browser/system language, with Traditional Chinese for Chinese locales. Headers, dialogs, navigation, evidence/status labels and Verification Lab use the chosen language. Raw source, Git patches/commit messages, supplied output and model responses remain unchanged. Technical identifiers and original diagnostic details may remain in their original language. If local preference storage is unavailable, switching still works for the current session.

## Development

Prerequisites: Node 24+, installed Git, stable Rust MSVC, Visual Studio C++ Build Tools and Windows WebView2. No WSL required. Ensure Cargo is on PATH (`$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH` in PowerShell if necessary).

```powershell
npm ci
npm run fixture
npm run tauri -- dev
```

`npm run dev` alone is a visual browser preview; local repository commands require the desktop app. The explicit `TRACE_QA=1` development harness exists only for end-to-end validation against the same native Rust core and is never shipped in the frontend bundle.

## Tests and Windows package

```powershell
npm test
npm run test:native
cargo build --manifest-path src-tauri/Cargo.toml --no-default-features --bin trace-probe
npx playwright install chromium
npm run build
npm run test:e2e
npm run tauri -- build
```

The Windows executable is `src-tauri/target/release/TRACE.exe`. NSIS installers are under `src-tauri/target/release/bundle/nsis/`. Build results and actual acceptance status are recorded in [validation](docs/VALIDATION.md).

## Verification Lab (0.2.0)

Open a saved case and select **Verify hypothesis**. Enter a direct executable and JSON argument array, review the resolved command/revision/location/permissions, then explicitly approve execution. For the existing fixture use `npm.cmd` with arguments `["test"]`, or `node` with `["--test", "test.mjs"]`. No dependencies are installed automatically.

Run the current bad test, select the manifest's full known-good SHA and test it, then review and approve bisect. Each revision is tested in a clean detached worktree. The result is **BISECT CONFIRMED** for the selected test predicate and range; **Root cause unverified** remains visible. Timeout/cancellation/launch errors never become BAD classification. Source HEAD, branch, status and nonignored file digest are compared before/after. Cases retain all executions and hashes across restart; interrupted worktrees have explicit cleanup/discard controls.

Approved tests inherit normal user/network/filesystem permissions: checkout isolation is not a security sandbox. See [verification engine](docs/VERIFICATION_ENGINE.md) for the command boundary, Windows process trees, capture limits, cleanup and semantic scope.

## Optional AI

TRACE connects only to local Ollama at `127.0.0.1:11434`. Start your Ollama service with an installed model. In an investigation, choose **AI context**, inspect the exact redacted package, enter the installed model name, then generate a hypothesis. No API key is stored and nothing is automatically sent to a remote AI provider. AI cannot execute tools, manufacture deterministic nodes or supply arbitrary graph edges. Responses containing unknown evidence IDs or unexpected schema fields are rejected.

Local model quality varies; cited evidence and confidence labels cannot substitute for causal verification. TRACE 0.2.0 adds explicitly approved repository tests and Git bisect in isolated worktrees. Model interpretation remains separate from experimental boundary evidence.

Cases live in the application's local data folder as `cases.sqlite`, outside investigated repositories. Saved cases retain original excerpts. Reopen or choose **Check snapshot** to detect changed HEAD, worktree status or source hashes.

The end-to-end harness serves the production bundle on `127.0.0.1:24917` with a test-only IPC transport adapter. Build the frontend before running it. Optional real AI E2E: set `TRACE_AI_MODEL` to an installed Ollama model. Browser tests neither invent Git output nor simulate model responses.

See [architecture](docs/ARCHITECTURE.md), [evidence model](docs/EVIDENCE_MODEL.md), [security](docs/SECURITY.md), and the [exact showcase workflow](docs/SHOWCASE.md).
