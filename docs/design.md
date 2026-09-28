# CrossCut design

This file is the design record: what CrossCut is for, the situations it has
to handle well, the candidate designs, and why the current one was chosen.
Earlier rounds are in git history.

It is evidence, not authority. When the world it describes changes, reopen
the design and rewrite this file.

## 1. What we are trying to make true

**Why:** every project has more worth caring about than the task at hand.
CrossCut gets your agent to advocate for it.

This is the canonical why. The README, the site and the doctrine say it in
their own form, but derive from it.

> An engineer and their agents can take control of quality across many
> projects and tools. For every aspect of quality that matters, they can see
> where each project stands, track it over time, and improve it on purpose.
> They can also decide, visibly, which aspects they will not manage.

Separating what we are sure of from what we are not:

- **Terminal outcomes:**
  - Every aspect that matters is accounted for, across a varied ecosystem
    (the diversity of aspects).
  - For each aspect, where each project stands is visible, current, and
    cheap to refresh.
  - Improving one aspect across many projects is a tight loop, and the
    result can be confirmed.
  - Deciding not to manage something is recorded and respected. Leaving
    essential complexity unmanaged is a legitimate choice, but it should be a
    visible one.
  - Agents are the main operators, so all of this has to work for them.
- **Hard constraints:**
  - It never gates. A red cell is information. It is not a failure of the
    run, or of the person.
  - Target projects do not have to know CrossCut exists.
  - It has to work across different kinds of project: CLIs, sites,
    services, libraries, deployments, scheduled jobs.
- **Strong hypotheses:**
  - A concern is a generally useful capability or property. The map of it
    across projects is the product.
  - Most cells can be answered by machines, and should be.
- **Unknowns:**
  - How many cells can really be deterministic in a typical ecosystem.
  - What agentic checks cost at ecosystem scale.
  - How one ecosystem's concern material transfers to another.
- **Historical accidents to avoid repeating:**
  - Ratcheting results. That was agent-tools' predecessor.
  - Making every mechanism agentic, and letting prompts drift into bug
    hunting. Both happened in CrossCut round 1.

## 2. Evidence from two failed shapes

### agent-tools `crates/standards` (the predecessor)

- **Kept the right thing:** a concern × project map, and deterministic,
  fixture-tested checkers.
- **What went wrong:**
  - Results were ratcheted (pending → passing, never back), so a change
    outside a project turned its map red.
  - A concern was one test and one bit. The per-project detail lived only
    in panic text, and nothing kept it.
  - Judgment concerns became commit-keyed attestations, which went stale
    with every commit.
  - Applicability lived in code (`NOT_APPLICABLE` lists), so it was never
    reasoned about.
  - Adding a concern meant touching the module, the registry and the specs.
  - The machinery about the machinery (ledger, attestations, gatekeepers)
    became a quarter of all commits.

### CrossCut round 1 (this repository's earlier commits)

- **Kept the right thing:**
  - A concern is one plain file that survives without the tool.
  - The skill carries the doctrine.
  - Headless refresh works through existing harnesses: a pool, timeouts,
    containment.
  - No gating anywhere.
- **What went wrong:**
  - **Every mechanism became agentic.** Even `gh release view` ran only as
    prose an agent was told to follow, so nothing could run without a model,
    cheaply, in CI.
  - **Discovery drifted into bug hunting.** The prompts rewarded
    "surprising findings", and agents answered with defects.
  - **A refresh regenerated the whole view.** A human decision, such as
    "Windows support for oc: deliberately not", survived only if the next
    agent chose to copy it forward.
  - **Views were prose containing a table**, so machines had to parse
    Markdown written by a model.
  - **There was no answer to "how is this mechanism tested?"**, for custom
    checks or for prompts.
  - **The tool was built and tested against real repositories.** Applying
    CrossCut got mixed up with building it, and the real repositories'
    peculiarities leaked into the design. From now on, tests, evaluations and
    demos use invented ecosystems (section 7).

## 3. An invented ecosystem to think with

This is **Juniper**, a small team's projects. Nothing in the design is
allowed to depend on details of Max's real repositories.

| project | kind | notes |
|---|---|---|
| `pantry` | Rust CLI | Released to GitHub. Self-updates. |
| `larder` | Rust CLI | Sibling of pantry, written later. No self-update. |
| `ledger-api` | Python service with Postgres | Deployed by hand. Holds customer data. |
| `ledger-web` | Static site | Deployed by CI to Pages. |
| `nightly-sync` | Scheduled job | Cron on one VM. |
| `common-auth` | Library | Consumed by ledger-api, and by the CLIs, by git tag. |
| `photo-vault` | Upstream open-source deployment | Only config and data are owned. |

## 4. Episodes the design has to survive

Each episode is written from the operator's side, which usually means an
agent's.

1. **First map.** "Use CrossCut on Juniper." The agent should end up with a
   concern set drawn from the catalogue and from asymmetries such as
   "pantry self-updates, larder doesn't". It should have a mechanism per
   concern, cheapest first, and a filled grid. It should not produce a list
   of bugs.
2. **New project.** `larder` is added. Its column should fill in without
   anyone editing each concern. The interesting view is "where larder lags
   its sibling".
3. **New concern.** "I want every tool to stay current." A deterministic
   check for the CLIs, deciding n/a for the site, and judgment for
   photo-vault's upstream images.
4. **Raise the suite.** An agent is asked to give every CLI self-update. It
   reads the concern, fixes larder, and reruns the check in seconds. The
   cell turns yes. No model is needed to confirm it.
5. **A deliberate "no".** Max decides that nightly-sync will never have
   version visibility, because it is being retired. That decision has to
   survive every future refresh. It should show as `deferred` with its
   reason, and should re-surface if the observation changes, for example if
   nightly-sync gains a version file anyway.
6. **Drift.** ledger-web's deploy stops publishing its version file. The
   next time anyone runs `crosscut check`, the cell changes, and the diff of
   the observations shows it. Nothing fails. Scheduled runs are out of scope
   (Max, 2026-09-28): checks run when a person or an agent runs them.
7. **The judgment tier has to be tested too.** An agentic check for "agent
   guidance is true" gets rewritten. Did the rewrite make it better or
   worse? It should be run against invented projects with known answers.
8. **A custom check becomes noise.** A layout change makes a script report
   `missing` everywhere. Its fixtures should catch this. The concern stays;
   the check is fixed or replaced, or the concern moves up to judgment.
9. **Offline, or no credentials.** `gh` is logged out. The cells that need
   it become `unknown: gh not authenticated`, and everything else still
   runs.
10. **Cost.** 20 concerns × 7 projects is 140 cells. If deterministic cells
    take milliseconds, and only the judgment cells cost model calls, a
    refresh is cheap enough to run daily.
11. **An agent working on one project** asks: which concerns apply to
    `ledger-api`, and where does it stand? That is one column of the map.
    In the wrapper pattern, work happens from the wrapper checkout (as with
    `tools/<name>` in agent-tools), so searching upwards finds `crosscut/`.
    A project with no wrapper keeps its own `crosscut/`. Only in the wrapper
    pattern do projects not need to know about CrossCut.
12. **The complexity tax.** A concern of a different kind: it asks whether
    each part has a discoverable reason to exist. Its cells are judgment, and
    its evidence is a list of candidates for deletion.
13. **Relational concerns.** "Consistent conventions across the CLIs"
    compares projects with each other. The check needs to see its siblings,
    not just its own project.
14. **CrossCut disappears.** What is left should still make sense: a folder
    of concern definitions, decisions, check scripts and dated observations.

## 5. Candidate models

- **A. Round 1.** One Markdown file per concern. A refresh has an agent
  rewrite the file's "Current view" section, which is prose containing a
  table.
- **B. Observations, decisions and mechanisms, with a computed map.**
  - Each concern is a directory: a definition, which includes the human's
    decisions, plus a mechanism, test fixtures, and machine-written
    observations.
  - The map is observations joined with decisions.
- **C. A code-first suite.** Checkers in one compiled crate, as in the
  predecessor, but reporting rows instead of pass or fail.
- **D. One record per cell.** A file, or a database row, per (concern,
  project), each with its own status and history.

| Episode | A | B | C | D |
|---|---|---|---|---|
| 1 first map | ok | ok | heavy: code per concern | ok |
| 2 new project | an agent rewrites every view | a new column, cells computed | ok | 20 new files |
| 4 raise and confirm | a model call to confirm | the check reruns in ms | ok | ok |
| 5 deliberate "no" survives | fragile: the agent must copy it forward | structural: the decision lives apart from observations | needs a decisions file anyway | ok |
| 6 drift, cheap rerun | model per run | cheap | cheap | cheap |
| 7, 8 mechanisms tested | nothing | fixtures, for every tier | fixtures, for code only | nothing |
| 9 partial failure | per concern | per cell | per test | per cell |
| 13 relational | prose | the check sees its siblings | ok | awkward |
| 14 without CrossCut | readable | readable | a dead crate | readable, but a sprawl |
| Adding a concern costs | 1 file | 1 directory: definition plus check | module plus registry | many files |
| Agents parse model prose | yes | no: observations are written by the tool | no | no |

B wins or ties almost everywhere. It keeps A's best trait, plain files that
survive without the tool, and C's best trait, deterministic and tested
checks. It avoids D's sprawl, because observations for a concern live in one
file.

## 6. The chosen model

### The integrating idea

**Separate what is observed from what is decided, and compute the map from
both.**

- **Observations** are regenerable at any time, by any mechanism. They are
  owned by the tool, and never edited by hand.
- **Decisions** are the human's, such as "n/a: archived" or "deferred:
  being retired". They live with the concern's definition, and no refresh
  touches them.
- **The map** is the observation, annotated by the decision if there is one.
  When the two disagree, for example a decision says deferred but the
  project now observes yes, the map says so.

Several requirements fall out of this one split:
- Deliberate non-management is durable (episode 5).
- A refresh is idempotent and safe to run headless.
- Checks only have to report facts.
- History is just `git log` over the observations file.
- "What changed?" is `git diff`.

### Layout

```
crosscut/
  projects                 which projects the map covers: paths or globs, one per line
  concerns/
    <slug>/
      concern.md           name, user stories, what it looks like per kind
                           of project, and ## Decisions. Human- and agent-owned.
      check                the mechanism: an executable (tiers 1-2)
        or check.md        a prompt (tier 3)
      fixtures/<case>/     invented projects, each with an `expect` file
      observed.tsv         written by `crosscut check`, and committed (section 9). Never edit by hand.
```

`crosscut/projects` is a plain list with one path or glob per line (for
example `tools/*`), relative to the directory that holds `crosscut/`. Each
project's name is its directory name. There is nothing else to keep in sync,
because checks discover facts from the projects themselves: a site URL from
`docs/CNAME`, a version from the manifest, and so on. A richer format waits
until a flow needs it.

### The mechanism interface: one cell at a time

Every mechanism answers one question: **for this project, what is its status
on this concern, and what is the evidence?**

```
check <project-dir>          environment: CROSSCUT_PROJECT, CROSSCUT_PROJECTS (siblings)
→ stdout: <status>: <evidence>     status ∈ yes, partly, missing, n/a, unknown
```

The interface is the same for every tier:

1. **Tier 1, an off-the-shelf check.** The `check` script is glue around a
   mature tool (`gh`, `curl`, `cargo audit`, a linter), a few lines long.
   The tool does the judging.
2. **Tier 2, a custom check.** Real logic of our own. The bar is high:
   - It must come with fixtures covering true positives and true negatives.
   - It is itself on the map for concerns such as tests that catch
     breakage, and the complexity tax.
   - When it turns noisy, it gets replaced.
3. **Tier 3, an agentic check.** `check.md` is a prompt. CrossCut runs it
   through the installed harness for one project and parses one status
   line. It is easy to write, and just as much in need of fixtures. Running
   those fixtures costs model calls, so they run on demand.

Before tier 1 there is always a tier 0, which is a question rather than a
mechanism: could the concern disappear, or could the good property become
structural?

`deferred` is never observed. It exists only as a decision, which is what
keeps "not managed" a human act.

### Commands

| Command | What it does | Model? |
|---|---|---|
| `crosscut check [slug...]` | Runs mechanisms per cell, in a pool, and writes `observed.tsv`. | Only for tier 3 |
| `crosscut map [--project p]` | The grid of observations and decisions. One project gives one column (episode 11). | no |
| `crosscut test [slug...] [--agentic]` | Runs each mechanism against its fixtures, and compares with `expect`. | only with `--agentic` |
| `crosscut prompt <mode>` | The skill's modes, for any harness. | – |
| `crosscut install-skill` | Installs the skill. | – |

`check` exits 0 when every cell ran, whatever the statuses. A cell that could
not run becomes `unknown: <why>`. A failed mechanism does not fail the
command.

### What the skill is for, in this model

The skill is where agents learn to do what the tool cannot:
- choose concerns: from the catalogue, from asymmetries between siblings,
  and from the reservoirs;
- write definitions and user stories;
- pick the lowest tier that works, and write that check;
- write fixtures;
- propose decisions to the human;
- read the map to decide where the leverage is.

The doctrine, including its requirement to carry itself forward, stays at
the top of SKILL.md.

## 7. How CrossCut itself is tested

- **The tool:** black-box tests against invented fixture ecosystems in
  `tests/fixtures/`. There is no real repository anywhere in the suite.
- **Mechanisms:** `crosscut test` runs fixtures. That is the same machinery
  users rely on for their own concerns, so CrossCut's tests exercise it.
- **The prompts:**
  - An evaluation ecosystem, Juniper from section 3, is built as fixture
    directories with known answers.
  - Running `setup` over it headless should produce capability concerns such
    as staying-current, with larder missing where pantry has it. It should
    not produce a bug list.
  - This is scored by hand, or by a second agent, and run on demand, because
    it costs model calls.
- **The site demo** is Juniper's real `crosscut map` output, so the demo is
  true.

## 8. Harness facts still in force

These were verified in round 1:
- `claude -p`, `codex exec` and `opencode run` all work headless.
- All three load Agent Skills.
- Only Codex's read-only sandbox actually prevents writes. A Claude refresh
  once left an untracked file in a repository despite being told not to.
- A misconfigured `opencode run` exits 0 with no output.

The pool, timeouts, containment flags and stderr reporting carry over as
they are.

## 9. Committing observations, worked through as flows

Max's hypothesis is that results should be committed. Here it is tested
against concrete flows in small steps. `obs` means the concern's
observations file.

**Flow A: raise one project, then confirm.**
1. Max, in the Juniper wrapper: "give larder self-update, like pantry".
2. The agent runs `crosscut map --project larder`. The map shows
   staying-current `missing`.
3. It reads `concerns/staying-current/concern.md` and `check`.
4. It implements self-update in `larder`, and commits there.
5. It runs `crosscut check staying-current --project larder`, which takes
   milliseconds, because the check is tier 1. The larder row becomes `yes`.
6. It commits in the wrapper: the larder pointer bump, and the `obs` change,
   together.

→ If `obs` is committed, the wrapper commit records *what the change did to
the map*, next to the code that did it. A reviewer sees
`larder missing → yes` in the diff. If it is not committed, that record
exists nowhere.

**Flow B: the next day, a fresh agent.**
1. "Where are we on staying-current?"
2. `crosscut map` reads the committed `obs` immediately, and shows each
   row's `since` date.
3. For a stale-looking tier 1 or 2 row, rerunning costs milliseconds. For
   tier 3 it costs model calls, so the committed row saves them.

→ Committed observations are a cache of expensive judgment, and a record of
cheap facts.

**Flow C: two agents in parallel clones.**
1. Agent 1 checks staying-current and agent 2 checks version-visibility.
   They write different files, so there is no conflict.
2. Agent 1 checks staying-current for larder, and agent 2 for pantry. That
   is the same file but different lines, so git merges them, provided the
   rows are in a stable sorted order.

→ One `obs` file per concern, one row per project, sorted by project. A
single global results file would conflict constantly.

**Flow D: rerun with nothing changed.**
1. Someone runs `crosscut check` twice in a row.
2. The first run changes whatever changed. The second should produce **no
   diff**, or history fills with noise.
3. Deterministic evidence is stable, so this works for tiers 1 and 2.
   Tier-3 evidence is reworded on every run.

→ Rows carry a `since` date, which moves only when the status changes, and
no "last checked" timestamp, because that would change on every run. For
tier 3, if the status is unchanged, the old row is kept whole. Git history
then becomes the history of *changes to the map*, which is what "tracking
over time" means.

The cost: the file does not say when a row was last *confirmed*. For tiers 1
and 2, rerunning is the answer. For tier 3, `since` is a lower bound. This
compromise is accepted, and the thing that would reopen it is someone needing
"last verified" for judgment cells.

**Flow E: "what changed this month?"**
1. `git log -p --since=1.month -- crosscut/concerns/*/observed.tsv`.
2. Every hunk is a status change, or a factual evidence change, dated by its
   commit.

→ This only works if the file is committed, and only reads well because of
flow D.

**Flow F: a project is removed from the inventory.**
1. `nightly-sync` is retired, and removed from `crosscut/projects`.
2. The next check drops its rows. Git keeps them.

**Conclusion.** Commit `obs`: one file per concern, sorted rows of
`project, status, since, evidence`, rewritten only when something actually
changed. `crosscut check` never commits. The agent commits the observations
with the work that caused them.

## 10. Resolved

- Tier-3 granularity is **per cell** (Max, 2026-09-28).
- There are no scheduled runs. They are out of scope (Max, 2026-09-28).
- The wrapper-discovery question dissolves; see episode 11.

## 11. Evaluation 1 of the round 2 prompts (2026-09-28)

A fresh agent was given only `crosscut prompt setup`, the binary, and an
expanded invented Juniper with no `crosscut/`.

It produced three capability concerns, one at each tier, each with a reason
for its tier and a tier-0 "design it away" idea:
- version-visibility, at tier 2;
- staying-current, at tier 1, over `cargo metadata`;
- recoverable-state, at tier 3.

It also produced:
- 18 fixture cells, all passing, including the tricky cases;
- sibling gaps (larder lacks what pantry has; auth is solved twice);
- decisions proposed but not made;
- defects mentioned in one line each.

The prompt-check loop worked too. Haiku answered `n/a` where the agent
judged `unknown`, so the agent tightened `check.md` and added a fixture.

Fixed after the eval: setup now includes `establish.md`, and an empty
`map` says how to start.

Still open, and worth reopening when they bite:
- **Enforcing read-only prompt checks.** Only Codex's sandbox enforces it.
  One option is preferring Codex when both are installed; another is
  running the harness in a disposable copy of the project.
- **Showing prompt checks whose fixtures were never run.** The map cannot
  show this without recording test runs, which is new state. Wait until
  someone is misled by an untested judgment.
- **Tension between testing a prompt check and observing with it,** when
  the model budget is tight. Establish says to test first. The eval's
  budget was artificial, so that stands.
