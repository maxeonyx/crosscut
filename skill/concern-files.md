# Concern files

A concern is a compressed, generally useful thing that has to be true for
people's needs to be met, such as version visibility, staying current, or
recoverable state. CrossCut keeps two kinds of thing about it apart:
- **what is observed**, which a mechanism writes and anyone can regenerate;
- **what is decided**, which people write and no run touches.

The map is computed from both.

## Layout

```
crosscut/
  README.md                what this directory is (template below)
  projects                 the projects the map covers
  concerns/
    <slug>/
      concern.md           the definition and its decisions. Written by people and agents.
      check                the mechanism, if a machine can answer (tiers 1 and 2)
      check.md             ...or a prompt, if it takes judgment (tier 3)
      fixtures/<case>/     invented projects, and an `expect` file
      observed.tsv         written by `crosscut check`. Commit it; never edit it by hand.
```

`projects` lists one path or glob per line, relative to the directory that
holds `crosscut/`, for example `tools/*`. Lines starting with `#` are
comments. A project's name is its directory name. In a wrapper over many
repositories, the projects are its checkouts or submodules, and they never
need to know CrossCut exists.

## `concern.md`

```markdown
# Version visibility

Anyone who needs to know what is live can find out, at the moment they need it.

## User stories

- **The on-call engineer, mid-incident:** "is the fix live?" answered in seconds.

## What it looks like here

- CLIs: `--version`. Sites: a `version.json` naming the commit. Jobs: a version line in their log.

## Decisions

- **nightly-sync**: deferred: being retired once the ledger's own backups land (lead, 2026-09-01).
```

- **Name.** The H1 is the concern's name: short, and meaningful to a
  stranger. The sentence under it says what has to be true. The slug is the
  directory name.
- **User stories.** Say who needs this and what they are trying to do, with
  a concrete example. There is often one story, and sometimes several with
  different stakeholders. The stories are how you judge whether a project is
  `n/a`.
- **What it looks like here.** The same concern looks different in a CLI, a
  site, a database, a library or a job. Say how, so that a mechanism checks
  for the capability, not for one particular file.
- **Decisions.** Write each as `- **project**: status: reason (who,
  when)`, for example:
  - `n/a` when none of the stories apply;
  - `deferred` for known, and deliberately not now;
  - more rarely, `yes` or `missing` to overrule a mechanism that is
    misreading a project.

  Only people make decisions. An agent may propose one, but the human
  confirms it. On the map, a decision takes precedence over an observation,
  and `crosscut map` marks it with `*` when the latest observation
  contradicts it. `deferred` exists only as a decision, which is how "not
  managed" stays a human act.

## The mechanism: choose the lowest tier that works

Every mechanism answers the same question: **for this one project, what is
its status, and what is the evidence?** It answers with one line:

```
<status>: <evidence>          status is yes, partly, missing, n/a or unknown
```

Use `unknown: <why>` when the mechanism cannot tell, for example when a
project is not checked out or a tool is not logged in. That is an honest
answer. A mechanism that crashes, times out or prints anything else is a
broken mechanism. CrossCut keeps the previous row and reports the failure.

**Tier 0 is a question, not a mechanism.** Before writing any check, ask
whether the concern could disappear through design, or whether the good
property could become structural. An example is one shared release workflow
that gives every tool a version file.

**Tier 1: an off-the-shelf tool does the judging.** `check` is a few lines of
glue around a mature program such as `gh`, `curl`, `cargo metadata`, a
linter or a package auditor. The program knows the answer, and the glue only
translates it into a status line. Prefer this whenever it genuinely answers
the question.

**Tier 2: a custom check.** When no existing tool answers the question but
the property really is mechanical, write `check` yourself. The bar is high,
because this is code you now maintain:
- It needs fixtures (below) that cover every rule it applies, with at least
  one true positive and one true negative.
- It is software, so it sits on the map too: its tests, and whether it
  still earns its place.
- Rules keyed to today's file layout, or a growing pile of patterns, are a
  sign that the concern needs judgment instead.
- A check that turns noisy should be replaced, or moved up to tier 3.

**Tier 3: a prompt check.** When the property really takes judgment, write
`check.md` instead. It is the instruction a coding agent follows for one
project: what to read, what to try, what counts as evidence, and when to say
`unknown`.
- CrossCut runs one agent per cell, and passes the doctrine, the concern,
  and the project's siblings with it.
- A prompt check is easy to write, and just as much in need of fixtures.
  `crosscut test --agentic` runs them, at the cost of model calls.

A check runs in the project's directory, with these in its environment:
- `CROSSCUT_PROJECT`, the project's name;
- `CROSSCUT_PROJECT_DIR`, its directory;
- `CROSSCUT_PROJECTS`, one `name<TAB>dir` line per sibling, for concerns
  that compare projects;
- `CROSSCUT_CONCERN_DIR`, so a check can find files kept next to it.

Checks are trusted like any script in the repository, so run only ones you
trust.

## Fixtures

```
fixtures/<case>/<project>/...     one small invented project per directory
fixtures/<case>/expect            one `<project> <status>` line per project
```

A case is a tiny invented ecosystem. Make each project the smallest thing
that exercises one rule, such as a site with a version file, or one without.
Include the tricky kinds: a project the concern does not apply to, and
something that looks right but is not. `crosscut test` runs each mechanism
against its fixtures and reports every disagreement.

## `observed.tsv`

This file is written by `crosscut check`, and is never edited by hand. It
has one row per project, sorted: `project, status, since, evidence`.
- `since` moves only when the status changes.
- A prompt check whose status is unchanged keeps its old wording.

So rerunning an unchanged world produces no diff, and the file's git
history *is* the history of the map. Commit it together with the work that
changed it, so that a reviewer sees, for example, `larder missing → yes`
next to the code that did it.

## Deleting

When a concern stops earning attention, delete its directory, and say why in
the commit message. Git remembers, so do not keep an archive of dead
concerns.

## `crosscut/README.md` template

Write this when you create the directory. Adapt the first paragraph to the
place, and keep the rest in substance. It is how the doctrine reaches people
and agents who have never heard of CrossCut, so its closing requirement is
part of what it must carry.

```markdown
# CrossCut concerns

Each directory in `concerns/` is a cross-cutting concern of <this project /
these projects>: something generally useful that has to be true for people's
needs to be met, such as knowing what version is live, or staying current.
`concern.md` says which user stories it serves, what it looks like here, and
what people have decided about it. `check` or `check.md` is how to observe
it for one project. `observed.tsv` holds the latest observations, and its
git history is the history of the map. `projects` lists the projects.

These are visibility and food for thought, not obligations. Any gap can be
filled, deferred, accepted as a trade-off, designed away, or ignored. A map
that shows gaps is a successful map. Nothing here is a standard, a gate or a
score.

## The thinking behind them

- **Whether a concern applies, and how much it matters, depends on context.**
  Unknown is not bad. Not applicable is not good. Deferred is not forgotten.
- **The map is the point.** The most useful things it shows are a capability
  one project has and its siblings lack, the same thing solved several ways,
  and the thing nobody does.
- **Observe each concern the lightest strong way:**
  1. Make it disappear through design, or make the good property structural.
  2. Otherwise use a mature existing tool.
  3. Otherwise use custom code, but only if it is high-signal,
     low-maintenance and tested against fixtures.
  4. Use a judgment prompt where judgment is what the question needs, and
     test it too.
- **Concerns and their mechanisms change, shrink and get deleted.** Anything
  still here should have a reason to be here.

## Running it

Without any tool, follow a concern's `check` or `check.md` for each project
and update `observed.tsv` by hand. The CrossCut tool does it for you:
`crosscut check` runs the machine checks, `--agentic` adds the prompt checks,
`crosscut map` shows the grid, and `crosscut test` checks the checks.

Anyone or anything that changes, extends or regenerates these files must keep
this thinking and framing, and its rejection of compliance, audit, assurance
and enforcement. It must tell its successors to keep both, and to preserve
this preservation instruction again.
```
