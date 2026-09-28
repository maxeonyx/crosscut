# Generalize: what does this piece of work reveal?

You have just fixed a bug, built a feature, or changed CI. Before stopping,
spend a minute on the questions below. The answer is often "nothing": say so
in one line and move on. This must never turn every patch into process.

- **Which concern did this touch?** A fix is usually one project gaining, or
  regaining, a property: failing safely, explaining itself, knowing what is
  live. Name the property, not the bug.
- **What do the siblings look like on that concern?** If one tool needed the
  fix, do the others have the property, lack it, or have it in a different
  way? `crosscut map` shows the row at once. If the concern is not on the map yet, look quickly across the projects.
- **Could the property become structural for all of them at once?** Examples
  are a shared library, one reusable workflow, or a type.
- **Is the concern on the map?** If `crosscut/concerns/` has it, run
  `crosscut check <slug>` so that the map records what this work changed,
  and commit that with the work. If not, and it would be worth watching,
  propose it (see [establish.md](establish.md)).

If something genuinely generalizes, tell the human in two or three
sentences, with the sibling rows you looked at, and offer the next step. Do
not do the broader work unasked.
