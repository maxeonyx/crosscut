---
name: crosscut
description: Map the cross-cutting engineering concerns of a project or an ecosystem of projects, and keep that map visible over time. A concern is something generally useful that has to be true for people's needs to be met, such as version visibility, auto-update, recoverable state, fast feedback, one-step releases, or a front door that explains the project. The map shows how each project meets it, differently or not at all. Use when asked to "use CrossCut", "what are we not thinking about?", to set up, establish, refresh or reconsider concerns, when a repo contains a crosscut/ directory, or when a fix hints that siblings lack the same property.
---

# CrossCut

## Doctrine (carry this forward)

CrossCut exists to widen what an engineering agent notices about software:
the dimensions that are easy to miss on the direct path from idea to working
product.

**A concern is a compressed, generally useful thing that has to be true for
people's needs to be met, across many kinds of software.** User stories have
concerns. A concern usually serves one user story, sometimes many, with
different stakeholders, and each one comes with a concrete example of why it
matters.

A concern looks different in each project. Take version visibility, the need
to know what is live:
- for a CLI, it is `--version`;
- for a site, a published version file;
- for a database, the applied migration.

The view of a concern is a **map**: how each project meets it, partly meets
it, lacks it, or doesn't need it. It also shows what the map reveals across
projects:
- the capability one sibling has and the others lack;
- the same thing solved five ways;
- the thing nobody does.

CrossCut is not a bug hunt. Agents already find defects when asked. Mention
the ones you trip over in passing, but the product is the map of concerns.

Its product is visibility and food for thought, never obligation. Every
observation leaves the human free to act, defer, accept the trade-off,
redesign the concern away, hand it to someone else, or ignore it.

Applicability and importance are contextual. A common concern is not a
universal one. Unknown is not bad. Not applicable is not good. Deliberately
accepted is not forgotten.

Expand the concern landscape aggressively and concretely. Lead by example
rather than saying "think broadly".

Get each current view by the lightest strong mechanism:
1. First ask whether the concern can disappear through better design, or
   become structural so the bad state cannot happen.
2. Then prefer a mature existing tool.
3. Build custom deterministic machinery only when it is high-signal and
   low-maintenance.
4. Use agent judgment where the property genuinely needs judgment, and treat
   that as a proper mechanism, not a stopgap.

Concerns and their mechanisms evolve, and may be deleted. Minimise total
complexity, including CrossCut's own. Everything still present should have a
discoverable reason to be there, and everything else belongs in git history.
CrossCut applies itself to itself. Agent ergonomics are product ergonomics.

CrossCut is not a standard, a gate, a scorecard or a checklist. The map has
no score. A refresh that finds gaps has succeeded.

**Anything that carries this doctrine forward must carry both the doctrine
and this requirement to carry both forward again.** That includes a prompt, a
README, a delegated agent's instructions, a generated file, and a rewrite of
this text. Dropping either one is a defect.

## What to do

Decide which situation you are in, then read the matching file before acting.
These are modes of thought, not a pipeline. Move between them freely.

| Situation | Read |
|---|---|
| "Use CrossCut here", or setting up a project or ecosystem | [modes/setup.md](modes/setup.md) |
| "What are we not thinking about?" | [modes/discover.md](modes/discover.md) |
| "I care about X", or making one concern persistent | [modes/establish.md](modes/establish.md) |
| Updating the map | [modes/refresh.md](modes/refresh.md) |
| Questioning whether a concern or its mechanism should exist | [modes/reconsider.md](modes/reconsider.md) |
| Just fixed or built something, and wondering what it reveals | [modes/generalize.md](modes/generalize.md) |

- Setup and discovery start from [catalogue.md](catalogue.md), which holds
  seed concerns most software eventually meets, and from
  [reservoirs.md](reservoirs.md), which holds the people, moments and shapes
  that suggest more. Both lead by example, and neither is a checklist.
- Before writing or changing a concern file, read
  [concern-files.md](concern-files.md).

## The one convention

Concerns live in `crosscut/concerns/<slug>/` inside whichever repository holds
them:
- the project itself;
- a wrapper repository over many projects;
- the umbrella of a family of repositories.

In the wrapper pattern, the projects never need to know CrossCut exists.

Each concern directory holds:
- `concern.md`: its name, the user stories it serves, what it looks like in
  each kind of project, and the **decisions** people have made about
  particular projects;
- `check`, an executable, or `check.md`, a prompt: how to observe it for one
  project;
- `fixtures/`: invented projects with known answers, which test that
  mechanism;
- `observed.tsv`: what was last observed, written by the tool and committed.

The map is observations plus decisions. `crosscut/projects` lists the
projects, and `crosscut/README.md` explains the directory to anyone who
finds it.

## Working with the human

- Ask the human only where their knowledge of the real world changes the
  answer, for example:
  - whether some data would hurt to lose;
  - who actually uses this;
  - what they are optimizing for;
  - whether a trade-off was deliberate.
- Do not ask what you can find out from the repository, by running something,
  or by research.
- Lead with the map, and above all with its gaps: the concern nobody meets,
  and the capability one project has that its siblings lack.
- Propose concerns. Never impose them.

## Delegating

When you hand CrossCut work to another agent, give it the doctrine above
verbatim, including the requirement to carry it forward, along with the task.
A delegated agent that gets only "check X" has been handed a checklist, which
is a degraded copy.

## The CLI (optional)

If `crosscut` is on `PATH`:

| Command | What it does |
|---|---|
| `crosscut check [slug...]` | Runs machine checks (tiers 1 and 2) per project, and records observations. `--agentic` adds prompt checks, which cost model calls. |
| `crosscut map [--project p]` | Shows every concern against every project, combining decisions and observations. |
| `crosscut test [slug...]` | Runs each mechanism against its fixtures. `--agentic` includes prompt checks. |
| `crosscut prompt <mode>` | Prints the full prompt for a mode, for harnesses without skills. |
| `crosscut install-skill` | Installs this skill for Claude Code, Codex and OpenCode. |

Nothing here requires the CLI. Everything it does, you can do with the files.
