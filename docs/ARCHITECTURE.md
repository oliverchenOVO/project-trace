# Architecture

## Native boundary

Tauri 2 hosts React 19 / TypeScript / Vite with React Flow. Native asynchronous commands dispatch bounded blocking work to worker threads. The renderer can prepare/review explicitly approved isolated experiments, poll progress, cancel and retry owned cleanup. It can also open a repository, analyze incident inputs, reopen a case, review AI context or request a local hypothesis. It cannot submit shell strings or Git arguments. The verification API accepts only structured test commands and a native approval token.

`repository` validates canonical roots and registers session tokens. `stack` implements the extensible `StackTraceParser` trait. `git` launches the installed CLI directly with argument arrays, bounded output and a deadline. `investigation` resolves exact lines, records source hashes and builds explicit nodes and edges. `store` owns SQLite snapshots. `ai` builds the relevant context, redacts common secrets, contacts fixed local Ollama and validates the response.

The stack pipeline is input → NodeParser → canonical file resolution → UTF-8 excerpt → targeted Git history / single-line blame / commit metadata / patch. It supports Windows drive paths, spaces and percent-encoded file URLs. Source maps and AST symbol mapping are not implemented. Function names come from the stack and are not represented as statically verified symbols.

Analysis resolves at most eight frames, twelve recent commits per history query and targeted patches only. Git forensics focuses on the first resolved source; other frames retain source evidence without repeating history queries. Unresolvable frames remain visible with notices. All parsed frames remain available in the stack inspector. HEAD, branch and dirty state come from one porcelain-v2 status snapshot. No entire-repository indexing occurs. The graph defaults to a primary path; expand frames and focus path provide progressive inspection.

## Epistemic boundary

Only native parsers, filesystem reads and actual Git output produce observed/derived evidence. Heuristic text matches remain correlated candidate signals. A model returns a strict hypothesis object referencing only context-included IDs. The native application owns the confidence rule, hypothesis node and supporting/contradicting edges. No inference can create a verified node or a `causes` edge.

## Persistence

A separate `case_metadata` table stores display names, normalized tags, archive state and last-opened timestamps. Existing databases migrate additively on connection; these fields do not alter evidence snapshots or verification runs. The case list includes archived records and is not capped at fifty; presentation filters do not delete data.

SQLite stores the full original snapshot, repository identity and HEAD, worktree status, raw inputs, parsed frames, nodes, edges, provenance, hypotheses and source hashes. Reopening compares live metadata and hashes; missing source is invalid, changed source is stale. It retains original excerpts. A repository missing entirely yields an actionable open error; the database snapshot remains intact.

## Test adapter

`trace-probe` is a JSON stdin/stdout development adapter to the same core. The production-bundle QA server injects only a test transport for Tauri IPC and requires the exact loopback origin. Vite additionally mounts an optional development bridge only when `TRACE_QA=1`. There is no such listener or injected transport in the packaged desktop app. Browser tests use actual Git output, not fabricated evidence. A separate WebView2 smoke script exercises the real desktop IPC.

## Phase 1.1 verification

`execution` resolves direct executables/package-manager CLIs and uses Windows kill-on-close Job Objects. `verification::Service` owns approval, native Git worktree/bisect commands, worker cancellation and persisted lifecycle. Commands from evidence/AI are never executed. Every test has retained output artifacts and structured integrity hashes. Git configuration filter discovery is bounded; configured filters are disabled for inspection and administration, and captured provenance records actual command flags.

The bisect coordinator and rebuilt per-candidate test checkout are distinct owned worktrees. State transitions include a cleanup phase so final boundary labels wait for cleanup and source-state comparison. Incremental case/run writes and reads use SQLite transactions. Startup marks interrupted experiments without resuming. Audits check raw artifacts and graph-to-run references. Verification nodes remain downstream from actual native executions; model schemas cannot create them.

See [verification engine](VERIFICATION_ENGINE.md) for full semantics and ownership rules.

## Reading and reporting (0.4.0)

Graph search covers all retained evidence before presentation compaction and supports exact kind filtering. Selection changes only React Flow presentation state; highlighted paths follow ancestors plus immediate descendants. Reports assign case-local E references while retaining original IDs. Offline HTML uses escaped tables, local fragment links and an inline SVG schematic (up to 60 nodes); Markdown keeps the full reference mapping. Neither exporting nor case management initiates experiments or AI calls.
