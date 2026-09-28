# crosscut: agent instructions

## Read first

CrossCut's doctrine is the "Doctrine" section of [`skill/SKILL.md`](skill/SKILL.md). Read it before changing anything here. Everything in this repository serves it.

Any change to a prompt, a doc, a generated template, or the instructions you give a delegated agent must carry that doctrine forward, together with its requirement to carry both the doctrine and that requirement forward again. The same applies to rewriting this paragraph. There are two failure modes:
- the framing drifting towards standards, gates, scores or mandatory checklists;
- the carry-forward requirement quietly disappearing.

Both are product bugs.

[`docs/design.md`](docs/design.md) records why CrossCut has this shape, including the alternatives that were rejected. Update it when the shape changes.

## What is here

| Path | What it is |
|---|---|
| `skill/` | The product. SKILL.md holds the doctrine and the map; `modes/` holds one file per mode of work; `reservoirs.md` holds the intuition pumps; `concern-files.md` holds the file convention and the README template. |
| `src/main.rs` | The binary. It embeds `skill/` with `include_str!`, so a new skill file must be added to `SKILL_FILES`. |
| `tests/cli.rs` | Black-box tests. Harness runs use small `sh -c` commands, never a real model. |
| `crosscut/` | CrossCut applied to itself: concerns about this repository. |

## Commands

```bash
cargo test                       # fast and offline
cargo clippy --all-targets
cargo fmt --check
cargo run -- prompt setup        # read what an agent will receive
cargo run -- refresh --dry-run   # see a headless refresh prompt without spending anything
```

A real headless refresh costs model calls. Run one only when you are changing the harness integration. Try it on a scratch concern first.

## Working rules

- **Prompts are the product.** A change that makes the prompts shorter and more concrete usually beats one that adds CLI surface. Any new command, flag or file must earn its ongoing cost; say what it buys.
- **Keep the binary thin.** It delivers the skill, lists concerns, and runs headless refreshes. It does not interpret concern results, and its exit status never reflects what a refresh found.
- **Nothing about concern files may require the binary.** If you are tempted to add a schema field or a machine-only section, reread "Why the CLI exists at all" in `docs/design.md`.
- **Before stopping,** run the `reconsider` mode on this repository: `cargo run -- prompt reconsider`. Its own concerns are in `crosscut/concerns/`.
