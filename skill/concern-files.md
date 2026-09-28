# Concern files

A concern is a compressed, generally useful thing that has to be true for
people's needs to be met, such as version visibility, staying current, or
recoverable state. Each concern file names it, says which user stories it
serves, says what it looks like in different kinds of project, and keeps a
dated map of where each project stands.

## Where they live

```
crosscut/
  README.md          what this directory is (template below)
  projects.md        the projects the maps cover
  concerns/
    <slug>.md        one concern
    <slug>/          optional: helper scripts that only this concern uses
```

Put `crosscut/` in the repository that should own the concerns:
- the project itself, whose map has one row, or a row per component;
- a wrapper repository over many projects;
- the umbrella of related repositories.

`projects.md` gives one list item per project:

```markdown
- **name**: where it lives (a path, a remote, or both), what it is, and
  anything a refresh should know, such as "archived", "holds family photos",
  or "only Max uses it".
```

It starts with the name in bold. Map rows use the same names, and
`crosscut map` orders its columns by this list. A project that is not checked
out or not reachable becomes an `unknown` row, not an error.

## Shape of one file

Headings are conventions, not a schema. Keep them, because humans, agents,
`crosscut list` and `crosscut map` all use them to find their way.

```markdown
# Version visibility

Anyone who needs to know what is live can find out, at the moment they need
it.

## User stories

- **Max, mid-incident:** "is my fix live?" should take seconds, not an SSH
  session and a guess from file dates.
- **An agent reading a bug report:** which version is the reporter running?

## What it looks like here

- CLI tools: `--version`, and `--version --json` with the commit.
- Sites: a `version.json` at the site root, written by the deploy.
- The umbrella: `docs/version.json` naming every pinned tool.

## How to look

- `<bin> --version --json` for each tool; `curl -s https://<site>/version.json`.
- Judgment: is the version visible to whoever needs it, when they need it?

## Current view — 2026-09-28

Every tool except crosscut and agent-harness publishes both; they share one
copied mechanism.

| project | where it stands |
|---|---|
| trunc | yes: `--version --json`, and site `version.json` from the deploy |
| dotsync | yes: same mechanism as trunc |
| crosscut | missing: `--version` prints only the crate version, and the site has no version file |
| agent-harness | partly: `--version` exists, but there is no site version file |
| oc | n/a: archived; its frozen site still names 0.3.20 |

- **Across projects:** four copies of the same deploy step. A shared release
  workflow would give it to crosscut and agent-harness for free.
- **Since last map:** first map.
```

## Writing each part

**Name.** The H1 is the concern's name: short, and meaningful to someone who
has never seen the project. Under it, one sentence says what has to be true.
The slug is the short form of the name.

**User stories.** Say who needs this and what they are trying to do, with a
concrete example of it mattering. There is often one story, and sometimes
several with different stakeholders: the user, the on-call person, the next
agent, the person inheriting the project. The stories are how you judge
applicability. A project none of the stories touch is `n/a`.

**What it looks like here.** The same concern takes a different form in a
CLI, a site, a database, a library or a scheduled job. Say what it looks like
in the kinds of project this map covers. This section keeps a refresh from
checking for one particular file when the concern is really about a
capability.

**How to look.** This is the mechanism, and it should be the lightest strong
one:
1. Could the concern disappear? If a redesign would remove it, say so.
2. Is the good property structural? Then just confirm the structure still
   holds.
3. Is there a mature existing tool? Name the command.
4. Is there a small custom script? Keep it in `concerns/<slug>/`, and only
   while it is high-signal and cheap to maintain.
5. What still needs judgment? Write that part as a strong prompt.

Keep this section stable. A refresh does not rewrite it; reconsideration
does.

**Current view.** It is always the last section, headed
`## Current view — <date>`. Each refresh replaces it whole, and git keeps the
earlier maps.

1. **One headline sentence** saying what is most worth knowing now.
   `crosscut list` shows only this sentence.
2. **The map:** a table whose first column is headed `project`, with one row
   per project from `projects.md` that the concern could touch. The second
   cell starts with one of these words, followed by a colon and the
   specifics:
   - **yes**: meets it. Say how, because how is what siblings can copy.
   - **partly**: meets some of it. Say which part is missing.
   - **missing**: the stories apply and nothing meets them.
   - **n/a**: none of the stories apply here. Say why.
   - **deferred**: known, and deliberately not now. Say who decided, and
     why.
   - **unknown**: could not see. Say why: not checked out, no access, or a
     question only the human can answer.

   These words keep the distinctions that matter: `n/a` is not `yes`,
   `unknown` is not `missing`, and `deferred` is not forgotten. There is no
   score, and no total.
3. **Across projects:** what the map shows that no single row does:
   - one project has it and its siblings do not;
   - the same thing solved several ways;
   - nobody has it;
   - the lightest way to give it to all of them at once.

   This is usually the most valuable part.
4. **Since last map:** what changed. Read the previous map before replacing
   it.
5. **Noticed along the way:** only if a refresh touched a concern that is
   not represented yet. Defects you trip over can go here in one line each.
   They are not the product.

Keep it short. A map plus a few bullets is typical. If a view keeps growing
past a few hundred words, the concern is probably several concerns.

## The complexity tax

The complexity tax concern (see [catalogue.md](catalogue.md)) is a different
kind. Its map rows can say whether each project's reasons are discoverable,
but its substance is a list of specific things that no longer seem to earn
their place. Each one is a candidate for deletion, with the reason it seemed
to exist and why that reason looks gone.

## Deleting

When a concern stops earning attention, delete the file and its helper
directory, and say why in the commit message. Git remembers, so do not keep
an archive of dead concerns.

## `crosscut/README.md` template

Write this when you create the directory. Adapt the first paragraph to the
place, and keep the rest in substance. It is how the doctrine reaches people
and agents who have never heard of CrossCut, so its closing requirement is
part of what it must carry.

```markdown
# CrossCut concerns

Each file in `concerns/` is a cross-cutting concern of <this project / these
projects>: something generally useful that has to be true for people's needs
to be met, such as knowing what version is live, or staying current. Each
file says which user stories the concern serves, what it looks like here, how
to look, and gives a dated map of where each project in `projects.md` stands.
Git history holds the earlier maps.

These are visibility and food for thought, not obligations. Any gap can be
filled, deferred, accepted as a trade-off, designed away, or ignored. A map
that shows gaps is a successful map. Nothing here is a standard, a gate or a
score.

## The thinking behind them

- **Whether a concern applies, and how much it matters, depends on context.**
  Unknown is not bad. Not applicable is not good. Deferred is not forgotten.
- **The map is the point.** The most useful things it shows are a capability
  one project has and its siblings lack, the same thing solved several ways,
  and the thing nobody does. Propose new concerns when you find them.
- **Get each map the lightest strong way:**
  1. Make the concern disappear through design, or make the good property
     structural.
  2. Otherwise use a mature existing tool.
  3. Otherwise use a small script, but only if it is high-signal and
     low-maintenance.
  4. Use judgment where judgment is what the question needs.
- **Concerns and their mechanisms change, shrink and get deleted.** Anything
  still here should have a reason to be here.

## Refreshing

To refresh a concern, follow its "How to look" section and replace its
"Current view" section. Do it by hand or with a coding agent. The CrossCut
tool can do this headless (`crosscut refresh`) and show every map as one grid
(`crosscut map`), but nothing here depends on it.

Anyone or anything that changes, extends or regenerates these files must keep
this thinking and framing. It must also pass on the requirement to keep both
the thinking and framing and this requirement.
```
