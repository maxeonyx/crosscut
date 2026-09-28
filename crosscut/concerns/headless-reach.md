# What can a headless `crosscut refresh` actually do to the machine and repositories it runs in?

## Why this matters here

`crosscut refresh` hands a model a prompt built from repository content (the
concern file, `crosscut/projects.md`) and runs it unattended, often over
projects the operator did not write, and later possibly at work. Its promise,
in `skill/modes/refresh.md` and `docs/design.md` ("the headless run needs no
write permission"), is that a refresh only reads. Whether that is enforced or
merely requested decides the blast radius of one hostile or careless line in a
concern file, and whether a scheduled refresh can be left running without
anyone watching.

This applies strongly: the product is meant to be pointed at many unrelated
projects, and each harness enforces different things. Better means that the
reach of each supported harness is enforced by the harness (sandbox, tool
allowlist), matches what the docs claim, and is stated plainly where it is
weaker. It also means a runaway run is bounded in time and cost.

## How to look

1. **Could it disappear?** If `crosscut refresh` only ran harnesses in a real
   read-only sandbox, most of this question would reduce to "is the sandbox
   still on?". Say whether that is now the case.
2. **Structural facts (deterministic).** Read `Harness::command` in
   `src/main.rs` and note, per harness, what is enforced by flags rather than
   by prompt text:
   - tool allowlist and permission mode (Claude Code);
   - sandbox mode and network (Codex);
   - permission configuration (OpenCode);
   - any timeout, turn limit or budget (`claude --max-budget-usd`,
     `--max-turns`, a process timeout);
   - which model is used, and whether it can be chosen.
3. **Existing tools.** `claude --help`, `codex exec --help`,
   `opencode run --help`, and each harness's documentation for what its
   non-interactive mode inherits from user settings (hooks, MCP servers,
   pre-approved permissions). Record the harness versions you checked.
4. **Judgment.** Compare the enforced reach with what `refresh.md`,
   `design.md` and `README.md` promise. Ask what a concern file written by a
   hostile or careless author could make each harness do: write files, push,
   exfiltrate over the network, spend without bound. Ask whether a user
   running from inside a target repository would expect that. Do not
   actually attempt anything destructive; reasoning from the enforced flags
   is the evidence.
- Not in scope: whether the model obeys "do not modify files". Obedience is
  not a boundary.

## Current view — 2026-09-28

Commit `d8866a1` says subagents and MCP servers are now denied to Claude refreshes, but it changed only a code comment. Neither is enforced, and this run could call claude.ai tools that write to an external service. Codex is still the only harness with an enforced boundary. Evidence comes from source at `47a63e0`, `--help` output, local config and this run itself (claude 2.1.283, codex-cli 0.154.0, opencode v1.14.22-max.22). `ps` shows this run was launched as `target/release/crosscut refresh --harness claude`. No adversarial runs were made.

- **Could it disappear?** Not yet. The default harness is still the first one found on `PATH`, Claude first (`src/main.rs:407-428`).
- **Claude (`src/main.rs:434-449`):**
  - **The flags are unchanged since `b200de9`:** `-p --no-session-persistence --setting-sources user --permission-mode dontAsk --allowedTools Read,Grep,Glob,Bash,WebFetch,WebSearch --disallowedTools Edit,Write,NotebookEdit`. `ps` on this run's parent process confirms exactly this.
  - **The comment overstates the reach.** The comment at `:435-438` says "MCP servers and subagents are off too, so one refresh cannot reach external services or fan out". Neither is true. `git show d8866a1 -- src/main.rs` is a two-line comment change, and there is no `--strict-mcp-config` and no `Agent`, `Workflow` or `mcp__*` entry in `--disallowedTools`. `tests/cli.rs:358-365` checks only the flags that do exist. High confidence.
  - **External write tools were available.** In this run, `mcp__claude_ai_Claude_Docs__batch` and `__update` were delivered as directly callable tools, and `create` and `delete` were a lookup away. `Agent` and `Workflow` were also listed. None of these is in `--allowedTools`. Whether `dontAsk` would refuse them at call time is unknown. That was deliberately not tested, because a successful call would publish externally.
  - **Unscoped `Bash` and `WebFetch` remain.** They can write, delete, `git push`, and reach the network. High confidence.
  - **Project settings are not loaded,** because of `--setting-sources user`. High confidence.
- **Codex (`:451-461`):** `exec --sandbox read-only --skip-git-repo-check --ephemeral`.
  - **The user config is very permissive.** `~/.codex/config.toml` sets `sandbox_mode = "danger-full-access"` and `approval_policy = "never"`, so the boundary rests entirely on the CLI flag overriding config. Medium-high confidence that it does. This was not verified by a run.
  - **Network under read-only is not confirmed.** Help does not state it, and the `--help` text says only "usually without network".
- **OpenCode (`:462`, `:494-496`):** still `opencode run <prompt>`, with no permission flags, no `--pure`, and the prompt on argv. The config was not re-read in this refresh.
- **Custom `--harness`:** it runs with whatever reach the user's own command has. That is explicit and user-chosen.
- **Bounds:**
  - **Time:** the 30-minute per-concern timeout is enforced, but it kills only the direct child (`:596-599`). Whether grandchild processes are also stopped is unknown.
  - **Cost:** still no budget or turn limit, even though `--max-budget-usd` exists. Since `598c14e`, six refreshes run at once by default, so the unbounded spend is multiplied by six.
- **Docs versus reality:**
  - `skill/modes/refresh.md:56-63` and `docs/design.md:208-214` are honest.
  - `crosscut refresh --help` now states the asymmetry: only Codex enforces read-only, and concern files should be trusted like scripts.
  - The misleading claim is in the source comment and in the `d8866a1` commit message, which a maintainer would trust.
- **Worth considering:**
  - Add `--disallowedTools Agent,Workflow,mcp__*` and `--strict-mcp-config`, or correct the comment. Adding a test would stop the claim drifting again.
  - Pass or expose `--max-budget-usd`, which matters more now that runs are parallel.
  - Prefer Codex when both it and Claude are installed.
  - For OpenCode, explore `--pure`, a deny policy and `-f`.
  - Kill the whole process group on timeout, not just the child.
- **Since last view:**
  - The last view's suggestion to deny `Agent`, `Workflow` and `mcp__*` was recorded in a commit as done, but it was not implemented.
  - Worker-pool parallelism was added.
  - The `--help` text now carries the reach warning.
  - The Codex user config was read for the first time.
- **Noticed along the way:** a view or commit message can describe intended changes as done. Nothing checks a claim like "X is now denied" against the code. A dry-run test for each claimed containment flag would catch this cheaply.
