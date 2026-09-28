# Reconsider: should this concern, or its mechanism, still exist?

Concerns and their mechanisms are software. They decay, they accumulate
residue, and they are sometimes replaced by something better. Reconsidering
is how CrossCut stays small.

For each concern, or for the whole set, ask:

- **Is the question still worth asking?** Architecture changes. If the
  durable state, the manual deploy, or the second tenant has gone, the
  concern may have gone with it. Delete it, and say why in the commit.
- **Could it disappear now?** A redesign that was too expensive a year ago
  may be cheap today.
- **Has part of it become structural or deterministic?** Suppose the last
  several views all reported the same mechanical fact. Hand that fact to an
  existing tool, or to the project's own CI, and keep only the judgment. A
  concern that has fully graduated into ordinary tooling can shrink to one
  line, or be deleted.
- **Has a deterministic mechanism become noise?** Scripts coupled to
  yesterday's layout, false alarms everyone ignores, maintenance with no
  signal: replace them with judgment, or with a better tool. Deleting a
  helper is an improvement.
- **Is there now a mature tool that answers it?** Use the tool, and delete
  the custom machinery.
- **Is the judgment prompt still good?** New models and harnesses change how
  prompts behave. A prompt that has decayed into generic "best practices"
  prose has stopped leading by example.
- **Are two concerns really one?** Is one concern really three? Merge or
  split them by the question, not by the mechanism.
- **Is the set as a whole the smallest that gives useful visibility?**

In a wrapper or a shared repository, propose changes to the question or to
"Why this matters here". The human owns what is worth asking. You can make
mechanical improvements to "How to look" directly, and explain them.

Apply the same questions to CrossCut itself: its prompts, its CLI, and its
own concerns. Check whether the framing has drifted towards standards, gates
or scores. Check whether any artifact passes on the doctrine without its
requirement to be carried forward again. Either one is a defect.
