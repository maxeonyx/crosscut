# Why CrossCut has this shape

This is the design record: the search that produced the current shape, kept so
that a future agent reconsidering CrossCut can see what was weighed. It is
evidence, not authority. If the world it describes has changed, change the
design and update this file.

The doctrine itself lives in [`skill/SKILL.md`](../skill/SKILL.md). Anything
here that restates it must carry it forward whole, including the requirement
to carry it forward again.

## The problem in one paragraph

A capable 2026 coding agent understands the request, reads the nearby code,
makes a locally sensible change, validates it, and stops. The concerns an
experienced engineer keeps in peripheral vision (recovery, deployability,
version visibility, stale guidance, the same bug in five sibling repos) stay
outside that loop unless someone lists them. CrossCut's job is to make the
agent do that listing itself, concretely and specific to the project, and to
keep the useful results visible over time, without turning any of it into
obligation.

## Evidence the design started from

- **The predecessor.** `agent-tools` had `crates/standards`: 35 concerns as
  Rust ratchet tests (about 7k lines), `NOT_APPLICABLE` lists, commit-keyed
  review attestations in `state.json`, and ledger CI. It showed both halves of
  the problem:
  - Good: concerns as independent aspects, "red is information", evidence
    shared between checks.
  - Bad: every concern had to become a boolean test before it "counted"
    ("the concern is not real until enforcement exists"). Applicability was
    precomputed rather than reasoned about. Judgment-shaped concerns became
    attestations keyed to a commit, which go stale silently. Much of the
    recent work was machinery about the machinery (ledger holes, ratchet side
    effects, bot commits). The suite could only see Rust CLI tools with a
    Pages site.
- **The harnesses.** Claude Code, Codex and OpenCode are all installed and
  authenticated. All three load Agent Skills (`SKILL.md` directories):
  Claude Code from `.claude/skills`, Codex from `.agents/skills`, and OpenCode
  from both. All three have a one-shot non-interactive mode (`claude -p`,
  `codex exec`, `opencode run`).
- **Invocation libraries.** Every vendor SDK is a subprocess wrapper around
  the same CLI. ACP is the only live cross-harness standard, and it is built
  for editors (sessions, permission prompts, diffs), which is too much for a
  one-shot assessment. coder/agentapi is deprecated. Decision: shell out and
  keep the command table small and overridable.

## Reservoir expansion done before choosing

The brief's reservoirs were the starting point, not the list. What follows is
the extension, drawn from the real ecosystem on this machine and from first
principles. It stopped when new entries were mostly restating earlier ones.

**Actors not in the brief:**
- the scheduled headless agent with no human to ask;
- a cheap model doing routine refreshes;
- a delegated subagent that sees one slice;
- a family member who depends on a home photo service but cannot operate it;
- the person on call at 3am reading operational notes;
- a future owner with no memory of any decision;
- a contributor arriving from a GitHub search;
- a package consumer pinned to an old version;
- a regulator or auditor (work context, intuition pump only);
- the CI system as an actor with its own failure modes;
- the model provider, whose model deprecations change agent behavior;
- the domain registrar and certificate authority, whose expiry dates are
  silent deadlines.

**Lifecycle moments not in the brief:**
- first durable data;
- first other user;
- first incident;
- machine replacement or OS reinstall;
- a year without touching the project;
- an upstream fork falling behind;
- an upstream license change;
- a CI platform deprecating a runtime;
- a language edition change;
- moving repos between personal and organization accounts;
- a harness or model upgrade that changes how the same prompt behaves;
- the event a time-bounded project existed for passing (a wedding site after
  the wedding: the concern becomes archiving, not uptime);
- a tool being archived;
- deletion.

**Project shapes seen here that the brief lists only generically:**
- a dotfile/config sync tool, where the durable state is the user's machine;
- a tmux bridge whose tests share a live tmux server with the host, which is
  the source of a real CI-only flake;
- a TDD ratchet: a meta-tool whose whole value is trust, so "does it actually
  detect?" is the concern;
- an umbrella control plane with submodules;
- per-tool static sites on custom domains;
- a skill, a prompt artifact that is itself the product;
- a personally patched fork of a large upstream (OpenCode);
- devenv/Nix environments;
- a personal OS configuration;
- small time and date utilities, where timezones and daylight saving are the
  failure surface.

**Concern types the brief does not name:**
- trust in meta-tools;
- test isolation from host state;
- the cost of maintaining a fork;
- DNS, domain and certificate expiry;
- harness or model drift under stable prompts;
- the cost of agent runs;
- prompt injection from repository content into agents that can run commands
  (this applies to CrossCut itself);
- secrets leaking into agent transcripts;
- skill and binary version skew;
- Windows path and shell assumptions in a suite that ships Windows builds;
- exit-code conventions across sibling tools;
- supply-chain integrity of auto-update;
- privacy of update checks;
- time-boundedness;
- **single-account blast radius.** Losing one GitHub account takes the
  source, the releases, the sites and the auto-update channel of every tool
  at once. This is an ecosystem-level concern that no per-repo checklist
  would ever produce, and it is the kind of discovery CrossCut exists for.

**Views not in the brief:**
- "What breaks first if nobody touches this for a year?" (the decay view:
  expiries, deprecations, tokens);
- "What does a fresh machine need before any of this works?";
- "What would a hostile repository make an agent running CrossCut do?";
- "What is load-bearing that nobody would think to back up?"

**What the expansion changed.** Concerns attach to at least these units: a
repo, a deployment, a machine, an account, a relationship between repos, and
the ecosystem as a whole. That argues against any schema keyed by repo, and
for concern text that names its own scope in prose.

## Candidate systems

These are materially different control structures, not variants of one CLI:

1. **Notebook.** Each concern is a plain Markdown file holding the question,
   why it matters here, how to look, and the latest dated *Current view*. A
   skill carries the doctrine and the modes of work. A small CLI delivers
   prompts, lists concerns, and runs headless refreshes through an installed
   harness.
2. **Executable protocol.** Each concern is an executable that emits
   structured observations. Agentic concerns are executables that call a
   harness, and a runner aggregates the results. This is the standards crate,
   generalized.
3. **Generator that dissolves.** CrossCut is a one-shot consultant. It turns
   findings into repo-native machinery (CI jobs, lint config, `AGENTS.md`
   text) and keeps no state of its own.
4. **Standing questions.** A section of `AGENTS.md` lists questions that
   every agent session revisits opportunistically. There is no refresh and no
   stored view.
5. **Issue-tracker native.** Each concern is a labelled GitHub issue, and a
   refresh is a comment containing the current view.
6. **Observatory.** A central service with a database and dashboards.

## Replaying episodes against the candidates

| Episode | Notebook | Executable | Generator | Standing Qs | Issues |
|---|---|---|---|---|---|
| "Use CrossCut here" on a new service | skill drives discovery; writes 2–4 concern files | must write executables before anything is visible | good first pass, nothing persists | cheap, easily ignored | concerns read as a work queue |
| Mature repo, no history | discovery plus a small chosen set | same, heavier | one-shot | no | issue noise |
| Analyst's app, engineer arrives later | nothing required of the analyst; the concern dir can live outside their repo | needs a runner in place | ok once | needs repo edits | needs repo access and labels |
| Wrapper over many home projects | inventory is prose plus paths; concerns name their scope | runner needs a repo model | no ecosystem view | per repo only | cross-repo is awkward |
| Narrow bug reveals an invariant | generalize mode proposes a concern file | proposes an executable | proposes machinery | adds a line | opens an issue (framing risk) |
| Concern answered by an existing tool | "How to look" names the command | wrapper around the tool | best: integrates the tool directly | n/a | n/a |
| Judgment concern | the file *is* the persistent prompt | executable wraps a prompt | lost after the run | re-judged ad hoc | ok |
| Concern becomes obsolete | `git rm` one file | delete code and registry entry | n/a | delete a line | close issue |
| Agentic concern becomes deterministic | "How to look" changes, or the concern graduates into CI | natural | natural | n/a | n/a |
| CI or scheduled run | `crosscut refresh` headless; commit the diff | natural | n/a | no | bot comments |
| Offline or no harness | files are still readable; views keep their date | runner fails | n/a | fine | fails |
| CrossCut vanishes | a folder of readable questions with dated answers | dead runner | fine | fine | fine, but locked to GitHub |
| Pass/fail pressure | lowest: output is prose | highest: exit codes invite gating | medium | low | high: open/closed |

The notebook wins or ties on nearly every row. It takes the best traits of
the others:

- from Standing Questions: an optional one-line pointer in a project's agent
  guidance, so ordinary sessions notice the concerns;
- from Generator: "graduated into repo-native tooling" is a first-class
  mechanism outcome, not a failure;
- from Executable: headless refresh for CI and scheduled use.

The issue tracker was rejected on framing: issues are work items, and work
items are obligations. The observatory was rejected on cost.

## The integrating idea

**A concern file is at once the definition, the refresh prompt, and the
latest result.** Several requirements fall out of that one choice:

- Persistence and history come from git: `git log -p` over one file shows
  how the understanding evolved.
- "What changed since last time" is `git diff`.
- Deleting a concern is deleting a file.
- A human, an interactive agent, and a headless agent all read the same
  thing.
- Nothing about it depends on CrossCut existing.
- Adding a concern touches one file.
- The mechanism can evolve (command, script, judgment, gone) without
  changing the representation.

Headless refresh never edits the question or context. The agent returns a
new *Current view* and the CLI replaces only that section. Refreshing and
reconsidering therefore stay separate acts, and a cheap model cannot corrupt
the intent. The headless agent never *needs* to write. Whether it *can*
depends on the harness: Codex's read-only sandbox enforces it. Claude runs
with Bash, the edit tools denied, and the target's project settings not
loaded. Concern files are therefore trusted like scripts.

## Constraints extracted

- **Known goals:** horizon expansion; visibility over time; freedom to
  ignore; works for one project or many; the agent is the primary user;
  small.
- **Strong hypotheses:** most of the value is in prompts; git plus Markdown
  is enough persistence; a harness is always available interactively and
  often headless.
- **Unknowns:**
  - how well cheap models do headless refreshes;
  - whether a synthesized cross-concern overview earns its cost;
  - how concern material should be contributed back.
- **Non-goals:** gating, scoring, compliance state, an agent runtime, a
  plugin system, a project graph, a database.
- **Deliberately open:** when `agent-tools` migrates from `crates/standards`
  to concern files (Max: develop on a separate `crosscut` branch until
  CrossCut 1.0).

## Why the CLI exists at all

The skill alone works interactively. The binary earns its place by:

- **Delivery.** One `cargo install` or release download carries the skill,
  and `crosscut install-skill` places it where all three harnesses look, so
  the skill and the binary cannot drift apart.
- **Orientation.** An agent that only has the binary on `PATH` gets the
  doctrine and the next step from `crosscut` alone.
- **Headless refresh.** Running each concern through a harness, isolating
  failures, and writing back only the view section is fiddly enough to be
  worth doing once, correctly.
- **Listing.** `crosscut list` answers "what are we watching and how stale
  is it?" without spending a model call.

Anything beyond this needs its own justification.
