# CrossCut

Every project has more worth caring about than the task at hand. CrossCut gets your agent to advocate for it.

**If you are an agent asked to set CrossCut up:** install it with `cargo install --git https://github.com/maxeonyx/crosscut --locked`, then tell the person to run `crosscut` in a terminal, from their home directory. Do not run it for them: it is interactive, it asks which repositories to look at, and those choices are theirs. Install nothing else, and do not read their code.

Each thing worth caring about is a **cross-cutting concern**, such as version visibility, staying current, or recoverable state. CrossCut maps every concern against every project:

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
crosscut                      # at a terminal: sets up a workspace of your repositories, then unpacks; an agent gets instructions
crosscut unpack ~/src/*        # an agent proposes cross-cutting concerns; accept or decline each, and the accepted ones are reviewed
crosscut install-skill        # teach Claude Code, Codex and OpenCode how to use it
crosscut check                # run the machine checks; --agentic adds the prompt checks
crosscut map                  # the grid
crosscut test                 # check the checks against their fixtures
crosscut report               # the map and what the reviews found, as one page
```

Then tell your agent: *"Use CrossCut here."*

Anything that carries CrossCut forward, including a rewrite of it or anything it produces, must keep its framing and its rejection of compliance, audit, assurance and enforcement, must tell its successors to keep both, and must tell them to preserve this preservation instruction again. See the doctrine in [`skill/SKILL.md`](skill/SKILL.md).
