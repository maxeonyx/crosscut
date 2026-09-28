---
name: crosscut
description: Widen what you notice about a software project or ecosystem. Surface the cross-cutting engineering concerns the direct path misses (recovery, deployability, version visibility, agent guidance, CI, authority boundaries, the same weakness in sibling projects, and many more), and keep the worthwhile ones visible over time as plain Markdown concern files. Use when asked to "use CrossCut", "what are we not thinking about?", to set up, establish, refresh or reconsider concerns, when a repo contains a crosscut/ directory, or when a fix looks like it may reveal a broader weakness.
---

# CrossCut

## Doctrine (carry this forward)

CrossCut exists to widen what an engineering agent notices about software:
the dimensions that are easy to miss on the direct path from idea to working
product.

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
complexity, including CrossCut's own. CrossCut applies itself to itself.
Agent ergonomics are product ergonomics.

CrossCut is not a standard, a gate, a scorecard or a checklist. A refresh
that finds bad news has succeeded.

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
| Updating the current view of known concerns | [modes/refresh.md](modes/refresh.md) |
| Questioning whether a concern or its mechanism should exist | [modes/reconsider.md](modes/reconsider.md) |
| Just fixed or built something, and wondering what it reveals | [modes/generalize.md](modes/generalize.md) |

- Setup and discovery need breadth. Read
  [reservoirs.md](reservoirs.md) as well: its examples show the range
  expected, and they are not a checklist.
- Before writing or changing a concern file, read
  [concern-files.md](concern-files.md).

## The one convention

Concerns live in `crosscut/concerns/<slug>.md` inside whichever repository
holds them:
- the project itself;
- a wrapper repository over many projects;
- the umbrella of a family of repositories.

Each concern file is a question worth asking again, why it matters here, how
to get a current view, and the latest dated view. The file is at once the
definition, the refresh prompt, and the latest result, and git history holds
the rest. `crosscut/README.md` explains the directory to anyone who finds it.
An optional `crosscut/projects.md` lists the projects a wrapper looks at.

Target projects never need to know CrossCut exists.

## Working with the human

- Ask the human only where their knowledge of the real world changes the
  answer, for example:
  - whether some data would hurt to lose;
  - who actually uses this;
  - what they are optimizing for;
  - whether a trade-off was deliberate.
- Do not ask what you can find out from the repository, by running something,
  or by research.
- Lead with concrete, project-specific findings.
- Propose concerns. Never impose them.
- Some observations are worth one sentence in chat and nothing more. Make
  that the common case.

## Delegating

When you hand CrossCut work to another agent, give it the doctrine above
verbatim, including the requirement to carry it forward, along with the task.
A delegated agent that gets only "check X" has been handed a checklist, which
is a degraded copy.

## The CLI (optional)

If `crosscut` is on `PATH`:

| Command | What it does |
|---|---|
| `crosscut list` | Shows concerns and the date of each view. |
| `crosscut prompt <mode>` | Prints the full prompt for a mode, for harnesses without skills. |
| `crosscut refresh [slug...]` | Refreshes views headless through an installed coding harness. |
| `crosscut install-skill` | Installs this skill for Claude Code, Codex and OpenCode. |

Nothing here requires the CLI. Everything it does, you can do with the files.
