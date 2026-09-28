# Reconsider: should this concern, or its mechanism, still exist?

Concerns and their mechanisms are software. They decay, they collect
residue, and they get overtaken by something better. Reconsidering is how
CrossCut stays small.

For each concern, or for the whole set, ask:

- **Is the question still worth asking?** If the architecture changed and
  the stories no longer apply anywhere, delete the directory, and say why in
  the commit.
- **Could it disappear now?** A shared change that was too expensive a year
  ago may be cheap today.
- **Is the mechanism on the lowest tier that works?**
  - A prompt check whose answers have settled into something mechanical can
    move down to a custom check, or to an existing tool.
  - A custom check that an existing tool now answers should be replaced by a
    few lines of glue around that tool.
  - A custom check that has become noise should be fixed, replaced, or moved
    up to a prompt. Signs of noise: `crosscut test` disagrees with its
    fixtures, it fails everywhere, or it reports `missing` on projects that
    plainly meet the concern.

  Moving tiers is an improvement, not an admission of failure.
- **Do the fixtures still represent reality?** Add a case for each misread
  you have seen, so it cannot come back unnoticed.
- **Are the decisions still true?** `*` in `crosscut map` marks decisions the
  latest observation contradicts. Stale decisions belong to the human, so
  ask.
- **Are two concerns really one?** Is one concern really three? Merge or
  split them by the need, not by the mechanism.
- **Is the whole set the smallest that gives useful visibility?**

In a shared repository, propose changes to a concern's name, stories or
decisions. The human owns what is worth asking. You can make mechanical
improvements to checks and fixtures directly, and explain them.

Apply the same questions to CrossCut itself: its prompts, its CLI, and its
own tests. Check whether the framing has drifted towards standards, gates or
scores. Check whether any artifact passes on the doctrine without its
requirement to be carried forward again. Either one is a defect.
