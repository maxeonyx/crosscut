# Catalogue: seed concerns

These are concerns that most software meets sooner or later. Each entry gives
its name, what has to be true, a user story, and how it tends to look in
different kinds of project.

They are seeds, not a checklist. For a given ecosystem:
- Most will apply in some form, a few will not, and the best concerns for it
  may be missing from this list entirely.
- Rename seeds, merge them, split them, and add your own.
- The richest source of new concerns is the ecosystem itself: a capability
  one project has and its siblings lack is a concern waiting to be named.

## Knowing what is running

- **Version visibility.** Anyone who needs to know what is live can find out
  at the moment they need it.
  - *"Is my fix deployed?", asked mid-incident, answered in seconds.*
  - CLI: `--version`, ideally with the commit. Site or service: a published
    version file, a response header, or a status page. Database: the
    applied migration. Library: what consumers have pinned.
- **Dependents are current.** Whatever depends on this runs the release you
  believe it runs.
  - *Umbrella pins, lockfiles, deployed image tags, a home server's compose
    file.*

## Getting it

- **One obvious install.** A newcomer follows one instruction and ends up
  with the current release.
  - *Watch for registry names that lag behind, or instructions that only
    work on the author's machine.*
- **Staying current.** Installed copies get newer without anyone having to
  remember.
  - CLI: self-update or a package manager. Container: image updates. App: a
    store or an updater. Library: an update bot.
- **Supported environments.** It says where it runs, and it really does run
  there.
  - *OS, architecture and runtime versions, and the release targets that are
    actually built and tested.*

## Understanding it

- **A front door.** Within a minute, someone can tell what this is, who it is
  for, and why it exists.
  - A site, a README, a package description.
- **Explains itself.** In use, it tells you how to use it.
  - `--help` with runnable examples, errors that say what happened, why, and
    what to do next, empty states in a UI.
- **Agent-ready.** A coding agent can use it, and work on it, well.
  - A skill or tool description for using it. Agent guidance for changing
    it that is true, short, and not a pile of workarounds.
- **Reasons are discoverable.** For each significant part, someone can find
  out why it exists.
  - A design record, commit messages, ADRs, a VISION. This is what the
    complexity tax below depends on.

## Changing it

- **Fast feedback.** The common edit gets a trustworthy signal in seconds,
  not minutes.
- **Tests that catch real breakage.** Tests exercise behaviour from the
  outside, and break when the behaviour breaks.
- **Standalone build.** A fresh clone builds and tests with only what it
  declares.
- **Parallel work.** Several people or agents can work at once without
  colliding, for example on shared generated files, global state or a single
  checkout.
- **The next one is cheap.** Adding the fifth command, endpoint, vertical or
  tool does not mean touching twelve places.

## Shipping it

- **One-step release.** From "this is ready" to published artifacts, with no
  hand steps anyone has to remember.
- **Checks where change happens.** Proposed changes are checked before they
  land. Triggers are correct. The cost and latency are worth what the checks
  catch.
- **Narrow release authority.** You know what code and which credentials can
  publish, and fewer is better.
- **Release communication.** Users can find out what changed and why, for
  example from a changelog, release notes or migration notes.

## Running it

- **Observable.** When it misbehaves, someone can see why without adding
  print statements.
- **Fails safely.** Partial runs, retries, a second run and destructive
  operations all leave things in a sane state.
- **Secrets handled.** Secrets are not in the repository, can be rotated,
  and are not needed where they should not be.
- **Access boundaries.** Who can do what is decided on purpose and enforced,
  not simply everyone.

## Keeping it

- **Recoverable state.** Whatever would hurt to lose is backed up, and a
  restore has actually been done. Watch for restore credentials that live on
  the machine being restored.
- **Data can leave.** Users, or you, can get the data out in a usable form.
- **Survives losing a machine or an account.** It can be rebuilt from
  scratch, and no single login can take everything down.
- **Dependencies stay healthy.** Dependencies are known, updated along a
  path that works, and checked for vulnerabilities.
- **Time-bound things are tracked.** Certificates, domains, tokens, pinned
  toolchains and deprecated runtimes are not surprises.
- **Lifecycle is clear.** It is obvious whether this is maintained,
  experimental, frozen or archived, and what that implies.

## Across the ecosystem

- **Shared where sharing helps.** The same problem is solved once when that
  reduces work. It is not coupled when coupling would force lockstep
  releases.
- **Consistent where consistency helps.** The same flags, conventions and
  workflows are used across sibling projects, so that knowing one means
  knowing the rest.

## A different kind: the complexity tax

**Every part present has a reason to be there, and anything unneeded lives
only in git history.** The current state should be minimal.

This one is unlike the others:
- It depends on the design being recorded faithfully against the system's
  goal. That goal sits outside the software, so the record can go stale
  without anything in the code changing.
- A map row can say whether a project's reasons are discoverable at all.
- The substance is judgment: for each part, can its reason still be found,
  and does it still hold?

Treat what you find as a candidate for deletion, not as a defect. The work is
repeated cleanup, so views of this concern are most useful when they name
specific things that could go.
