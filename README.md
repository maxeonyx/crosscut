# CrossCut

CrossCut keeps a map of the cross-cutting concerns in your projects: the generally useful things that have to be true for people's needs to be met. Examples:
- **version visibility:** can you tell what's live?
- **staying current:** do installed copies update themselves?
- **recoverable state**
- **one-step releases**
- **a front door** that says what the project is for
- and whatever your own ecosystem turns out to need.

For each concern, the map shows how every project meets it, differently or not at all. The gaps are the point. For example: one tool updates itself and its sibling doesn't, or nobody publishes a version.

```
                    larder    ledger-web  nightly-sync  pantry
recoverable-state   n/a       n/a         partly        n/a
staying-current     missing   n/a         n/a           yes
version-visibility  missing   yes         deferred      yes
```

How it's built:
- **One directory per concern**, holding its user stories and a check. The check is the cheapest thing that answers it:
  1. an off-the-shelf tool;
  2. a custom script, tested against invented fixtures;
  3. a prompt for your coding agent, when it takes judgment.
- **Observations are kept apart from decisions.** "Deferred: being retired" is yours, and no run overwrites it.
- **Git holds the history of the map.**

Nothing here gates, scores or passes anything.

```bash
cargo install --git https://github.com/maxeonyx/crosscut --locked
crosscut install-skill        # teach Claude Code, Codex and OpenCode how to use it
crosscut check                # run the machine checks; --agentic adds the prompt checks
crosscut map                  # the grid
crosscut test                 # check the checks against their fixtures
```

Then tell your agent: *"Use CrossCut here."*

Anyone extending or rewriting CrossCut, or anything it produces, must keep this framing, and must pass on the requirement to keep both the framing and this requirement. See the doctrine in [`skill/SKILL.md`](skill/SKILL.md).
