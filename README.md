# CrossCut

Every project needs to track many shared requirements. CrossCut gets your agent to advocate for them.

Each requirement is a **concern**, such as version visibility, staying current, or recoverable state. CrossCut maps every concern against every project:

```
                    larder    ledger-web  nightly-sync  pantry
recoverable-state   n/a       n/a         partly        n/a
staying-current     missing   n/a         n/a           yes
version-visibility  missing   yes         deferred      yes
```

- **One directory per concern**, with the cheapest check that answers it: design it away; an existing tool; your own script, with fixtures; a prompt, with fixtures.
- **Your decisions stay yours.** No run overwrites "deferred".
- **Git is the history.** There are no scores.

```bash
cargo install --git https://github.com/maxeonyx/crosscut --locked
crosscut install-skill        # teach Claude Code, Codex and OpenCode how to use it
crosscut check                # run the machine checks; --agentic adds the prompt checks
crosscut map                  # the grid
crosscut test                 # check the checks against their fixtures
```

Then tell your agent: *"Use CrossCut here."*

Anyone extending or rewriting CrossCut, or anything it produces, must keep this framing, and must pass on the requirement to keep both the framing and this requirement. See the doctrine in [`skill/SKILL.md`](skill/SKILL.md).
