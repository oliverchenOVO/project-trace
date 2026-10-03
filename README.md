# TRACE — AI Bug Crime Scene

A local-first Windows desktop lab for investigating JavaScript / TypeScript / Node.js failures in real Git repositories. Follow incident → stack frame → source → blame → commit → actual patch, with inspectable provenance at every step.

**TRACE never invents evidence. TRACE investigates evidence.**

Observed input, deterministic derivation, correlation and AI hypotheses have separate labels and graph treatments. Blame establishes last modification, never causation. Pasted test output is explicitly user-supplied; it does not claim TRACE ran a test.

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
npm run test:e2e
npm run tauri -- build
```

The Windows executable is `src-tauri/target/release/TRACE.exe`. NSIS installers are under `src-tauri/target/release/bundle/nsis/`. Build results and actual acceptance status are recorded in [validation](docs/VALIDATION.md).

## Optional AI

TRACE connects only to local Ollama at `127.0.0.1:11434`. Start your Ollama service with an installed model. In an investigation, choose **AI context**, inspect the exact redacted package, enter the installed model name, then generate a hypothesis. No API key is stored and nothing is automatically sent to a remote AI provider. AI cannot execute tools, manufacture deterministic nodes or supply arbitrary graph edges. Responses containing unknown evidence IDs or unexpected schema fields are rejected.

Local model quality varies; cited evidence and confidence labels cannot substitute for causal verification. TRACE does not implement bisect or repository test execution in this version.

Cases live in the application's local data folder as `cases.sqlite`, outside investigated repositories. Saved cases retain original excerpts. Reopen or choose **Check snapshot** to detect changed HEAD, worktree status or source hashes.

See [architecture](docs/ARCHITECTURE.md), [evidence model](docs/EVIDENCE_MODEL.md), [security](docs/SECURITY.md), and the [exact showcase workflow](docs/SHOWCASE.md).
