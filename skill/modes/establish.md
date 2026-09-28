# Establish: make one concern persistent

The human said something like "I care about knowing whether these can be
recovered", or you proposed a concern and they agreed. Understand the concern
properly before adding anything.

## Understand the concern

- **Name the concern, not a mechanism.** "Version visibility", not "has a
  version.json". The concern is the need, and the file is one way a site can
  meet it.
- **Write the user stories.** Who needs this, doing what? Give a concrete
  example of it mattering. There is usually one story; add others when
  different stakeholders need different things from it.
- **Say what it looks like in each kind of project.** How does a CLI meet
  it? A site? A library? A database? This is what lets one concern span a
  varied ecosystem.
- **Say what would make us stop caring.** That tells you when to delete it.

## Choose the mechanism: climb down the ladder

1. **Can the concern disappear?** For example, remove the state or the
   manual step.
2. **Can it become structural?** A shared release workflow that gives every
   tool a version file fills a whole row of the map, and stays filled.
3. **Does a mature tool already answer it?** Name the command.
4. **Would a small custom script give high signal at low maintenance?** Keep
   it in `concerns/<slug>/`. Regexes against today's layout are a sign that
   an agent is the better mechanism.
5. **What needs judgment?** Write that part of "How to look" as an excellent
   prompt.

Most concerns are mixed: a few cheap facts per project, then judgment over
them. Do not spend model calls on what `git`, `gh` or `curl` can report, and
do not build machinery for what a model judges well.

## Write it and map it once

Create `crosscut/concerns/<slug>.md` following
[../concern-files.md](../concern-files.md). Then do the first refresh
yourself, as in [refresh.md](refresh.md), so that the file ends with a real
map.

The first map often shows that the concern was framed wrong: too narrow, too
broad, or tied to one mechanism. If so, fix the framing now. It is cheap now
and expensive later.
