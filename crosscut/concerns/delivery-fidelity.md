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

Applies strongly; today the skill reaches no other project at all. First view,
observed on this machine.

- **Not installed:** there is no `crosscut` on `PATH`, no
  `~/.claude/skills/crosscut`, and no `~/.agents/skills/`. Every
  cross-project use so far must have come from pasting
  `crosscut prompt setup`. That is fine for a day-old tool. It means there is
  no skew yet, and also that nothing has exercised the install path.
- **No version in installed copies:** `install_skill` (`src/main.rs:234-259`)
  writes the files verbatim, with no version or commit stamp. It never
  removes files the skill no longer ships, so a deleted mode would keep
  loading from an old install. An agent elsewhere cannot tell which CrossCut
  it has. The cheapest structural fix is to write one stamp line (a version
  or commit) into the installed SKILL.md, and have `crosscut` warn when the
  two differ.
- **Install path:** `README.md:16` says
  `cargo install --git https://github.com/maxeonyx/crosscut`. The repository
  is **private** (`gh repo view`), so this works only with the owner's
  credentials, and not at all from a work machine without them. Deliberate
  for now? (A question for Max.)
- **Codex skill location:** `design.md:39-41` says Codex loads from
  `.agents/skills`. On this machine, the existing Codex skills live in
  `~/.codex/skills/`. That directory is not one of `install-skill`'s
  defaults. I did not verify which one codex-cli 0.154.0 actually reads.
  Unknown, and worth a quick check with Codex's documentation.
- **Harness flags:** every flag used in `Harness::command` exists in claude
  2.1.283, codex-cli 0.154.0 and opencode v1.14.22-max.22 (checked with
  `--help`). Fine but easy to regress, and no test or CI would notice: the
  tests use `sh -c` harnesses by design (`tests/cli.rs:1-2`). The OpenCode
  here is a personally patched fork (`-max.22`), so "works with opencode"
  currently means "works with Max's fork".
- **No CI or release:** there is no `.github/`. The sibling tools in the
  umbrella have `devenv.nix`, and some have `rust-toolchain.toml`. CrossCut
  is untracked in the umbrella (`git status` there shows `?? tools/crosscut/`,
  and it is excluded in the umbrella's `Cargo.toml`). This is consistent
  with "develop separately until 1.0" (`design.md:223-225`).
- **Worth considering:**
  - an install stamp, plus a staleness warning;
  - a single, cheap, manual "smoke refresh" recipe for use after a harness
    upgrade, such as one scratch concern with a trivial "How to look". It
    exists in spirit in `AGENTS.md` ("try it on a scratch concern first"),
    but not as something anyone can run.
- **Since last view:** first view.
- **Noticed along the way:** the skill `description` asks the harness to
  load CrossCut "when a fix looks like it may reveal a broader weakness".
  Installed globally across many projects, that could pull SKILL.md (5KB)
  plus a mode file into many ordinary sessions. For generalize, the mode file
  is 1.4KB, so the whole load is small. The bigger cost is the attention a
  mid-task CrossCut detour takes, not the tokens. Whether that is worth it is
  unknown until it is in use.
