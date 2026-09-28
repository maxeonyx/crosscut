# Refresh: update the current view

Refreshing means updating our understanding of a known concern. It does not
mean judging the project.

A refresh that finds eight of ten deployments cannot be recovered has
succeeded: it learned something. A refresh that cannot get enough evidence
has also succeeded, as long as it says clearly what is unknown and why.

## For each concern

1. Read the whole file.
   - The question and "Why this matters here" tell you what to care about.
   - "How to look" tells you how.
   - The previous "Current view" is your baseline.

   `git log -p -- <file>` shows how earlier views evolved, when that helps.
2. Follow "How to look".
   - Run the deterministic parts first. They are cheap and they ground the
     judgment.
   - Then do the judgment parts properly: read, run, try things, as a
     capable engineer would.
   - If a project is missing, or a credential or tool is unavailable, that
     part becomes a stated unknown. Do not stop, and do not guess.
3. Write a new `## Current view — <today>` section to replace the old one,
   following [../concern-files.md](../concern-files.md):
   - applicability and importance in context;
   - evidence, with how confident you are in it;
   - unknowns, and why each is unknown;
   - what is worth considering;
   - what changed since the last view;
   - anything noticed along the way.
4. Do not rewrite the question, "Why this matters here" or "How to look"
   during a refresh.
   - If "How to look" is stale (a command fails, a path moved, a better tool
     now exists), say so in the view.
   - If the whole concern looks wrong or obsolete, say that too.
   - Changing the definition is [reconsider.md](reconsider.md), and in a
     wrapper repository it is the human's call.

## Refreshing many concerns

- **Refresh each concern independently.** One concern failing to run must not
  stop, or colour, the others.
- **Delegating is fine.** See "Delegating" in SKILL.md, and hand each
  agent the concern file.
- **Afterwards, synthesize briefly, in chat or in the commit message:**
  - what changed;
  - what is newly worth attention;
  - anything noticed along the way that suggests a new concern.

The human decides what to do with any of it.

## Headless

`crosscut refresh` runs this mode through an installed coding harness, one
concern at a time. It asks the agent for the new view only, and writes that
into the file itself, so a headless agent never needs to edit anything.

"Never needs to" is not "cannot". The agent gets a shell so that it can run
"How to look" commands, and only Codex's read-only sandbox enforces the
request not to change anything. Treat concern files like scripts: refresh
only ones you trust.

Its exit status is about whether the refresh *ran*. It says nothing about
what the refresh found.
