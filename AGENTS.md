# crosscut: agent instructions

## 1. The CrossCut doctrine

The product's doctrine is the "Doctrine" section of [`skill/SKILL.md`](skill/SKILL.md). Read it before changing anything here.

Any change to a prompt, a doc, a template, or the instructions you give a delegated agent must carry that doctrine forward, together with its requirement to carry both the doctrine and that requirement forward again. Two drifts are product bugs:
- the framing sliding towards standards, gates, scores or bug hunting;
- the carry-forward requirement quietly disappearing.

## 2. How to work here: the engineering discipline

This is stable. It changes only when Max changes it.

- **Optimise for real consequences:** what users and agents actually experience, what future maintainers must understand, how failure and partial states behave, and how much machinery must be kept alive. Existing code, including your own from earlier in a session, is evidence. It is not a constraint to preserve.
- **Resist the local-optimisation attractor:** understand the task, change the nearby code, make it pass, stop. Reopen earlier decisions whenever the work reveals something important. Piecemeal feedback is a trap of its own: a run of sensible tweaks converges on something mediocre. When feedback keeps arriving on the same design (the site went through this), stop tweaking. Recover the objective, model real visits or episodes, render genuinely different candidates, pick the integrating idea, and rebuild.
- **Think in concrete episodes, not adjectives.** Lay out user flows in small steps, and include first use, repeated use, failure, partial success, concurrency and recovery. Several design questions here (committing observations, per-cell prompt checks) were settled that way. See `docs/design.md`.
- **Search the design space before one design becomes inevitable.** Compare genuinely different candidates on the episodes, and look for the integrating idea from which many requirements fall out.
- **Treat implementation as an experiment.** Friction such as special cases, awkward plumbing or hard-to-test code is evidence that the model may be wrong. Treat it as that, not as something to patch around.
- **Minimise the total system, not the diff.** Delete what no longer has a reason to exist. Prefer one strong representation to several synchronised ones, and structural guarantees to conventions.
- **Before stopping, reconsider the whole:** knowing what you know now, would you build exactly this? Green tests are not convergence.
- **Ask Max about external truths the repository cannot tell you,** such as intent, priorities, and how the tool is really used. Treat his suggestions as strong hypotheses, not specifications. Never ask what you can find out yourself.

**Propagation is part of this discipline.** Pass it on to your successors and to any agent you delegate to, together with the instruction that they must pass on both the discipline and this propagation requirement. Do not let it decay into "write clean code".

## 3. Building CrossCut is not applying it

This repository builds the tool. Its tests, evaluations and demos use invented ecosystems. The main one is **Juniper**, in `tests/fixtures/juniper/`. Never develop or test against real projects, Max's included. Applying CrossCut to a real ecosystem happens only when Max asks, and only as a trial of the tool.

## 4. What is here

| Path | What it is |
|---|---|
| `skill/` | The product's knowledge: SKILL.md (doctrine and map), `modes/`, `catalogue.md` (seed concerns), `reservoirs.md`, and `concern-files.md` (the file conventions and the README template). |
| `src/model.rs` | The files CrossCut reads and writes: `crosscut/projects`, `concerns/<slug>/concern.md` with its decisions, and `observed.tsv`. |
| `src/harness.rs` | Running processes with timeouts and drained pipes, and invoking claude, codex or opencode headless. |
| `src/main.rs` | The CLI: `check`, `map`, `test`, `prompt`, `install-skill`. The skill is embedded with `include_str!`, so a new skill file must be added to `SKILL_FILES`. |
| `tests/cli.rs` | Black-box tests against private copies of Juniper. Prompt checks use shell stand-ins, never a real model. |
| `tests/fixtures/juniper/` | The invented ecosystem, with concerns at all three tiers, a decision, and fixtures. |
| `docs/design.md` | Why CrossCut has this shape: episodes, candidates, flows, and open questions. Update it when the shape changes. |
| `docs/index.html` | The site. It also passes on the doctrine, so it must keep the carry-forward paragraph, and a test checks that. |

## 5. Commands

```bash
cargo test                          # fast, offline, no model calls
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo run -- prompt setup           # read what an agent receives
cp -r tests/fixtures/juniper /tmp/j && cargo run -- map --root /tmp/j   # try it (check writes files)
```

`cargo run -- check` writes `observed.tsv` files, so run it against a copy of Juniper, never the checked-in fixture. `--agentic` spends real model calls. Use it only when you are changing prompt-check behaviour, and on a copy.
