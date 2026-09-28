# Refresh: update the map

Refreshing means updating the map of a known concern. It does not mean
judging the projects. A map that shows six of eight projects missing
something has succeeded. So has one that could not see some projects, as
long as those rows say `unknown` and why.

## For each concern

1. **Read the whole file.**
   - The user stories and "What it looks like here" say what to look for in
     each kind of project.
   - "How to look" says how.
   - The previous map is your baseline.
2. **Go project by project** through `crosscut/projects.md`.
   - Run the cheap deterministic parts first.
   - Then do the judgment parts properly.
   - A project you cannot see is `unknown`, with the reason. Do not guess.
3. **Write a new `## Current view — <today>`** following
   [../concern-files.md](../concern-files.md):
   - a headline;
   - the map table;
   - what the map shows across projects;
   - what changed since the last map.
4. **Do not rewrite the concern's definition during a refresh.** If "How to
   look" is stale, or the concern looks wrongly framed, say so in the view.
   Changing the definition is [reconsider.md](reconsider.md).

## Refreshing many concerns

Refresh each concern independently: one that fails to run must not stop the
others. If you delegate, see "Delegating" in SKILL.md, and hand each agent
the concern file.

Afterwards, `crosscut map` shows the grid. Summarize what changed in it.

## Headless

`crosscut refresh` runs this mode through an installed coding harness. It
uses one agent per concern, six at a time by default (`--jobs`). It asks
each agent for the new view only, and writes that into the file itself.

"Never needs to write" is not "cannot write". The agent gets a shell so it
can run "How to look" commands, and only Codex's read-only sandbox enforces
the request not to change anything. Treat concern files like scripts:
refresh only ones you trust.

The exit status is about whether the refresh *ran*. It says nothing about
what the map shows.
