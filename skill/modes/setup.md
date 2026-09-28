# Setup: "Use CrossCut here"

You are bringing CrossCut to a project or an ecosystem of projects: new, old,
or someone else's. The goal is a map. It should show which cross-cutting
concerns matter here, how each project meets them, and above all where the
gaps are. Keep the persistent set small enough to stay alive.

This is not a bug hunt. You will trip over defects: mention them in one
line and move on.

## 1. Find out what is actually there

List the projects and write `crosscut/projects.md` (see
[../concern-files.md](../concern-files.md)). For each project, work out:
- what kind of thing it is: a CLI, a site, a service, a library, a
  deployment of upstream software, or a scheduled job;
- who uses it;
- what state it holds;
- how it ships.

A project you cannot reach becomes a line saying so.

Read the manifests, CI and deploy configuration, READMEs and agent guidance,
and recent history. Run cheap commands that only look, such as `--help`,
`--version` and a site's version file, because what actually happens
matters.

Running has side effects:
- Test suites rewrite lockfiles and leave sessions and daemons behind.
- They also share state with the host.

Before running anything heavier, note `git status` everywhere it could touch,
and put back anything you change.

If `crosscut/` already exists, read it, and treat it as evidence rather than
authority. The same goes for any existing concern system, such as a
standards suite, a checklist or a set of CI gates:
- What each one looks at is usually a good concern.
- The machinery around it is not the point. What it costs and what it shows
  can themselves be a concern.

## 2. Build the candidate map

1. **Start from [../catalogue.md](../catalogue.md).** For each seed, ask
   whether any of its user stories touch these projects, and what it would
   look like here.
2. **Go well beyond the catalogue.** [../reservoirs.md](../reservoirs.md)
   has people, moments and shapes that suggest concerns the catalogue does
   not list. The strongest source is the ecosystem itself:
   - **A capability one project has and its siblings lack.** One tool
     auto-updates and the rest don't. One site publishes its version and the
     others don't. Each of these is a concern waiting to be named.
   - **The same thing done several ways** across projects.
   - **Something every project needs and none does.**
3. **Sketch a rough map per candidate:** each project as yes, partly,
   missing, n/a, deferred or unknown, with a few words of how.

Write this candidate grid in your notes, and keep adding to it until new
candidates are duplicates.

## 3. Judge which concerns are worth keeping

A concern is worth persisting when you expect to want its map again: when
projects will be added, when things drift, or when a gap is worth watching
even if nobody fills it yet.

A capability that every project already meets structurally, and cannot
regress, may need only one line in your report. A prototype and a service
holding customer data carry different burdens, and the stories decide which
applies where.

## 4. Talk to the human, briefly

Lead with the grid and its gaps:
- the concerns nobody meets;
- the capabilities one sibling has and the others lack;
- the places where one shared change would fill several rows.

Ask only what the software cannot tell you:
- Who actually depends on this?
- Would this data hurt to lose?
- Is this gap deliberate?

Propose the concerns to persist, often more than a handful, because each one
is small. Say which candidates you would leave out, and why.

If nobody is available to answer, for example in a headless or delegated
run, establish the set you would recommend, and put your questions in your
final report.

## 5. Establish them

For each chosen concern, follow [establish.md](establish.md). Create
`crosscut/README.md` from the template in
[../concern-files.md](../concern-files.md).

Optionally, add one line to each project's agent guidance so that ordinary
sessions notice the concerns. Do this only if the human wants that.

## 6. Leave a clear picture

End with:
- the grid (`crosscut map`, if the CLI is available);
- its three or four most interesting gaps, and the lightest way to fill each;
- what you persisted, and what you deliberately left out;
- what you could not see.
