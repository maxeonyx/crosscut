# Does everything that carries CrossCut's doctrine still carry it whole, together with the requirement to carry it forward, and has the framing drifted towards standards, gates or scores?

## Why this matters here

The doctrine in `skill/SKILL.md` *is* the product. Everything else here
(modes, CLI, templates) serves it. `AGENTS.md` names the two failure modes
as product bugs:
- the framing drifting towards standards, gates, scores or mandatory
  checklists;
- the carry-forward requirement quietly disappearing.

Both happen gradually, through well-meant edits: a shortened README, a
paraphrase in a delegated agent's prompt, a template trimmed for brevity, a
mode that picks up "must" language. The predecessor (`crates/standards` in
`agent-tools`) drifted into exactly the gate-shaped framing CrossCut exists
to avoid. This concern applies strongly for as long as the prompts are
being actively rewritten. Better means that every carrier is either
structurally derived from `SKILL.md`, or deliberately and visibly
paraphrased, and that the prose still reads as visibility rather than
obligation.

## How to look

1. **Structural.** Prompts built by the binary embed `SKILL.md`'s body
   whole (`prompt()` and `refresh_prompt()` in `src/main.rs`), so every mode
   prompt and every headless refresh carries the doctrine by construction.
   Confirm that this still holds:
   - `cargo test`, specifically
     `every_mode_prompt_carries_the_doctrine_and_its_references`,
     `bare_invocation_orients_an_agent_with_the_doctrine` and
     `readme_template_carries_the_framing_and_the_requirement_to_carry_it`;
   - `cargo run -- refresh --dry-run` in a directory with a concern, to see
     that the refresh prompt still starts from the full skill body.
2. **Deterministic inventory.** `grep -rn "carry\|pass on" skill README.md
   AGENTS.md docs src` lists the places that restate or reference the
   requirement. Compare the list with the previous view. A carrier that has
   disappeared, or a new artifact with no carrier, is the signal.
3. **Judgment.** Read `SKILL.md`'s doctrine, then every paraphrase:
   `README.md`, `AGENTS.md`, `docs/design.md`, the `crosscut/README.md`
   template in `skill/concern-files.md`, `orientation()` in `src/main.rs`,
   and any `crosscut/README.md` in this repository. For each one, ask:
   - Does it keep freedom to ignore, context-dependence, "bad news is
     success", and the mechanism ladder? Or has it kept only the slogan?
   - Does it pass on the requirement to pass it on?
   - Across the modes and reservoirs: has imperative or compliance language
     crept in ("must check", "ensure", "required", "all projects should")?
     Would a fresh agent reading only this come away thinking CrossCut is
     a checklist?
   - Tests check one sentence. A doctrine gutted around that sentence would
     still pass, so read the whole thing.
- Not in scope: whether the doctrine itself is right. That is
  `reconsider`, and it is Max's call.

## Current view — 2026-09-28

Nothing has drifted since the last view. The only changes are a `list` headline fix (`3fded27`) and an uncommitted brevity nudge in `refresh_prompt()`, so the open items are the same three: the two tiers of wording, the grep's blind spot, and the unguarded doctrine cut. Observed with high confidence from `cargo test` (20 passed), a dry run, the diff `52869d2..HEAD` plus the working tree, and the inventory grep.

- **Prompts are still handled structurally.**
  - `prompt()` (`src/main.rs:224`) still starts from the skill body.
  - `refresh_prompt()` has moved to `:533`.
  - The dry run still opens with `# CrossCut` / `## Doctrine (carry this forward)`.
  - `orientation()` (`:204`) still embeds only what `doctrine()` (`:196`) cuts out. That is fine but easy to regress, and no test checks where the cut ends.
- **Uncommitted change to `refresh_prompt()`:** it adds "Aim for a view of under 400 words … Start with the one sentence most worth knowing now." It comes after the full doctrine and pushes towards conclusions, not process, so there is no framing drift. It repeats guidance already in `concern-files.md`, which is harmless.
- **Inventory.** No carrier was lost and none was added. The same places match: `SKILL.md:8,41-42,100`, `concern-files.md:155,194`, `README.md:23`, `AGENTS.md:7,9,23`, `docs/index.html:210`, `docs/design.md:9-10` and `crosscut/README.md:38`.
- **The inventory's blind spot is unchanged.** The grep in "How to look" still leaves out `docs/index.html` and `crosscut/README.md`, and still misses `skill/modes/reconsider.md:38-39` ("passes on" / "carried"). A wider pattern, `carr|pass.* on`, does catch it. "How to look" is still stale here.
- **The two tiers of carrying are still unresolved.**
  - `docs/design.md:8-10` still says restatements "must carry it forward whole".
  - `README.md`, the README template, `crosscut/README.md` and the site each carry a paraphrase plus a link.
  - The `324221c` edit to `design.md` records the decision to delete `crates/standards` ("machinery for the machinery"). That reinforces the anti-gate framing, but it didn't touch this clause.
- **No framing drift seen.** A search for `ensure|required|must check|mandatory` over `skill/` found nothing.
- **Unknown:** whether Max counts paraphrase plus a link as carrying the doctrine. Only Max can answer that.
- **Worth considering (unchanged, and cheap):**
  - Add one clause to `docs/design.md` naming the two tiers.
  - Widen the grep pattern and add the two missing paths to it.
  - Add a test that `doctrine()` ends at `## What to do`.
- **Since last view:**
  - The headline fix and its test (19 → 20 tests).
  - The design-notes update.
  - The uncommitted word-limit nudge.
  - None of these changed any carrier.
