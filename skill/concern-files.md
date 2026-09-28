# Concern files

A concern is a persistent engineering question that we have decided is
useful to be able to ask again. The question matters more than whatever
currently answers it.

## Where they live

```
crosscut/
  README.md          what this directory is (template below)
  projects.md        optional: the projects a wrapper looks at
  concerns/
    <slug>.md        one concern
    <slug>/          optional: helper scripts that only this concern uses
```

Put `crosscut/` in the repository that should own the concerns:
- the project itself;
- a wrapper repository over many projects;
- the umbrella of related repositories.

A concern in a wrapper names its own scope in prose. It might cover one
project, several, a deployment, a machine, an account, or the relationship
between two repositories.

`projects.md` is prose plus locations, one short entry per project: where it
lives (a local path, a remote, or both), what it is, and anything a refresh
should know, such as "holds durable family photos" or "deployed on the home
server". Leave it out when the containing repository, with its submodules,
is the whole world. A project that is not checked out or not reachable is a
fact to report in a view, not an error.

## Shape of one file

Headings are conventions, not a schema. Keep them because humans, agents and
`crosscut list` use them to find their way.

```markdown
# Can we tell which version of each service is actually running?

## Why this matters here

Three services deploy from main by hand. When something breaks, the first
question is whether the fix is live, and today the answer means SSHing in and
guessing from file dates. This applies strongly to `billing-api`, which has
customers, and weakly to `docs-site`. Better means one command, or one URL,
that names the running commit for each service.

## How to look

- `curl -s https://billing.example.com/version` shows what is running there
  (deterministic).
- Compare the running commit with `git log origin/main -1` in each repo.
- Judgment: is the version visible to the person who needs it, at the moment
  they need it? A version string buried in a log file does not count.
- Not in scope: local development builds.

## Current view — 2026-09-28

Applies strongly to billing-api; the others are low stakes.

- **billing-api:** no version endpoint. The deploy script copies files
  without recording a commit. Unknown what is running (high confidence that
  it is unknowable today).
- **docs-site:** the build embeds the commit in the footer. Good, and cheap
  to keep that way.
- **Worth considering:** make the deploy script write the commit into the
  artifact. Then this question answers itself, and this concern could shrink
  to a single `curl`.
- **Since last view:** first view.
- **Noticed along the way:** the deploy script also runs migrations with no
  way to roll them back. That may deserve its own concern.
```

## Writing each part

**Title.** The H1 is the question, phrased so that someone who has never seen
the project understands what we want to keep visible. The slug is a short
name for it.

**Why this matters here.**
- Where the concern applies, and how strongly.
- What concrete consequence it protects or improves.
- What "better" would mean in this context.

The same question matters differently to a throwaway script and to a service
holding customer data. Say which one this is.

**How to look.** This is the mechanism. Order it by the ladder, and say
honestly which rungs are in use:
1. Could the concern disappear? If a redesign would remove it, say so.
2. Is the good property structural? Then just confirm the structure still
   holds.
3. Is there a mature existing tool, such as `cargo audit`, `gh api`,
   `restic check`, a linter, or the platform's own status page? Name the
   command.
4. Is there a small custom script? Keep it next to the concern in
   `concerns/<slug>/`. Keep it only while it is high-signal and cheap to
   maintain.
5. What still needs judgment? Write that part as a strong prompt: what to
   read, what to try, what a fresh, capable engineer would ask.

Keep this section stable. A refresh does not rewrite it. Reconsideration
does.

**Current view.** Always the last section, headed `## Current view — <date>`.
Each refresh replaces it whole, and git keeps the earlier views. Useful
things to say:
- **Applicability and importance, briefly and in context.** Use words that
  keep distinctions:
  - applies strongly / weakly / probably not;
  - unknown;
  - good in one place and weak in another;
  - handled structurally;
  - fine but easy to regress;
  - deliberately accepted;
  - worth redesigning away;
  - opportunity found.
- **Evidence, and how confident you are in it.** Distinguish what you
  observed from what you inferred.
- **What is unknown, and why.** Name the missing access, the checkout that
  was not present, or the question only the human can answer.
- **Worth considering:** suggestions with their leverage, never demands.
- **Since last view:** what changed. Read the previous view before you
  replace it.
- **Noticed along the way:** dimensions this refresh touched that no concern
  represents yet. Leave this out when there are none. Do not invent them.

A view should be readable in a minute or two:
- Lead with a one-line headline that says what is most worth knowing now.
  "Applies strongly" on its own tells the reader nothing, and `crosscut list`
  shows only this first sentence.
- Keep the evidence that supports the conclusions, and leave the working
  out.
- A few hundred words is typical. If a view keeps growing past that, the
  concern is probably several concerns. Or the investigation belongs in a
  report or commit message, and the view needs only its conclusions.

Never use pass/fail, scores or grades. If someone later wants a gate on one
concern, that is their choice, built on top of CrossCut. It is not
CrossCut's worldview.

## Deleting

When a concern stops earning attention, delete the file and its helper
directory. Say why in the commit message. Git remembers, so do not keep an
archive of dead concerns.

## `crosscut/README.md` template

Write this when you create the directory. Adapt the first paragraph to the
place, and keep the rest in substance. It is how the doctrine reaches people
and agents who have never heard of CrossCut, so its closing requirement is
part of what it must carry.

```markdown
# CrossCut concerns

Each file in `concerns/` is an engineering question we decided is worth being
able to ask again about <this project / these projects>: why it matters here,
how to get a current view, and the latest dated view. Git history holds the
earlier views.

These are visibility and food for thought, not obligations. Any of them can
be acted on, deferred, accepted as a trade-off, designed away, or ignored. A
view that reports bad news is a successful view. Nothing here is a standard,
a gate or a score.

## The thinking behind them

- **Whether a question applies, and how much it matters, depends on context.**
  Unknown is not bad. Not applicable is not good. Accepted is not forgotten.
- **Look for what nobody is asking yet.** Look concretely, across people,
  moments, projects and time. Propose new concerns when you find them.
- **Get each view the lightest strong way:**
  1. Make the concern disappear through design, or make the good property
     structural.
  2. Otherwise use a mature existing tool.
  3. Otherwise use a small script, but only if it is high-signal and
     low-maintenance.
  4. Use judgment where judgment is what the question needs.
- **Concerns and their mechanisms change, shrink and get deleted.** Keep the
  whole set as small as the visibility it buys allows.

## Refreshing

To refresh a concern, follow its "How to look" section and replace its
"Current view" section. Do it by hand or with a coding agent. The CrossCut
tool (`crosscut refresh`) can do this headless, but nothing here depends on
it.

Anyone or anything that changes, extends or regenerates these files must keep
this thinking and framing. It must also pass on the requirement to keep both
the thinking and framing and this requirement.
```
