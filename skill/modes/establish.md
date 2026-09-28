# Establish: make one concern persistent

The human said something like "I care about knowing whether these deployments
can be recovered." Your job is to understand the concern properly *before*
adding anything.

## Understand the concern

- **What is the terminal concern?** Take "does an AGENTS.md exist?" It is
  standing in for "can a fresh agent get productive quickly, without being
  misled?" Keep going until you reach the consequence someone actually feels.
- **Where does it apply, and how strongly?** Which projects, deployments,
  accounts, or relationships?
- **What would better look like here?** Name concrete outcomes, not
  "follows best practices".
- **What would make us stop caring?** That tells you when to delete the
  concern later.

## Choose the mechanism: climb down the ladder

1. **Can the source of the concern disappear?** Remove the state, the manual
   step, or the workaround. If a redesign is the best answer, say so. Then
   the concern's view becomes "worth redesigning away", with the proposal.
2. **Can the good property be made structural?** Examples: generated
   metadata, one blessed command, an ownership boundary, a type. Then the
   view only confirms that the structure holds.
3. **Does a mature tool already answer it?** Name its command in
   "How to look". Do not wrap it in anything.
4. **Would a small custom script give high signal at low maintenance?**
   Build one only if it will still work after files move, rarely raises
   false alarms, and does not miss the important cases. If you find yourself
   writing regexes against today's directory layout, stop: an agent is
   probably the smaller and more accurate mechanism.
5. **What needs judgment?** Write that part of "How to look" as an excellent
   prompt:
   - what to read and what to run;
   - what a fresh, capable engineer would ask;
   - what counts as evidence;
   - what the known blind spots are.

   If a cheap model could do it, say so.

Most concerns end up mixed: a few deterministic facts gathered cheaply, then
judgment over them. Do not spend model calls rediscovering what `git`, `gh`
or a linter can report. Equally, do not spend engineering building machinery
for what a model judges well.

For every mechanism, think about:
- signal against noise;
- the false positives and false negatives that matter;
- cost in time, tokens and API calls;
- what it needs to run (tools, credentials, network);
- how it fails partway;
- whether someone could mistake "the mechanism ran" for "the project is
  good".

## Write it and look once

Create `crosscut/concerns/<slug>.md` following
[../concern-files.md](../concern-files.md). Then do the first refresh
yourself, as in [refresh.md](refresh.md), so that the file ends with a real
current view.

The first view often shows that the concern was framed wrong. If so, fix the
framing now. It is cheap now and expensive later.

Adding a concern should touch that one file, plus a helper directory only
when a script genuinely earns it. If it needs more than that, something is
wrong with the representation. Tell the human.
