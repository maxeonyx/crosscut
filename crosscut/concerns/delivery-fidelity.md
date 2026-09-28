# When an agent in some other project uses CrossCut, is it getting the CrossCut in this repository, and does the harness invocation still work?

## Why this matters here

CrossCut is meant to be used across many unrelated projects, and later
possibly at work. Agents there never see this repository. They see:
- a copy of `skill/` installed by `crosscut install-skill`;
- a `crosscut` binary installed at some earlier point;
- whatever the harness CLIs accept today.

Each of these drifts on its own clock. A prompt improved here does nothing
until it is reinstalled. A harness release that renames a flag breaks
`crosscut refresh` silently in every project at once. An install path that
works only for the author blocks the work use case entirely. This applies
strongly while the prompts are changing daily, and it applies to every
harness named in `README.md`. Better means an agent anywhere can tell which
CrossCut it has, stale copies are cheap to notice, and a broken harness
invocation shows up before a scheduled refresh depends on it.

## How to look

1. **Could it disappear?** If the skill were loaded from one place that
   updates with the binary (for example `crosscut prompt` being the only
   delivery, or install-skill writing a stamp that the binary checks), skew
   would become structural. Say whether that has happened.
2. **Deterministic:**
   - `command -v crosscut && crosscut --version`, compared with
     `Cargo.toml`'s version and `git log -1`;
   - for each install location (`~/.claude/skills/crosscut`,
     `~/.agents/skills/crosscut`, and wherever each harness actually reads
     user skills), `diff -r skill <dir>`;
   - `claude --help`, `codex exec --help`, `opencode run --help`: does every
     flag in `Harness::command` (`src/main.rs`) still exist? Record the
     versions.
   - `gh repo view maxeonyx/crosscut --json visibility`: can the install
     command in `README.md` work for anyone but the owner?
3. **Judgment:**
   - From a fresh shell in an unrelated repository, could an agent find out
     which version of the skill it loaded, and whether it is stale?
   - Do the harness skill-discovery locations claimed in `docs/design.md`
     match each harness's current documentation?
   - Does `install-skill` leave behind files that the current skill no
     longer ships?
   - Is the SKILL.md `description` triggering in the sessions where it helps,
     and staying quiet in the others? Evidence for this only exists once
     CrossCut is in use elsewhere.
- Not in scope: the content of the prompts (see `doctrine-integrity`).

## Current view — 2026-09-28

Anyone can install CrossCut now, but it still reaches no other project on this machine. Installed copies still carry no version stamp. The Claude refresh is less contained than its code comment claims.

- **Could it disappear?** Not yet. `install_skill` (`src/main.rs:252-277`) still copies `SKILL_FILES` verbatim, with no version or commit stamp. It still never removes files the skill no longer ships. Skew is not structural, so an agent elsewhere could not tell which CrossCut it loaded, or whether it is stale.
- **Not installed anywhere (observed):**
  - there is no `crosscut` on `PATH`;
  - there is no `~/.claude/skills/crosscut`;
  - there is no `~/.agents/skills/`;
  - there is no `crosscut` under `~/.codex/skills/` or `~/.config/opencode/skills/`.

  So there is no skew to measure. `Cargo.toml` is 0.1.0 at `324221c`.
- **Install path: now public.** `gh repo view` reports `PUBLIC`, so `cargo install --git https://github.com/maxeonyx/crosscut --locked` (`README.md:16`) should now work without the owner's credentials. I did not try it from a clean machine.
- **Codex user skill location: probably wrong default (medium confidence).** The codex 0.154.0 binary's own skill-creator text says user skills go in `$CODEX_HOME/skills`, which defaults to `~/.codex/skills`. That directory is where this machine's existing Codex skills live. `.agents/skills` shows up in the binary only next to its external-agent-migration code. `docs/design.md:38-42` records Codex listing a *project-level* `.agents/skills/crosscut`, which does not settle the user level. `install-skill` writes only `~/.claude/skills` and `~/.agents/skills`. So a Codex user may not get the skill globally. A real Codex session would settle this cheaply.
- **Harness flags still exist (observed with `--help`):**
  - claude 2.1.283 has `-p`, `--no-session-persistence`, `--setting-sources`, `--permission-mode`, `--allowedTools`, `--disallowedTools` and `--model`;
  - codex-cli 0.154.0 has `exec`, `--sandbox`, `--skip-git-repo-check`, `--ephemeral` and `-m`;
  - opencode v1.14.22-max.22 has `run` and `-m`. That is still the owner's patched fork.

  These are fine but easy to regress.
- **CI now exists** (`.github/workflows/ci.yml`: fmt, clippy, test on ubuntu). It still would not notice harness drift. The tests use `sh -c` harnesses, and the runner has no harness CLIs installed. A manual smoke refresh after a harness upgrade remains the gap.
- **The Claude invocation doesn't do what its comment says (observed):**
  - Commit `d8866a1` says it denies subagents and MCP servers to Claude refreshes. It changed only the comment (`src/main.rs:434-437`). There is no `--strict-mcp-config`, and there is no `Agent`, `Workflow` or `mcp__*` in `--disallowedTools`.
  - This run is presumably such a refresh, going by its prompt format. It still had `Agent`, `Workflow` and claude.ai Claude Docs MCP tools that can write.
  - The invocation "works", but not as intended. This belongs mainly to `headless-reach`. It is noted here because a comment that runs ahead of the code is itself a fidelity gap.
- **Worth considering:**
  - an install stamp in the installed SKILL.md, plus a warning from `crosscut` when it differs;
  - adding `~/.codex/skills/crosscut` to the defaults, once confirmed;
  - a one-command scratch-concern smoke refresh for each harness;
  - making the Claude denials real, or correcting the comment.
- **Since last view:**
  - the repository went from private to public;
  - CI and a Pages site were added;
  - the design notes now record Codex listing a project-level skill;
  - the Codex user-level location has medium-confidence evidence;
  - the MCP/subagent comment gap is new.
- **Noticed along the way:** nothing checks that a commit message or code comment claiming a containment change matches the flags actually passed. Pinning the Claude argument list with a `--dry-run` snapshot test would make that structural.
- **Still unknown:** whether the SKILL.md `description` triggers in the right sessions. CrossCut is not in use elsewhere yet.
