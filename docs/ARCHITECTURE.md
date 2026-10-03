# Architecture

## Native boundary

Tauri 2 hosts React 19 / TypeScript / Vite with React Flow. Native asynchronous commands dispatch bounded blocking work to worker threads. The renderer can open a repository, analyze incident inputs, reopen a case, review AI context or request a local hypothesis. It cannot submit shell strings or Git arguments.

`repository` validates canonical roots and registers session tokens. `stack` implements the extensible `StackTraceParser` trait. `git` launches the installed CLI directly with argument arrays, bounded output and a deadline. `investigation` resolves exact lines, records source hashes and builds explicit nodes and edges. `store` owns SQLite snapshots. `ai` builds the relevant context, redacts common secrets, contacts fixed local Ollama and validates the response.

The stack pipeline is input → NodeParser → canonical file resolution → UTF-8 excerpt → targeted Git history / single-line blame / commit metadata / patch. It supports Windows drive paths, spaces and percent-encoded file URLs. Source maps and AST symbol mapping are not implemented. Function names come from the stack and are not represented as statically verified symbols.

Analysis resolves at most eight frames, twelve recent commits per history query and targeted patches only. Git forensics focuses on the first resolved source; other frames retain source evidence without repeating history queries. Unresolvable frames remain visible with notices. All parsed frames remain available in the stack inspector. HEAD, branch and dirty state come from one porcelain-v2 status snapshot. No entire-repository indexing occurs. The graph defaults to a primary path; expand frames and focus path provide progressive inspection.

## Epistemic boundary

Only native parsers, filesystem reads and actual Git output produce observed/derived evidence. Heuristic text matches remain correlated candidate signals. A model returns a strict hypothesis object referencing only context-included IDs. The native application owns the confidence rule, hypothesis node and supporting/contradicting edges. No inference can create a verified node or a `causes` edge.

## Persistence

SQLite stores the full original snapshot, repository identity and HEAD, worktree status, raw inputs, parsed frames, nodes, edges, provenance, hypotheses and source hashes. Reopening compares live metadata and hashes; missing source is invalid, changed source is stale. It retains original excerpts. A repository missing entirely yields an actionable open error; the database snapshot remains intact.

## Test adapter

`trace-probe` is a JSON stdin/stdout development adapter to the same core. The production-bundle QA server injects only a test transport for Tauri IPC and requires the exact loopback origin. Vite additionally mounts an optional development bridge only when `TRACE_QA=1`. There is no such listener or injected transport in the packaged desktop app. Browser tests use actual Git output, not fabricated evidence. A separate WebView2 smoke script exercises the real desktop IPC.
