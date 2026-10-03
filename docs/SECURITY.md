# Security and local repository boundaries

Normal investigations are read-only. Git operations are fixed internal calls to rev-parse, status, log, show, diff and blame. The service also permits ls-files for future targeted path discovery. No frontend command accepts arbitrary executable names, shell strings or Git argument arrays. No repository script, package hook or content instruction can trigger execution.

Git is spawned with executable + argument arrays, no shell, no optional locks, prompts disabled and hidden child windows on Windows. Inherited GIT_* variables are stripped. fsmonitor is disabled, signature verification is disabled, pager is fixed, and patches disable external diff and textconv. Output is limited to 2 MB per stream and each Git operation has a 20-second deadline. Oversized or timed-out operations fail explicitly.

Repository roots are canonicalized and registered as session tokens. Sources are canonicalized, checked against the root, and must be regular files. Both slash styles of `..` traversal are rejected; file URLs are decoded before validation. Symlinks resolving outside the root are rejected. Source reads are capped at 512 KB and require UTF-8 nonbinary content. SHA-256 checks after analysis detect source changes; HEAD and worktree status are checked before and after collection.

Sensitive/generated path components are blocked: .git, .env, .env.*, *.pem, *.key, id_rsa, id_ed25519, credentials*, secrets*, node_modules, dist, build and coverage. These exclusions apply before source inspection and AI packaging. Relevant patch content is always restricted to the resolved file.

AI receives only explicitly previewed evidence excerpts, bounded per item and by a 32 KB working budget. Common credential assignments, bearer tokens, common provider token formats and PEM private-key blocks are redacted. Pattern redaction is not a complete secret scanner: review the context before sending it even to a local model. AI is fixed to loopback Ollama, with proxies disabled, no tools, structured output and strict evidence-ID validation. Hypothesis nodes cannot enter subsequent deterministic AI context.

The local SQLite database intentionally preserves original inputs and source excerpts. It is not encrypted. It stays outside the investigated repository and is never committed to GitHub. Access follows the current Windows user's filesystem permissions. Do not publish a case containing confidential material.

Test execution and bisect are deferred; a future implementation must require explicitly configured commands and isolated worktrees. The fixture creation script is a developer tool that creates real commits in its own new folder and refuses to overwrite an existing directory. It is not invoked by normal analysis.
