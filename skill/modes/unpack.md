# Unpack: "What are we not thinking about?"

This mode proposes concerns that are not on the map yet. Each one is a card the person can accept or decline. It persists nothing unless the person accepts.

**Unpack from a description of each project, never from its code.** A short description is all you need: "a streaming data API for a stock exchange, Python, built with Make and Nix". Software of that kind meets a known landscape of concerns, and your job is to lay that landscape out. Reading the code pulls you towards the bugs in it, and a card that starts from a bug stays a bug report however you word it. Once the person accepts a concern, a reviewer with the code finds where it cashes out in each project. That review is a separate step, and it is not yours.

If you are working inside a session that can see the code, do not read it for this mode. If you have no description, write one first from the README, the agent guidance and the manifests, in two or three sentences, and stop reading there.

A card is a concern in its finished form, as in [../catalogue.md](../catalogue.md):
- **the rule:** one small sentence, in the rule form ("X should Y"), true as written, hedged only where real exceptions exist;
- **the justification:** the concept behind it, named as a computer scientist would name it (time-of-check to time-of-use, the birthday bound, survivorship bias, identity elements), and why the rule follows;
- **stories:** several, concrete and varied, from different kinds of software, and at least one that says where the rule does not apply;
- **here:** which of the projects it applies to, and how it tends to cash out in software like them;
- **how to review:** how a reviewer with the code would find every instance of it.

## 1. Generate widely

For each project, take its kind, its people and its moments, and ask what that kind of software meets:
- **The kind:** a streaming API, a data warehouse's ETL jobs, a static site generator, a CLI. Each kind has its own landscape: what goes wrong with long-lived connections, with scheduled loads, with derived files, with output read by scripts.
- **The tools:** the language, runtime, build system, platform and libraries the description names. Each carries well-known concerns of its own, and these are often the cheapest wins: a Python module should generally have no side effects when it is imported; a container's entrypoint should pass stop signals on to the program; a Make target should not depend on files it does not declare. Include the ones that fit. Rote is fine when it pays.
- **The schlep:** shared upkeep that every repository needs and nobody enjoys, such as agent guidance (AGENTS.md) that is current, a public repository's GitHub metadata (description, topics, homepage), or CI caching that actually hits. These are not deep engineering, and they are cross-cutting concerns all the same: cheap to keep visible across many repositories, and easy to let slide in each one.
- **The people and moments** in [../reservoirs.md](../reservoirs.md): the operator at 3am, the fresh agent, the deploy, the restore, the second instance.
- **Across the projects:** what several of them share, such as schedules, caches, published files or retries, and what one kind teaches about another.
- **The seed cards** in [../catalogue.md](../catalogue.md). They show the level every card must sit at. A seed that software like this meets is a good card: use its rule as written. A seed this kind of software does not meet stays out.

Write each candidate the way a concern is written: from a concrete story, back to the general concept, then forward to the smallest rule that is plain, agreeable to most engineers, and literally true.

## 2. Keep each rule at its most general true form

**A card is the most general form of the rule.** The person keeps the rule and reviews for it again later, in these projects and others, and each review should find different instances. A rule that only fits one situation finds nothing else.

For each candidate, name the concept a computer scientist would use: ownership and lifetimes, parsing at the boundary, monotonic time, idempotency, staleness. State the rule at that level. Then test it: in a different project of a different kind, would a review for this rule find things? If not, climb. Stop before it turns vague: "resources should be handled correctly" finds nothing. The seeds are the right height.

**General, but never contorted: name things by their everyday names.** Being generic does not mean giving up being concrete. A reader who already knows the thing must understand the rule on first reading, without parsing it. Unfamiliar words are fine, because the justification explains them. Abstract paraphrase is not. If the thing has a name ("signing keys", "time zones", "retries", "the cache"), use the name. In Max's words, about a card titled "The identity that authorises updates should outlive the machine that builds them": "'Android app signing keys' is perfectly fine! Or just 'Signing keys'. While we are trying to be *generic*, we don't want to forgo being *concrete*. We shan't contort our speech." Write "Signing keys should not exist only on the machine that builds", not "the identity that authorises updates".

Merge candidates that are the same rule. A rule that applies to several of the projects is worth more than one that applies to a single project.

## 3. Filter hard

Drop a candidate when any of these is true:
- **It is on the default path.** If leaving it unmanaged would leave something obviously broken, the ordinary path already handles it.
- **It is a lecture.** Common practice told to an expert. Judge this by the reader.
- **It is a bug or a gotcha.** This is not a bug hunt: a defect is not a concern. Neither is a platform trap any agent could find without taste.
- **It is about change** (migrations, API changes, renames). A change is not a property of the code.
- **It is compliance, security or obligation** ("is there a linter", "does every PR get two approvals").
- **It is too absolute.** "Every" and "never" need something close to a proof. Prefer "generally" or "should not" where there are real exceptions, and name them in a story.
- **It needs a re-architecture to act on.** Leave it out of the cards.

## 4. Rank by leverage

Leverage is value against the effort of acting. The cheap rule that pays across several of the projects comes first. The important but expensive one does not lead.

## 5. Present

Lead with the cards, highest leverage first. Typically a dozen or more survive the filter; never pad.

Offer to establish any card as a persistent concern. Do not do it unasked.
