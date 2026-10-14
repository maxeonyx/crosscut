# Unpack: "What are we not thinking about?"

This mode proposes concerns that are not on the map yet. Each one is a card the person can accept or decline. It persists nothing unless the person accepts.

**Unpack from a description of each project, never from its code.** A short description is all you need: "a streaming data API for a stock exchange, Python, built with Make and Nix". Software of that kind meets a known landscape of concerns, and your job is to lay that landscape out. Reading the code pulls you towards the bugs in it, and a card that starts from a bug stays a bug report however you word it. Once the person accepts a concern, a reviewer with the code finds where it cashes out in each project. That review is a separate step, and it is not yours.

If you are working inside a session that can see the code, do not read it for this mode. If you have no description, write one first from the README, the agent guidance and the manifests, in two or three sentences, and stop reading there.

A card is a concern in its finished form, as in [../catalogue.md](../catalogue.md):
- **the rule:** one small sentence, in the rule form ("X should Y"), true as written, hedged only where real exceptions exist;
- **the justification:** the concept behind it, named as a computer scientist would name it (time-of-check to time-of-use, the birthday bound, survivorship bias, identity elements), why the rule follows, and where it does not apply;
- **stakeholder stories:** two or three, from software of any kind, never from these projects;
- **here:** one stakeholder story for each of the projects it applies to;
- **how to review:** how a reviewer with the code would find every instance of it.

### Stakeholder stories

A concern exists because someone wants something from software. A stakeholder story says who, in what situation, wants what, and what they need from the software because of it:

> As <a stakeholder>, <in a situation> I want <something>, so I need <what the rule gives them>.

The stakeholders are the people who use the software and the developers who work on it. Either can be a human or an agent. Max's example of the form: "As a developer, I repeatedly make changes to the app and need to see the result, so I need this loop to happen as fast as possible."

Write what the stakeholder actually wants, which is the value they get, not a proxy for it. Max, about his own second example ("As a user, I want no disruption to my data, no matter what goes on behind the scenes"): "the user doesn't actually care about the *data*, necessarily, but about the value they get from the app".

The stakeholder stories are general. They come from software of any kind, and they name none of these projects, their features or their data. The seeds' examples show how varied they can be.

**Here** brings the rule to each project, as one more stakeholder story: who wants what from this project, as its description tells it. It is not about the code. You have not seen the code, so you cannot know whether the project already does this, and you must not guess: no bugs, no gaps, no fixes. In Max's words: "don't write about the bug, write about what the stakeholder ... wants from the software". For a ticker feed: "As a trader watching the open, I want every price as it happens, so I need to see when the feed has fallen behind."

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

Descriptions name platforms, and a rule that names a platform's mechanism is usually one step short of its general form. Max, about "Alarms set with the operating system should be set again after anything that can clear them": "Yes, but also we want to generalize it. So it's like state managed outside of the app, you know. Could apply to Windows Registry. It could apply to whatever iOS does." The platform belongs in Here, not in the rule.

For each candidate, name the concept a computer scientist would use: ownership and lifetimes, parsing at the boundary, monotonic time, idempotency, staleness. State the rule at that level. Then test it: in a different project of a different kind, would a review for this rule find things? If not, climb. Stop before it turns vague: "resources should be handled correctly" finds nothing. The seeds are the right height.

**General, but never contorted: name things by their everyday names.** Being generic does not mean giving up being concrete. A reader who already knows the thing must understand the rule on first reading, without parsing it. Unfamiliar words are fine, because the justification explains them. Abstract paraphrase is not. If the thing has a name ("time zones", "retries", "the cache", "a scheduled job"), use the name. In Max's words, about a card that called a plain thing by an abstract paraphrase: "While we are trying to be *generic*, we don't want to forgo being *concrete*. We shan't contort our speech." The subject of the rule is a plain thing the reader recognises at once: "Retries", "A scheduled job", "The cache". Never a noun phrase built from a clause, such as "Work a caller was told had succeeded should not be lost", which has to be untangled before it means anything; say "Acknowledged writes should survive a crash" instead.

Merge candidates that are the same rule. A rule that applies to several of the projects is worth more than one that applies to a single project.

## 3. Filter hard

Drop a candidate when any of these is true:
- **It is on the default path.** If leaving it unmanaged would leave something obviously broken, the ordinary path already handles it.
- **It is a lecture.** Common practice told to an expert. Judge this by the reader.
- **It is a niche hazard.** A rare way for one thing to go wrong is a bug in waiting, however it is worded. Max: "Niche hazard is a red flag. It's bug flavored."
- **It is a bug or a gotcha.** This is not a bug hunt: a defect is not a concern. Neither is a platform trap any agent could find without taste.
- **It is about change** (migrations, API changes, renames). A change is not a property of the code.
- **It is compliance, security or obligation** ("is there a linter", "does every PR get two approvals"). That includes security-flavoured hygiene that buys the stakeholders nothing. Stakeholders are the people who use the software and the developers and coding agents who work on it. Separating development and production installs of an app with one user is not worth a card; a change that lets an agent work on it safely can be.
- **It is too absolute.** "Every" and "never" need something close to a proof. Prefer "generally" or "should not" where there are real exceptions, and name them in the justification.
- **It needs a re-architecture to act on.** Leave it out of the cards.

## 4. Rank by leverage

Leverage is value against the effort of acting. The cheap rule that pays across several of the projects comes first. The important but expensive one does not lead.

Broad, everyday concerns usually beat clever ones: how fast a developer or agent sees the result of a change, whether the commands to build and check work as written, whether a failure is noticed. Max, about a security-flavoured card ranked first ("Signing keys should not exist only on the machine that builds", for two personal apps): "It's security flavored, first of all, which is a red flag. And it simply isn't actionable or valuable for the project. The signing key for a sideloaded personal app... doesn't really matter?" And: "There are way better things like feedback loop", "frankly, a whole bunch of generic stuff is basically better."

## 5. Present

Lead with the cards, highest leverage first. Typically a dozen or more survive the filter; never pad.

Offer to establish any card as a persistent concern. Do not do it unasked.
