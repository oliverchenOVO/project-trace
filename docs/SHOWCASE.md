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

## Phase 1.1: experimentally verify the existing case

12. Reopen the saved case with the actual AI hypothesis. Select **Verify hypothesis**.
13. Enter executable `npm.cmd`, arguments `["test"]`, working directory `.`, timeout 60 seconds. Select **Run current bad test**, inspect the exact plan and choose **Approve and run**. Expect TEST REPRODUCED at the manifest HEAD, retained executed output, source unchanged and cleanup cleaned.
14. Paste the manifest's `good` full SHA, select **Test known good**, review and approve. Expect KNOWN GOOD PASSED.
15. Select **Review bisect execution**. Review the same selected predicate and approved good/bad SHAs, then choose **Approve and bisect**. TRACE rechecks endpoints, uses real Git bisect and retains every test step.
16. Expect the manifest's `regression` SHA as first bad, BISECT CONFIRMED / REGRESSION BOUNDARY VERIFIED, source unchanged and worktrees cleaned. Root cause unverified must remain visible.
17. Close the Lab and select the boundary node. Inspect full first-bad SHA, good/bad SHAs, exact command and links to each original execution. Select the matching suspect commit to see its scoped confirmation.
18. Close and restart TRACE. Reopen the saved case and inspect retained executions/boundary; no tests rerun automatically.
19. Separately approve a `node` command with arguments `["-e", "setInterval(()=>{},1000)"]`. A short timeout must be INCONCLUSIVE with timeout output, not BAD. A longer run cancelled intentionally must be CANCELLED and cleaned. If the app closes during it, restart should show INTERRUPTED VERIFICATION and owned cleanup/discard controls.

The fixture is the same real three-commit repository; no Git/test/bisect output is mocked. Source maps, static symbol mapping and log chronology remain deferred. Installation remains unsigned and clean-machine installation has not been verified.
