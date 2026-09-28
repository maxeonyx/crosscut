# Refresh: update the map

Refreshing updates the observations. It does not judge the projects. A map
that shows six of eight projects missing something has succeeded. So has one
where a cell says `unknown: not checked out`.

## Steps

1. **Run the machine checks.** `crosscut check` runs every tier 1 and 2
   mechanism for every project in `crosscut/projects`. It is cheap, needs no
   model, and is fine to run whenever you like.
2. **Decide whether to spend model calls.** `crosscut check --agentic` also
   runs the prompt checks, one agent per cell. Scope it with a concern slug,
   or `--project`, when you only need part of the map.
3. **Read what changed.** `crosscut check` prints each cell whose status
   moved, and `git diff crosscut/` shows the rest. Then read `crosscut map`.
4. **Treat failures as information about the mechanism, not the project.**
   A cell that could not run keeps its previous row, and the failure is
   printed.
   - If a check fails everywhere, it is probably broken. Run
     `crosscut test <slug>`, and see [reconsider.md](reconsider.md).
   - If a decision is marked `*`, the world has moved away from what someone
     decided. Tell the human.
5. **Commit `observed.tsv`** together with whatever work changed it.

Do not edit `observed.tsv` by hand, and do not change decisions during a
refresh. Propose decision changes to the human instead.

## Without the tool

Follow each concern's `check` or `check.md` for each project, and update
`observed.tsv` by hand:
- one sorted row per project;
- `since` changes only when the status changes.
