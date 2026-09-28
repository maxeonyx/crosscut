# Setup: "Use CrossCut here"

You are bringing CrossCut to a project or ecosystem, which may be brand new,
years old, or someone else's. The goal is a person who can see more than
before, plus the smallest persistent set of concerns worth keeping. The goal
is not a complete catalogue.

## 1. Find out what is actually there

Read before you theorize:
- the code and its layout;
- the manifests and lockfiles;
- CI and deploy configuration;
- the agent guidance and the README;
- recent history (`git log --stat` over the last few months shows where the
  effort goes);
- open issues and PRs, if they are cheap to reach.

Then run things: the setup command, the tests, `--help`. What happens when
you run it tells you more than reading about it.

Running has side effects:
- Test suites rewrite lockfiles and status files.
- They leave sessions, containers and daemons behind.
- They share state with the host: a tmux server, a local database, caches.

Prefer commands that only look. Before running anything heavier, note
`git status` in every repository it can touch, and afterwards put back
anything you changed. Whatever it disturbed is worth reporting too.

Work out what the unit is:
- one repository;
- several;
- a deployment of upstream software;
- a machine;
- durable data that outlives the code.

For a wrapper over many projects, write or update `crosscut/projects.md` as
you go. A project you cannot reach becomes a line saying so. It does not
stop the work.

If `crosscut/` already exists, read it first, and treat the existing
concerns as evidence rather than authority.

The place may already have its own concern system: a standards suite, a
review checklist, a dashboard, a set of CI gates. Treat that as evidence too:
- Its good mechanisms can become rungs in "How to look".
- What it cannot see tells you where to look.
- The system itself can be the subject of a concern: what does it cost, and
  what does it show?

## 2. Expand before narrowing

Read [../reservoirs.md](../reservoirs.md). Then write, in your working
notes, a reservoir specific to this place:
- who touches it, and at what moments;
- what it is made of;
- what it depends on;
- what decays;
- what would hurt to lose;
- what is done by hand;
- what repeats across siblings;
- what the last few months of commits reveal.

Ground every entry in something you observed. Keep going past the point of
feeling done, and stop when new entries are duplicates. Include a condensed
form of it in what you report at the end: a reservoir nobody sees tends not
to get written.

This is where the value comes from. Five generic entries means you have
satisfied the instruction without doing the work.

## 3. Judge applicability and importance

For each candidate, ask:
- Does it apply? Where?
- What concrete consequence is at stake, and for whom?
- How much does it matter, compared with the rest?
- What would better look like here?
- What do we know, and what is unknown?

A prototype and a service holding customer data should not carry the same
burden. For a young project, say which dimensions will start to matter and
when ("once it stores user data, recovery becomes the main question"), and
persist almost nothing yet.

For a mature project, separate what is load-bearing from historical residue.
Do not create a concern for every weakness you see.

## 4. Talk to the human, briefly

Bring your most surprising, concrete findings first. Ask only what software
evidence cannot tell you:
- Is this data precious?
- Who actually depends on this?
- Is this trade-off deliberate?
- What are you optimizing for?

Propose a small initial set of concerns: often two to five, and each should
be one you expect to want to re-ask. Say what you would *not* persist, and
why.

If nobody is available to answer, for example in a headless or delegated
run, establish only the set you would recommend. Keep it small, and put your
questions in the final report instead of guessing the answers.

## 5. Establish what they choose

For each chosen concern, follow [establish.md](establish.md). Create
`crosscut/README.md` from the template in
[../concern-files.md](../concern-files.md).

Optionally, add one line to the project's agent guidance so that ordinary
sessions notice the concerns, for example: "Cross-cutting concerns worth
keeping visible live in `crosscut/concerns/`; consider whether your change
touches one." Add it only if the human wants the concerns to be ambient.

## 6. Leave a clear picture

End with:
- what you found;
- what you persisted;
- what you noticed but deliberately left alone;
- what you could not see, and why.

That last list is often the most valuable part.

Some findings are one small, obvious fix with a lot of leverage, such as a
command in the guidance that does damage. Just report those clearly, or fix
them if that is in scope. A concern is for a question worth asking again,
not for a single defect.
