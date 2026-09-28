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

Applies strongly. Commit `b200de9` narrowed the Claude path and made the docs honest, and a refresh is now bounded in time. Claude's unscoped Bash is still the main open reach, along with OpenCode having no flags at all. The evidence comes from source, `--help` output and local config (claude 2.1.283, codex-cli 0.154.0, opencode v1.14.22-max.22). It also comes from this refresh itself: `ps` shows it was launched by `target/debug/crosscut refresh headless-reach --harness claude`, so the Claude observations below describe the real command from the inside. No adversarial runs were made.

- **Could it disappear?** Not yet. Only Codex runs in a real read-only sandbox. The default harness is still the first one found on `PATH`, Claude first (`src/main.rs:403-423`).
- **Claude (`src/main.rs:432-444`):** the command is now `-p --no-session-persistence --setting-sources user --permission-mode dontAsk --allowedTools Read,Grep,Glob,Bash,WebFetch,WebSearch --disallowedTools Edit,Write,NotebookEdit`.
  - **Project settings no longer load.** `--setting-sources user` removes the last view's main new vector: a target's `.claude/settings.json` hooks, allow rules and MCP servers. High confidence that this is enforced by the flag.
  - **Memory files are probably no longer loaded either.** The umbrella's `CLAUDE.md` is `@AGENTS.md`, and so is the one in this repo. This run's context has no trace of that content (for example "Commit and push frequently"). That suggests project-level memory files are not loaded under `--setting-sources user`. Medium confidence: this rests on one observation from inside the run and no documentation.
  - **Unscoped `Bash` remains.** Denying Edit, Write and NotebookEdit removes only the convenient write tools. Bash can still write, delete, `git push`, and reach the network with `curl` or `gh`, and WebFetch is also allowed. High confidence.
  - **User-level extras still load.** They are inherited from user settings and the account. In this run the rust-analyzer plugin's `LSP` tool was visible, and so were `Agent`, `Workflow`, `WebFetch` and `WebSearch`. So were claude.ai-connected MCP tools that can write (`mcp__claude_ai_Claude_Docs__batch`/`update`/`create`/`delete`, which create docs on an external service).
    - None of these is on `--allowedTools`. Under `dontAsk` they are probably refused, but that was not tested, deliberately.
    - `Agent` and `Workflow` could spawn subagents, and whether they are denied is unknown. If they are allowed, cost could multiply.
    - `--strict-mcp-config` is not passed, and it is unknown whether it would suppress claude.ai connectors.
  - **Why not `--bare`.** Help now shows that `--bare` requires `ANTHROPIC_API_KEY` auth, so it is not a free drop-in for subscription users. That explains why `--setting-sources` was chosen instead.
- **Codex (`:445-455`):** `exec --sandbox read-only --skip-git-repo-check --ephemeral`, with the prompt on stdin. This is still the one enforced boundary.
  - Whether read-only also blocks network is still not confirmed from help. Medium confidence that it does.
  - `~/.codex/config.toml` was not read in this refresh.
- **OpenCode (`:456`, `:488-490`):** `opencode run <prompt>`, plus `-m` if a model is given. It still passes no permission flags and no `--pure`, and the prompt still travels on argv, so it is visible in `ps` and limited in length on Windows.
  - On this machine the permission config was not re-read. The last view found that it restricts only `external_directory`, so edit and bash fall to OpenCode's defaults (believed to allow both).
- **Custom `--harness` commands (`:457-472`):** they run through `sh -c`/`cmd /C` with whatever reach the user's command has. This is explicit and user-chosen, so it is what they would expect.
- **Bounds:**
  - **Time:** there is now a per-concern timeout, 30 minutes by default (`:135`). It kills the child on expiry (`:586-599`). This is enforced. The kill targets only the direct child, so whether grandchildren the harness spawned (shells, MCP servers) are also stopped is unknown.
  - **Model:** `--model` now passes through (`:474-483`).
  - **Cost:** still no budget or turn limit. `--max-budget-usd` exists for `claude -p` and is not passed. A 30-minute run is bounded, but spend inside it is not.
- **Docs versus reality:** these now largely match.
  - `skill/modes/refresh.md:56-63` says plainly that "never needs to" is not "cannot", that only Codex enforces read-only, and that concern files should be trusted like scripts.
  - `docs/design.md:206-210` says the same.
  - `README.md:13` makes no reach claim. It is silent rather than wrong, and a user who reads only the README or `--help` would not learn about the asymmetry. Whether `crosscut refresh --help` states it was not checked.
- **Worth considering:**
  - Pass `--max-budget-usd` (or expose it) for Claude. It is one flag and closes the remaining unbounded dimension.
  - Scope Bash (for example `Bash(git log:*)`, `Bash(cargo *)`), or deny `Agent`, `Workflow` and `mcp__*` explicitly via `--disallowedTools`. Either one makes the Claude reach enforced rather than dependent on `dontAsk` defaults.
  - Prefer Codex when both it and Claude are installed, since Codex is the one with a real sandbox. Or say in `--help` which harness gets which reach.
  - OpenCode: `--pure`, a per-run deny policy if one exists, and `-f` for the prompt. All three are still unexplored.
  - Kill the process group on timeout, not just the child.
- **Since last view:**
  - **Code changed** (`b200de9`): `--setting-sources user`, `--disallowedTools Edit,Write,NotebookEdit`, `--no-session-persistence`, a timeout, `--model`, and Codex `--ephemeral`. The last view's claim that the code was unchanged no longer holds, and its line numbers are stale.
  - **Docs** now state the asymmetry honestly.
  - **New observations:** memory files are apparently suppressed, user-level plugin and claude.ai MCP tools are still present, and `--bare` needs an API key.
- **Noticed along the way:**
  - The previous view claimed "one commit, `1e5539a`" while describing code that `b200de9` then changed in the same commit that added the concern. Views written against an uncommitted working tree can misdate themselves. Recording `git rev-parse HEAD` (and dirty state) in each view would make this visible, and it fits the existing open point that views don't record which harness or model wrote them.
  - Account-level claude.ai connectors reach headless runs regardless of any repo or user settings file. That is a reach vector outside anything CrossCut or the target controls.
