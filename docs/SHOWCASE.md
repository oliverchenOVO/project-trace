# Showcase: a real missing-user regression

1. Run `npm run fixture` once. The developer script creates `fixtures/showcase` with three real Git commits. Inspect `evidence/manifest.json` for exact safe, regression and HEAD SHAs. It actually runs the safe tests (exit 0), regression tests (exit 1) and crashing Node script.
2. Launch `TRACE.exe`, then choose **Open repository…** and select `fixtures/showcase`. **Path…** also accepts the absolute Windows path, including spaces and Unicode.
3. Paste `fixtures/showcase/evidence/stack.txt` into **Stack trace**. This is actual stderr from `node crash.mjs`, including encoded Windows file URLs.
4. Switch to **Test output · optional** and paste `evidence/test-output.txt`. TRACE records it as user-supplied output; it does not claim to have run the test itself.
5. Choose **Build evidence graph**. Select the source node for `src/parser.ts:3`. The highlighted line is `return user!.id.trim();` and the exact column comes from the stack.
6. Select **Last changed line**, then the commit. Compare its full SHA to `regression` in the manifest. The later documentation-only commit must not be blamed for the crash line.
7. Select **Actual commit patch**. Git's real patch contains the removed `if (!user) return null;` and changed return expression. Select **Provenance** to inspect the recorded argument array and original output.
8. Select **Attached test output** to read the real failure, including the test command, cwd and exit code captured by the developer script.
9. Use **AI context** to inspect the exact redacted excerpts. Enter an installed local Ollama model, for example `qwen2.5:1.5b` if available on your machine. Choose **Generate hypothesis**. Unknown citations or schema violations produce an explicit error, not a graph node.
10. Select the violet dotted hypothesis. Check supporting evidence links, uncertainty, verification steps and LOW/MEDIUM confidence explanation. The status bar must still say **Root cause unverified**.
11. Close and reopen TRACE, then select the saved case. The snapshot should persist. Use **Check snapshot** after changing a fixture source file to inspect stale evidence warnings; restore fixture changes manually or regenerate a fresh fixture folder.

The core collection path works with arbitrary local Git repositories. Missing files, generated files, unsupported stack frames and source-map locations produce explicit notices. Current source is an inspection snapshot, not proof of the original deployed revision.

Phase 1.1: isolated worktree bisect with an explicit test command, command execution capture, source maps and static symbol mapping. Until then there is no causal verification or BISECT CONFIRMED label.
