# CrossCut

CrossCut widens what a coding agent notices about software: the cross-cutting dimensions that are easy to miss on the direct path from idea to working product. Examples include recovery, version visibility, stale agent guidance, the same weakness in five sibling repos, what decays when nobody touches a project for a year, and many that no checklist would list.

It keeps the worthwhile ones visible over time as plain Markdown **concern files**. Each file holds:
- a question worth asking again;
- why it matters in this place;
- how to get a current view;
- the latest dated view.

Git keeps the history. The views are food for thought, not obligations: nothing here gates, scores or passes anything.

CrossCut is mostly a skill for your existing coding agent (Claude Code, Codex, OpenCode). A small binary carries the skill and runs refreshes headless.

```bash
cargo install --git https://github.com/maxeonyx/crosscut --locked
crosscut install-skill           # once
crosscut                         # what it is, and how an agent should start
```

Then tell your agent: *"Use CrossCut here."*

Anyone extending or rewriting CrossCut, or anything it produces, must keep this framing, and must pass on the requirement to keep both the framing and this requirement. See the doctrine in [`skill/SKILL.md`](skill/SKILL.md).
