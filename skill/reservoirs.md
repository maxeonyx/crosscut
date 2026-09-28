# Reservoirs

These examples exist to knock you out of your ordinary distribution, so that
you see concerns that neither [catalogue.md](catalogue.md) nor the obvious
task would suggest. They show the breadth expected, not a list to work
through. Most of them will not apply to the project in front of you. The
concerns that matter most there may look like none of them.

**How to use this file.** Read it, then put it aside and write your own
reservoir for the actual project or ecosystem. Ground every entry in
something you saw, such as a file, a command's output, a commit, or a
dependency. Keep going until new entries are duplicates or clearly out of
scope. That is usually well past the point where you feel you understand the
categories. Only then narrow down.

## People and moments

- **The author, the evening a small tool first works.** Nobody asked what
  happens when it is run twice, run from another directory, or run on the
  laptop they get next year.
- **An analyst whose internal app has quietly become how the team works.**
  Everyone who can open the URL can see and edit everything. Nobody has
  noticed, because everyone who can open it is on the team, for now.
- **Someone opening a repo untouched for fourteen months.** The README's
  setup command fails, and it fails because of a toolchain release, not
  because of anything in the code.
- **An agent sent to fix one flaky test.** The flake is shared host state. The
  same shared state makes three other suites order-dependent, and it is why
  CI needs a retry.
- **A family member who relies on a self-hosted photo library.** They
  cannot operate it. Whether it survives a disk failure depends on a backup
  job nobody has restored from.
- **The person on call at 3am.** They have the runbook, and the runbook
  names a dashboard that was renamed.
- **The maintainer of a patched fork of a large upstream.** Every upstream
  release costs an afternoon, and nobody has written down which patches
  still matter.
- **Five agents working in parallel on five verticals.** They keep colliding
  on one generated file that every vertical regenerates.
- **Someone installing a public package from a search result.** The install
  instructions assume a tool only the author has.
- **The owner of a site built for one event.** The event is over. The site
  still renews a domain and runs a server.
- **A future owner inheriting the project.** Every decision they cannot
  reconstruct from the code is gone.
- **The headless agent in a nightly job.** It cannot ask anyone anything, so
  what it cannot find out has to be reported as unknown.
- **A regulator or auditor.** They ask who changed what and when, and "the
  agent did it" is the answer.

## Shapes of things

Before assuming "the project" means one repository, ask which of these you
are looking at:
- a repository;
- a deployment;
- a machine;
- an account;
- a scheduled job;
- a durable store that matters more than any code;
- a relationship between repositories;
- the ecosystem as a whole.

Some examples of what each shape can turn up:
- a script collection with no entry point;
- a wrapper repo that points at independent repos;
- a monorepo with one deployable and forty libraries;
- an open-source application you deploy but did not write, where what you
  own is its configuration and its data;
- a static site whose only moving part is its domain;
- a skill or prompt file that *is* the product;
- a meta-tool whose whole value is that people trust its answers;
- a dotfile sync tool, where the durable state is everyone's home directory;
- a library consumed by services you do not control;
- a desktop app with an auto-updater, where the update channel is the most
  powerful code path you ship.

## From a moment to a concern

Each of these moments suggests a concern, which is a property that would
have made the moment fine, and a map row for every sibling project:
- The tool that never updated. The concern is staying current. Which of the
  others update themselves?
- "Is the fix live?" The concern is version visibility. Which projects can
  answer that in seconds?
- The restore nobody has tried. The concern is recoverable state. Which
  projects hold state, and which have a restore that has actually been done?
- The fifth vertical that took a week. The concern is that the next one is
  cheap. Where else does adding one mean touching twelve places?

The seed concerns themselves are in [catalogue.md](catalogue.md).

## The mechanism ladder, by example

- **Disappear.** A service kept a local cache only to survive a flaky
  upstream. Once the upstream got a timeout and retry, the cache and its
  backup concern were both deleted.
- **Structural.** "Is the version visible?" stopped needing a check once the
  build embedded the commit and the one deploy command was the only way to
  ship.
- **Existing tool.** Dependency vulnerability questions went to
  `cargo audit` or `npm audit`, and repository settings to `gh api`. A
  hand-written scanner would only rediscover what they already know.
- **Small custom script.** "Does every documented command still run?" was
  answered by a script that extracts code blocks and runs them in a scratch
  directory. It is general, and it still works when files move. A regex over
  today's paths would not.
- **Judgment.** "Would a fresh agent do good work from this guidance?",
  "Does the architecture make the fifth vertical cheap?", "Is this operation
  doc what people actually do?" These are properly answered by an agent with
  a strong prompt. That is a real mechanism, not a stopgap.
- **Moving back down the ladder.** A custom link-checker produced constant
  noise after a repository restructure. The agentic refresh already caught
  every real case, so the checker was deleted.

## Views to offer

The same concern files can answer many questions. Answer them by reading the
files and the projects, not by building analytics.

- What have none of these projects considered?
- Which concerns matter nearly everywhere, and which only where there is
  durable data?
- What depends on something that cannot be recovered?
- Where have we solved the same problem five ways?
- What breaks first if nobody touches this for a year?
- Which unknowns matter more than the known problems?
- Which concerns are obsolete now?
- Which judgments have stabilized enough to hand to a deterministic tool?
  Which deterministic tools have become noise?
- If I have one hour, where is the most leverage?
- What will an agent handed this tomorrow fail to recover?

## Combinations

Some of the best discoveries sit where two dimensions cross. Try pairing a
shape with a moment and a concern:
- scheduled job × machine replacement × silent failure;
- public package × upstream license change × consumers pinned to old
  versions;
- home service × certificate expiry × a family member who cannot fix it;
- five similar APIs × one auth bug × the fix landing in only one;
- CI change × cost × a path that is never actually exercised;
- agent guidance × a new model version × instructions that worked for the
  old one.
