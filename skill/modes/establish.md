# Establish: make one concern persistent

The human said something like "I want every tool to stay current", or you
proposed a concern and they agreed. Understand the concern before you build
anything, then give it the cheapest mechanism that genuinely answers it, and
test that mechanism.

## 1. Understand the concern

- **Name the concern, not a mechanism.** "Version visibility", not "has a
  version.json". The concern is the need, and the file is one way a site can
  meet it.
- **Write the user stories.** Who needs this, doing what? Give a concrete
  example of it mattering. Add stories for other stakeholders when they need
  something different from it.
- **Say what it looks like in each kind of project here**: CLI, site,
  library, job, deployment.
- **Say what would make us stop caring.** That tells you when to delete it.

Write `concern.md` (see [../concern-files.md](../concern-files.md)).

## 2. Choose the mechanism: the lowest tier that works

- **Tier 0: can the concern disappear, or become structural?** If one shared
  change would make the property true everywhere, and keep it true, propose
  that change. The check that follows it can be trivial.
- **Tier 1: does a mature tool already answer it?** For example `gh`,
  `curl`, `cargo metadata`, a linter, or a package auditor. Write `check`
  as a few lines of glue that turn its answer into `<status>: <evidence>`.
  This is the default whenever it genuinely works.
- **Tier 2: is it mechanical, but nothing answers it?** Write a custom
  `check`. The bar is high:
  - every rule it applies gets a fixture;
  - it must still work after files move;
  - it must rarely raise false alarms;
  - it must not miss the cases that matter.

  If you find yourself matching patterns against today's layout, stop: the
  concern probably needs tier 3.
- **Tier 3: does it really take judgment?** Write `check.md`: the
  instruction an agent follows for one project. Say what to read, what to
  try, what counts as evidence, and when to answer `unknown`. A cheap model
  often suffices, so say so.

A tier 3 check whose answers have become predictable is a candidate to move
down to tier 1 or 2 (see [reconsider.md](reconsider.md)).

## 3. Write fixtures, and make the mechanism pass them

Create `fixtures/<case>/`: small invented projects, one per rule or tricky
kind, and an `expect` file of `<project> <status>` lines. Include:
- a project that meets the concern;
- one that does not;
- one the concern does not apply to;
- one that looks right but is not.

Run `crosscut test <slug>`, adding `--agentic` for a prompt check, which
costs model calls. A mechanism that disagrees with its own fixtures is not
finished. Either the mechanism or the fixture is wrong: decide which.

## 4. Observe, and read the map

Run `crosscut check <slug>`, adding `--agentic` for a prompt check, then run
`crosscut map`.

The first map often shows that the concern was framed wrong: too narrow,
too broad, or tied to one mechanism. Fix the framing now, while it is cheap.

Propose decisions for the rows that need one, such as `n/a` with its
reason, but let the human confirm them. Commit `concern.md`, the mechanism,
the fixtures and `observed.tsv` together.
