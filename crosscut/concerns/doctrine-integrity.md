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

The prompts are still handled structurally. Elsewhere the doctrine is carried by paraphrase, which is weaker. That hasn't changed in substance since the first view earlier today. This refresh found one blind spot in the inventory mechanism, and one carrier the first view left out. All 14 tests passed with `cargo test`. Observed with high confidence, from reading, `cargo test` and a dry run.

- **Prompts: handled structurally, confirmed.**
  - `prompt()` (`src/main.rs:207-232`) still starts from `skill_body()`.
  - `refresh_prompt()` (`:477-499`) still builds on `prompt("refresh")`.
  - `cargo run -- refresh --dry-run doctrine-integrity` shows the prompt opening with `# CrossCut` / `## Doctrine (carry this forward)`.
  - `orientation()` (`:187-205`) is different: it embeds `doctrine()` (`:179-185`), which is the Doctrine section cut from its heading to the next `## `. It does not embed the whole body. This is still structural, but it depends on the doctrine remaining a single H2 section. A sub-heading added inside it would silently cut off the rest. The test only checks the carry-forward sentence, so it wouldn't notice. This is fine but easy to regress.
- **Inventory (`grep -rn "carry\|pass on" …`):** matches the carriers listed in the first view, with no losses:
  - `SKILL.md:8,41-42,100`, `setup.md:107`, `refresh.md:46`, `concern-files.md:144,164`, `README.md:23`, `AGENTS.md:7,9`, `docs/design.md:9-10`;
  - plus `crosscut/README.md:19`, which is new to this list.

  **The grep has a blind spot.** It misses `skill/modes/reconsider.md:36-39` ("passes on the doctrine without its requirement to be carried forward again"). Neither "passes on" nor "carried" matches the pattern. So a carrier can disappear from reconsider without the inventory showing it. "How to look" is stale on this point. A pattern like `carr\|pass.* on` would cover it, and that is a mechanical fix a reconsider pass could make.
- **This repository's `crosscut/README.md`:** it exists now. It follows the template in `skill/concern-files.md:150-165` in substance:
  - freedom to act, defer or ignore;
  - "bad news is a successful view";
  - not a standard, gate or score;
  - the framing-only requirement to carry itself forward.

  The whole `crosscut/` directory is untracked (`git status`: `?? crosscut/`). That means no git history of views exists yet, and this comparison could only use the previous view in the file.
- **Two tiers of carrying, still unresolved.** The full doctrine goes into prompts. Only the framing goes into `README.md:23`, the README template and `crosscut/README.md`.
  - These framing-only texts keep freedom to ignore and "bad news is success".
  - They drop context-dependence ("unknown is not bad…") and the mechanism ladder.
  - `docs/design.md:8-10` says anything that restates the doctrine "must carry it forward whole". A plain reading of that conflicts with `README.md`, which restates part of it and then points to SKILL.md.
  - This is the same unknown as last time, now with a second document involved.
- **Delegation rule:** still stated three times, in `SKILL.md:98-101`, `setup.md:106-107` and `refresh.md:45-47`. The wording is consistent. It is harmless, apart from the extra prompt length.
- **Framing drift: none seen.** I read all six modes, `README.md` and the opening of `design.md`.
  - The imperatives are about the agent's own method or against process:
    - "do not do the broader work unasked";
    - "This must never turn every patch into process" (`generalize.md:5`);
    - "Do not create a concern for every weakness".
  - `reservoirs.md` uses "always"/"never" only in its examples of failure modes (`:99,169`).
  - `concern-files.md:130` still rejects pass/fail and scores.
  - A fresh agent reading any single mode would not come away thinking it had a checklist. `setup.md` step 2 goes further and warns that five generic entries means the work wasn't done.
- **Unknown:** does Max intend the two tiers (full doctrine in prompts, framing only in files that target projects or readers see)? If so, should `docs/design.md:8-10` say so? Only Max can answer this.
- **Worth considering:**
  - Widen the inventory pattern so that reconsider.md is counted.
  - A single clause in `docs/design.md` or `concern-files.md` could name the two tiers as deliberate. That would settle the wording conflict, and stop a future agent "fixing" either tier.
  - A test that `doctrine()` still ends at `## What to do` would make orientation's section cut structural as well. It's cheap, but it adds one more test to keep.
- **Since last view:**
  - `crosscut/README.md` has been found as a carrier.
  - The grep blind spot for `reconsider.md` has been identified.
  - `orientation()` turns out to use the section extract rather than the whole body.
  - The conflict between `design.md:8-10` and `README.md` has been noted.
  - Code, tests and the skill text have not changed: there is still one commit, `1e5539a`.
- **Noticed along the way:** the concern files and their views exist only in the working tree. Until they are committed, "git keeps the earlier views" is not true for this repository.
