# CrossCut concerns

Each file in `concerns/` is an engineering question we decided is worth being
able to ask again about CrossCut itself (this repository: the skill in
`skill/`, the `crosscut` binary, and how both reach agents in other
projects): why it matters here, how to get a current view, and the latest
dated view. Git history holds the earlier views.

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
