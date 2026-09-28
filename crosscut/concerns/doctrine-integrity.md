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

A new carrier, the site (`docs/index.html`), is the most complete paraphrase so far, and a test pins its carry-forward sentence. With the prompts still structural and CI now running the tests, the risk has moved to the two-tier wording question and the inventory's blind spot. Neither has changed. Observed with high confidence from `cargo test` (19 passed), a dry run, the diff `b200de9..HEAD` and reading.

- **Prompts are handled structurally, confirmed.**
  - `prompt()` (`src/main.rs:224`) still starts from `skill_body()`.
  - `refresh_prompt()` (`:527`) still wraps `prompt("refresh")`.
  - The dry run opens with `# CrossCut` / `## Doctrine (carry this forward)`.
  - `orientation()` (`:204`) still embeds only the section cut by `doctrine()` (`:196-202`). That is fine but easy to regress: an `## ` heading added inside the doctrine would silently cut it short.
- **New since last view, and stronger:**
  - `.github/workflows/ci.yml` runs `cargo test` on every push and PR, so the three carrier tests now run on every change, not only when someone thinks to run them.
  - A new test, `readme_and_site_carry_the_requirement_to_carry_the_doctrine`, checks the README and the site. `AGENTS.md:23` lists the site as a carrier.
- **The site (`docs/index.html`).**
  - It keeps freedom to ignore, "Unknown is not bad. Not applicable is not good.", "a refresh that finds bad news has succeeded", and the full five-step ladder.
  - It shows a crossed-out scorecard. That is anti-gate framing, not drift.
  - Its "contagious paragraph" (`:210`) passes on the requirement and links to SKILL.md.
  - It drops "minimise total complexity" and "applies itself to itself", which is reasonable for a landing page.
- **Inventory.** No carrier has been lost. New since last view: `docs/index.html:210` and `AGENTS.md:23`. Line numbers have moved: `concern-files.md:155,194`, `crosscut/README.md:38`.
- **The inventory still has a blind spot.** The grep in "How to look" doesn't cover `docs/index.html` or `crosscut/README.md`. It matches the site only because "pass on" happens to appear. It still misses `skill/modes/reconsider.md:38-39`, which says "passes on" / "carried", and a wider pattern (`carr\|pass.* on`) would catch that. "How to look" is stale here.
- **Two tiers of carrying, still unresolved.**
  - Prompts carry the whole doctrine. `README.md:23`, the README template, `crosscut/README.md` and now the site each carry a paraphrase plus a link.
  - Meanwhile `docs/design.md:8-10` says restatements "must carry it forward whole".
  - The site's own paragraph says "must pass on the doctrine", but it passes on a summary. That makes four documents in tension with `design.md` as written.
- **No framing drift seen.**
  - The new `setup.md` text (side effects of running things, existing concern systems as evidence, "a concern is … not for a single defect") and the new "readable in a minute or two" guidance in `concern-files.md` both push away from process.
  - `grep -iE "ensure|required|must check|mandatory"` over `skill/` found nothing.
- **Unknown:** whether Max intends paraphrase plus a link to count as carrying the doctrine. Only Max can say.
- **Worth considering:**
  - Name the two tiers in `docs/design.md`. One clause would settle the four tensions.
  - Widen the grep and add `docs/index.html` and `crosscut/README.md` to its paths.
  - A test that `doctrine()` ends at `## What to do` would make orientation's section cut structural.
- **Since last view:**
  - `crosscut/` is committed, so views now have git history.
  - The site, the CI workflow and the site test were added.
  - Changes to `setup.md` and `concern-files.md` were neutral to positive.
- **Noticed along the way:** the Pages workflow publishes all of `docs/`, including `design.md`. That is probably harmless, since the repository is public.
