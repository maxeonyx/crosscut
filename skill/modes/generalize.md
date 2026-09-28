# Generalize: what does this piece of work reveal?

You have just fixed a bug, built a feature, or changed CI. Before stopping,
spend a few minutes on the questions below. The answer is often "nothing":
say so in one line and move on. This must never turn every patch into
process.

- **What invariant was missing?** State it in one sentence.
- **Can the same failure happen elsewhere?** Search: in this repository, in
  sibling projects, and in code that was copied from here. A quick grep or a
  structural search is cheap.
- **Is there a structural fix that removes the whole class?** Examples: a
  type, a shared helper, a constraint, deleting the duplicated code.
- **Should several projects get the same improvement?** Where would shared
  code help? Where would it create coupling that hurts?
- **Did the work reveal a dimension nobody is watching?** Examples: the bug
  was only found because a customer complained, the fix could not be
  verified in CI, or nobody could tell whether the fix was deployed.

If something genuinely generalizes, tell the human in two or three concrete
sentences, and offer the next step:
- fix the siblings;
- make the property structural;
- propose a persistent concern (see [establish.md](establish.md));
- if `crosscut/concerns/` exists and one of them is touched, note it in that
  concern's next view.

Do not do the broader work unasked. The human may have good reasons to keep
the change narrow.
