# Discover: "What are we not thinking about?"

This mode proposes concerns that are not on the map yet. Each one is a card the person can accept or decline. It persists nothing unless the person accepts.

A card is a concern in its finished form, as in [../catalogue.md](../catalogue.md):
- **the rule:** one small sentence, in the rule form ("X should Y"), true as written, hedged only where real exceptions exist;
- **the justification:** the concept behind it, named as a computer scientist would name it (time-of-check to time-of-use, the birthday bound, survivorship bias, identity elements), and why the rule follows;
- **stories:** several, concrete and varied, across different kinds of software, and at least one that says where the rule does not apply;
- **instances here:** where these projects meet it or miss it, each with a `path:line` and what is there. This is the reason to act now;
- **the effort** acting on it would take here.

## 1. Look at the real projects

For each project, work out what kind of thing it is, who uses it, what state it holds, and how it ships. Read manifests, CI and deploy configuration, READMEs and agent guidance, and recent history. Then read the code itself, a lot of it: the concerns worth raising live in how the code handles time, failure, retries, caches, identifiers, concurrency, shutdown, data that is copied or derived, and the people and programs at its edges. A landscape built from READMEs is generic.

Do not change anything. Run only commands that look.

## 2. Generate widely

Work from the code outwards, not from a list inwards:
- **Start from something concrete:** a function, a config value, a job definition, a comment that apologises for something. Ask what story it serves badly, and what general rule would have prevented it.
- **Walk the people, moments and shapes** in [../reservoirs.md](../reservoirs.md) against this code.
- **Look across projects:** a capability one has and its siblings lack, one problem solved several ways, something every project needs and none does.
- **Read the seed cards** in [../catalogue.md](../catalogue.md) to learn the grain. A seed belongs in your answer only when this code gives it a real instance. Most of your cards should be rules no seed anticipated.

Then write each candidate the way a concern is written: from the concrete instance, back to the general concept, then forward to the smallest rule that is plain, agreeable to most engineers, and literally true.

## 3. Filter hard

Drop a candidate when any of these is true:
- **It is on the default path.** If leaving it unmanaged would leave something obviously broken, the ordinary path already handles it.
- **It is a lecture.** Common practice told to an expert. Judge this by the reader.
- **It is a bug or a gotcha.** This is not a bug hunt: a defect is not a concern. Neither is a platform trap any agent could find without taste. Mention defects you trip over in one line, under also noticed.
- **It is about change** (migrations, API changes, renames). A change is not a property of the code.
- **It is compliance, security or obligation** ("is there a linter", "does every PR get two approvals").
- **It only fits one project.** The rule must be valuable to software in general. A finding that only this project could care about goes under also noticed.
- **It is too absolute.** "Every" and "never" need something close to a proof. Prefer "generally" or "should not" where there are real exceptions, and name them in a story.
- **It needs a re-architecture to act on.** Say it is important, and leave it out of the cards.

## 4. Rank by leverage

Leverage is value against the effort of acting here. The cheap rule that pays across several projects comes first. The important but expensive one does not lead.

## 5. Present

Lead with the cards, highest leverage first. Typically a dozen or more survive the filter; never pad. Then, briefly: what you also noticed, and what you could not see and what would let you.

Offer to establish any card as a persistent concern. Do not do it unasked.
