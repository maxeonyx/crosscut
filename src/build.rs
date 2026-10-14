//! Making checks permanent. After the first review, an agent recommends for each accepted
//! cross-cutting concern the cheapest kind of check that would be faithful, the person keeps
//! or changes each recommendation, and agents build the ones chosen.

use crate::model::{Concern, Project};

const TIERS_OPEN: &str = "<crosscut-tiers>";
const TIERS_CLOSE: &str = "</crosscut-tiers>";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tier {
    /// An existing tool, run by a script.
    Tool,
    /// A script an agent writes.
    Custom,
    /// A review prompt, checked in.
    Prompt,
}

impl Tier {
    pub const ALL: [Tier; 3] = [Tier::Tool, Tier::Custom, Tier::Prompt];

    pub fn key(self) -> &'static str {
        match self {
            Tier::Tool => "tool",
            Tier::Custom => "custom",
            Tier::Prompt => "prompt",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Tier::Tool => "Integrate a standard tool",
            Tier::Custom => "Build a custom check",
            Tier::Prompt => "Check in an agentic review prompt",
        }
    }

    pub fn meaning(self) -> &'static str {
        match self {
            Tier::Tool => "an existing linter or checker, run by a script",
            Tier::Custom => "a small script an agent writes, with no AI when it runs",
            Tier::Prompt => "review instructions written for these projects, for an agent",
        }
    }

    pub fn parse(key: &str) -> Option<Tier> {
        Tier::ALL.into_iter().find(|t| t.key() == key.trim())
    }
}

pub struct Recommendation {
    pub title: String,
    pub tier: Tier,
    pub why: String,
}

/// What each project looked like to the first review, for an agent that builds on it.
fn review_so_far(concern: &Concern) -> String {
    concern
        .observations()
        .iter()
        .map(|o| {
            let findings = std::fs::read_to_string(
                concern
                    .dir
                    .join("findings")
                    .join(format!("{}.md", o.project)),
            )
            .unwrap_or_default();
            format!(
                "### {}: {}: {}\n\n{}\n\n",
                o.project,
                o.status,
                o.evidence,
                findings.trim()
            )
        })
        .collect()
}

fn project_list(projects: &[Project]) -> String {
    projects
        .iter()
        .map(|p| format!("- `{}` at `{}`\n", p.name, p.dir.display()))
        .collect()
}

pub fn recommend_prompt(concerns: &[Concern], projects: &[Project]) -> String {
    let concerns: String = concerns
        .iter()
        .map(|c| {
            format!(
                "## `{}`\n\n{}\n\n### The first review\n\n{}",
                c.slug,
                c.definition.trim(),
                review_so_far(c)
            )
        })
        .collect();
    format!(
        "Recommend how each cross-cutting concern below should be checked from now on, in \
         these projects. The person accepted them, and an agent has reviewed each one once. \
         For each, choose the cheapest kind of check that would be faithful:\n\n\
         - `tool`: integrate a standard tool. An existing linter rule, compiler setting or \
         checker already decides it. Name the tool.\n\
         - `custom`: build a custom check. A small script can decide it by reading files or \
         running commands, without judgment. Say what it would look at.\n\
         - `prompt`: check in an agentic review prompt. Deciding it needs judgment, so an \
         agent keeps reviewing it, with instructions written for these projects.\n\n\
         A check runs often, so it must be cheap: it reads files and runs quick commands such \
         as git and grep. Choose `tool` only for a tool already installed here or already \
         used by these projects, and `custom` only when reading the files decides it. If \
         deciding needs the project built, installed or run, choose `prompt`. A script that \
         says `yes` when the rule is not met is worse than an agent review. You may read the \
         projects, but keep it quick.\n\n\
         # The projects\n\n{}\n# The cross-cutting concerns\n\n{concerns}\n\
         # Your answer\n\n\
         For every concern, write one line: its slug, a colon, `tool`, `custom` or `prompt`, \
         a colon, and one short sentence saying why, naming the tool or what the script looks \
         at. End your reply with those lines, every concern exactly once, between \
         `{TIERS_OPEN}` and `{TIERS_CLOSE}`.\n",
        project_list(projects)
    )
}

pub fn parse_recommendations(
    reply: &str,
    concerns: &[Concern],
) -> Result<Vec<Recommendation>, String> {
    let start = reply
        .rfind(TIERS_OPEN)
        .ok_or("the agent's reply had no <crosscut-tiers> answer")?;
    let rest = &reply[start + TIERS_OPEN.len()..];
    let body = &rest[..rest.find(TIERS_CLOSE).unwrap_or(rest.len())];
    let mut found: Vec<Option<(Tier, String)>> = concerns.iter().map(|_| None).collect();
    for line in body.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let mut parts = line.splitn(3, ':');
        let slug = parts
            .next()
            .unwrap_or_default()
            .trim()
            .trim_matches(['`', '-', ' ']);
        let tier = parts.next().unwrap_or_default().trim().trim_matches('`');
        let why = parts.next().unwrap_or_default().trim();
        let i = concerns
            .iter()
            .position(|c| c.slug == slug)
            .ok_or_else(|| format!("the recommendation names `{slug}`, which is not accepted"))?;
        let tier =
            Tier::parse(tier).ok_or_else(|| format!("`{tier}` is not tool, custom or prompt"))?;
        if found[i].replace((tier, why.to_string())).is_some() {
            return Err(format!("the recommendation names `{slug}` twice"));
        }
    }
    concerns
        .iter()
        .zip(found)
        .map(|(c, f)| {
            let (tier, why) =
                f.ok_or_else(|| format!("the recommendation leaves out `{}`", c.slug))?;
            Ok(Recommendation {
                title: c.name().to_string(),
                tier,
                why,
            })
        })
        .collect()
}

/// The prompt for an agent that builds one check, working in the concern's own directory.
pub fn build_prompt(concern: &Concern, tier: Tier, why: &str, projects: &[Project]) -> String {
    let task = match tier {
        Tier::Tool | Tier::Custom => format!(
            "Write `check` in your working directory: an executable, with a shebang, in bash \
             or python3, that checks one project for this cross-cutting concern. {}: {why}\n\n\
             CrossCut runs it with the project's directory as the working directory and these \
             environment variables: `CROSSCUT_PROJECT` (its name), `CROSSCUT_PROJECT_DIR`, \
             `CROSSCUT_PROJECTS` (every project, one `name<TAB>dir` per line) and \
             `CROSSCUT_CONCERN_DIR`. It prints one line, `status: evidence`. The status is \
             `yes`, `partly`, `missing`, `n/a` or `unknown`, and the evidence is one sentence \
             naming the files it looked at. It must not change anything, use the network or \
             call an AI, and it must be cheap: read files and run quick commands such as git \
             and grep. Never build, install or run the project, or start its toolchain.\n\n\
             Run it in every project below and compare its answers with the first review. \
             Where they disagree, read the code and fix whichever is wrong. It must decide \
             every project here: CrossCut runs it in each one, and throws it away if it \
             answers `unknown` in any.",
            tier.label()
        ),
        Tier::Prompt => format!(
            "Write `check.md` in your working directory: instructions for an agent reviewer \
             who checks one project for this cross-cutting concern. {}: {why}\n\n\
             The reviewer is also given the concern itself. Say what to search for and read in \
             projects like these, how to tell `yes`, `partly` and `missing` apart, and what \
             does not count. Read the projects and the first review to make it specific to \
             them. Keep it short.",
            tier.label()
        ),
    };
    format!(
        "Build a permanent check for one cross-cutting concern.\n\n{task}\n\n\
         Write nothing outside your working directory, `{}`. End with one line saying what \
         the check looks at.\n\n\
         # The projects\n\n{}\n# The cross-cutting concern\n\n{}\n\n# The first review\n\n{}",
        concern.dir.display(),
        project_list(projects),
        concern.definition.trim(),
        review_so_far(concern)
    )
}

/// Whether the check the agent was asked for is there, made executable if it is a script.
pub fn built(concern: &Concern, tier: Tier) -> Result<(), String> {
    match tier {
        Tier::Prompt => {
            if concern.dir.join("check.md").is_file() {
                Ok(())
            } else {
                Err("the agent wrote no check.md".into())
            }
        }
        Tier::Tool | Tier::Custom => {
            let path = concern.dir.join("check");
            if !path.is_file() {
                return Err("the agent wrote no check".into());
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&path)
                    .map_err(|e| format!("cannot read {}: {e}", path.display()))?
                    .permissions();
                perms.set_mode(perms.mode() | 0o755);
                std::fs::set_permissions(&path, perms)
                    .map_err(|e| format!("cannot make {} executable: {e}", path.display()))?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn concern(slug: &str) -> Concern {
        Concern {
            slug: slug.into(),
            dir: PathBuf::from("/nowhere").join(slug),
            definition: format!("# Rule {slug}\n"),
        }
    }

    #[test]
    fn every_accepted_concern_gets_exactly_one_known_tier() {
        let concerns = [concern("a"), concern("b")];
        let parse = |body: &str| {
            parse_recommendations(
                &format!("x\n{TIERS_OPEN}\n{body}\n{TIERS_CLOSE}"),
                &concerns,
            )
        };
        let recs = parse("a: custom: grep the build file\nb: `prompt`: needs judgment").unwrap();
        assert_eq!(recs[0].tier, Tier::Custom);
        assert_eq!(recs[0].why, "grep the build file");
        assert_eq!(recs[1].title, "Rule b");
        assert!(parse("a: custom: x").is_err(), "b left out");
        assert!(parse("a: custom: x\nb: magic: y").is_err());
        assert!(parse("a: tool: x\nb: tool: y\na: prompt: z").is_err());
    }
}
