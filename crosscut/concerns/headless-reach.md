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

Applies strongly. The harnesses still enforce very different things, and the docs still describe the strongest case as if it applied to all of them. Nothing in `src/main.rs` has changed since the last view: there is still one commit, `1e5539a`, and the line numbers are the same. This refresh adds findings about inherited settings, and about flags the harnesses offer that CrossCut does not use. The evidence comes from source, `--help` output and local config files (claude 2.1.283, codex-cli 0.154.0, opencode v1.14.22-max.22), not from adversarial runs.

- **Claude Code (`src/main.rs:412-421`):** it runs with `--permission-mode dontAsk --allowedTools Read,Grep,Glob,Bash,WebFetch,WebSearch`.
  - Unscoped `Bash` can write, delete, `git push` and reach the network. The only barrier is a sentence in the prompt at `:488`. High confidence.
  - **New: target repo settings.** `claude --help` says `-p` "skips the workspace trust dialog… Only use this in directories you trust". The command runs in the directory that holds `crosscut/`. When a project keeps its own concerns, any `.claude/settings.json` in that project is loaded without asking. That includes hooks, permission allow rules and MCP servers. So a checked-in project file, not just the concern text, can widen reach. Medium-high confidence: this is inferred from the help text and not tried.
  - **New: available flags.** Claude offers `--bare` (skip hooks and plugins), `--setting-sources`, `--strict-mcp-config`, `--max-budget-usd` and `--model`. None of them is passed.
  - **This machine:** `~/.claude/settings.json` has no permissions and no hooks, and one plugin (rust-analyzer-lsp). Neither this repo nor its parent has a `.claude/` directory. Other machines are unknown.
- **Codex (`:423-433`):** `--sandbox read-only` is a real boundary, enforced by the harness.
  - The help text does not say whether read-only also blocks network. It is still unconfirmed whether `curl` and `gh api` behave differently under Codex. Medium confidence that they are blocked.
  - Codex user config (`-c` overrides, `~/.codex/config.toml`) can still apply. What it contains here was not checked.
- **OpenCode (`:435-438`):** it runs as `opencode run <prompt>` with no permission flags, and the prompt goes on argv. The argv concerns are unchanged: the full prompt is visible in `ps`, and it will approach Windows' 32K command-line limit as concern files grow.
  - **This machine:** `~/.config/opencode/opencode.jsonc` restricts only `external_directory` (allowing `/tmp`, the cargo registry and similar). It sets nothing for `edit` or `bash`, so reach is OpenCode's defaults, which I believe allow both. It also loads local plugins and agents. Medium confidence.
  - It sets `experimental.permission_timeout: 240000`. Whether a headless run waits on a permission prompt or rejects it is unknown.
  - **New: available flags.** `opencode run` has `--pure` (no external plugins), `-m/--model`, `--ephemeral`, and `-f/--file` for attachments. `-f` might carry the prompt instead of argv, but this is untested. Its `--dangerously-skip-permissions` is described as approving only what is "not explicitly denied". That suggests a `permission` block passed per run could deny edit and bash. Whether such a block can be given per invocation is unknown.
- **Bounds:** unchanged.
  - There is no process timeout. `wait_with_output` at `:539` blocks forever on a hung harness.
  - There is no budget or turn limit, and no way to choose the model.
  - With no slugs given, it refreshes every concern in sequence, using the first harness found on `PATH` (`:391-398`, Claude first). So the default is the least constrained harness that is installed.
- **Docs versus reality:**
  - `skill/modes/refresh.md:58-60` says a headless agent "never needs write access". The last view cited this as `:299-302`, which was wrong.
  - `docs/design.md:207` says "needs no write permission".
  - Both describe what the agent *needs*. Neither says what the Claude and OpenCode paths actually *have*.
  - `design.md:109-110` names prompt injection as applying to CrossCut itself. The new trust-dialog finding makes that concrete: the injection can come from repo settings, not just from prose.
- **Worth considering:** ordered roughly by leverage per unit of effort.
  - Pass `--bare` or `--setting-sources user` plus `--strict-mcp-config` to Claude. This is one line, and it removes the vector where target repo settings apply.
  - Scope Claude's Bash (for example `Bash(git log:*)`), or prefer Codex's sandbox when it is installed. Or state the asymmetry plainly in `refresh.md` and `--help`.
  - Pass through `--model` and `--max-budget-usd` for Claude, `-m` and `--pure` for OpenCode, and add a timeout for each concern.
  - Check whether OpenCode can take a per-run deny policy, and whether the prompt can travel as a file via `-f`.
- **Since last view:**
  - Code: no changes.
  - New findings: repo settings apply because `-p` skips the trust dialog; mitigating flags exist in both Claude and OpenCode; OpenCode's actual permission config on this machine was read.
  - Corrected the `refresh.md` line citation.
- **Noticed along the way:**
  - Views still don't record which harness or model wrote them. That was raised last time and nothing has changed.
  - The `refresh.md:299-302` miscitation looks like a line number from the concatenated prompt, not from the file. Headless agents only see that combined prompt, so they may keep citing positions in it as if they were file lines. That could be worth one sentence in the refresh prompt.
  - (Added by hand after this refresh; the headless run did not see it.)
    `claude -p` also loads `CLAUDE.md`/`AGENTS.md` from every parent
    directory. Here that means the umbrella's `AGENTS.md`, which says
    "Commit and push frequently" (`../../AGENTS.md:314`) and "The concern is
    not real until enforcement exists" (`:163`). Those are the opposite of
    CrossCut's doctrine, and they are loaded next to an unscoped Bash. In a
    target project, whatever that project's guidance tells agents to do
    reaches the refresh agent too. `--bare` or `--setting-sources` may or may
    not suppress memory files; that is unverified.
