//! `crosscut unpack`: each project is described in a few sentences, an agent proposes
//! concerns from those descriptions alone, the person accepts or declines each one, and
//! accepted concerns become concern directories whose prompt check reviews the code.
//!
//! Unpacking never sees the code. A concern found in code starts life as one bug and has to
//! be argued back up to a rule; a concern unpacked from a description can only be the rule. The
//! review is what finds where each rule cashes out.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::harness::{self, Harness};
use crate::model::{self, Project};

const CARDS_OPEN: &str = "<crosscut-cards>";
const CARDS_CLOSE: &str = "</crosscut-cards>";
const ORDER_OPEN: &str = "<crosscut-order>";
const ORDER_CLOSE: &str = "</crosscut-order>";

/// What one unpacking run proposes. It is written to `crosscut/landscape.json`, so the cards
/// can be floated again without another run.
#[derive(Serialize, Deserialize)]
pub struct Landscape {
    pub cards: Vec<Card>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Card {
    pub slug: String,
    pub title: String,
    pub justification: String,
    /// Stakeholder stories from software of any kind.
    pub stories: Vec<String>,
    /// A stakeholder story for each project it applies to.
    pub here: Vec<Here>,
    pub how_to_review: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Here {
    pub project: String,
    pub story: String,
}

impl Card {
    pub fn projects(&self) -> Vec<&str> {
        self.here.iter().map(|h| h.project.as_str()).collect()
    }
}

pub enum Answer {
    Accept,
    Decline(String),
    Skip,
    Quit,
}

/// The prompt for one unpacking run: the unpack mode with the doctrine and seeds, then the
/// projects' descriptions, what is already decided, and the answer format.
pub fn prompt(
    mode_prompt: &str,
    about: &[(String, String)],
    known: &[String],
    declined: &str,
) -> String {
    let projects: String = about
        .iter()
        .map(|(name, text)| format!("### `{name}`\n\n{}\n\n", text.trim()))
        .collect();
    let known = if known.is_empty() {
        "none yet".into()
    } else {
        known.iter().map(|t| format!("\n- {t}")).collect::<String>()
    };
    let declined = if declined.trim().is_empty() {
        "none yet"
    } else {
        declined.trim()
    };
    format!(
        "{mode_prompt}\n\n---\n\n# Your task\n\n\
         Unpack concerns for these projects, as the unpack mode above describes. You have \
         their descriptions and nothing else: no code, and no tools. No human is available \
         until you finish: the person will see your cards one at a time, and accept or decline \
         each. A reviewer with the code will then look for each accepted rule in every \
         project.\n\n\
         ## The projects\n\n{projects}\
         Concerns the person already keeps, which you must not propose again: {known}\n\n\
         Concerns the person declined, with their reasons. Do not propose these again, and \
         learn from the reasons: {declined}\n\n\
         ## Your answer\n\n\
         End your reply with one JSON object between `{CARDS_OPEN}` and `{CARDS_CLOSE}`, and \
         nothing after it:\n\n\
         ```json\n\
         {{\n  \
           \"cards\": [\n    {{\n      \
             \"slug\": \"kebab-case-name\",\n      \
             \"title\": \"The rule, in the rule form\",\n      \
             \"justification\": \"The concept behind it, in the terms a computer scientist would use, why it holds, and where it does not apply.\",\n      \
             \"stories\": [\"As <stakeholder>, <situation and want>, so I need <what the rule gives>. From software of any kind, never these projects.\", \"...\"],\n      \
             \"here\": [{{\"project\": \"the name of a project it applies to\", \"story\": \"As <stakeholder of this project>, ..., so I need ...\"}}],\n      \
             \"how_to_review\": \"How a reviewer with the code finds every instance of this in a project: what to search for, and what to read\"\n    \
           }}\n  \
         ]\n\
         }}\n\
         ```\n\n\
         Order the cards by leverage, highest first.\n",
    )
}

/// The prompt that puts unpacked cards in order of leverage. Its answer is only an order: no
/// text it writes reaches the person.
pub fn order_prompt(cards: &[Card], about: &[(String, String)], projects: &[Project]) -> String {
    let projects: String = projects
        .iter()
        .map(|p| {
            let text = about
                .iter()
                .find(|(n, _)| *n == p.name)
                .map_or("", |(_, t)| t.as_str());
            format!("- `{}` at `{}`: {}\n", p.name, p.dir.display(), text.trim())
        })
        .collect();
    let cards: String = cards
        .iter()
        .map(|c| {
            let here: Vec<String> =
                c.here.iter().map(|h| format!("{}: {}", h.project, h.story)).collect();
            format!("- `{}`: {}. {}\n", c.slug, c.title, here.join(" "))
        })
        .collect();
    format!(
        "Put these concerns in order of leverage across these projects, highest first.\n\n\
         Leverage is the value of acting on a concern against the effort of acting on it, \
         summed across the projects. A cheap change that pays in several projects comes \
         first. An important concern that is expensive to act on does not lead. This is \
         leverage, not importance.\n\n\
         Value is what the change buys the stakeholders: the people who use the software, \
         and the developers and coding agents who work on it, as the descriptions say who \
         they are. For a personal app on one phone, that is its one owner, and the agents \
         that change it. Hygiene that guards against a risk none of them runs, such as \
         separating development and production installs of a sideloaded app with one user, \
         buys nothing, and goes last. Max, on exactly that card ranked first: \"who cares \
         for a sideloaded app on one phone? This is incredibly low leverage.\" A \
         security-flavoured change can still earn its place when it lets an agent work \
         safely and comfortably, but rarely for much else in software like this.\n\n\
         Take a quick look at the code to judge effort and reach: a few searches and a few \
         files, not a review. Spend at most a few minutes. Do not modify anything.\n\n\
         ## The projects\n\n{projects}\n## The concerns\n\n{cards}\n\
         ## Your answer\n\n\
         End your reply with every slug above, each exactly once, one per line, between \
         `{ORDER_OPEN}` and `{ORDER_CLOSE}`. Nothing else you write is shown to anyone.\n"
    )
}

/// The order in an agent's reply, as indices into `cards`. Anything but a reordering of
/// exactly these cards is refused.
pub fn parse_order(reply: &str, cards: &[Card]) -> Result<Vec<usize>, String> {
    let start = reply
        .rfind(ORDER_OPEN)
        .ok_or("the agent's reply had no <crosscut-order> answer")?;
    let rest = &reply[start + ORDER_OPEN.len()..];
    let body = &rest[..rest.find(ORDER_CLOSE).unwrap_or(rest.len())];
    let mut order = Vec::new();
    for slug in body
        .lines()
        .map(|l| l.trim().trim_matches(['`', '-', ' ']))
        .filter(|l| !l.is_empty())
    {
        let i = cards
            .iter()
            .position(|c| c.slug == slug)
            .ok_or_else(|| format!("the order names `{slug}`, which is not a card"))?;
        if order.contains(&i) {
            return Err(format!("the order names `{slug}` twice"));
        }
        order.push(i);
    }
    if order.len() != cards.len() {
        return Err(format!(
            "the order has {} of the {} cards",
            order.len(),
            cards.len()
        ));
    }
    Ok(order)
}

/// The prompt that describes one project, for unpacking to work from.
pub fn describe_prompt(project: &Project) -> String {
    format!(
        "Describe the project in `{}`, your working directory, in two or three plain \
         sentences. Say what kind of software it is, who or what uses it, what state it holds, \
         how it is built, deployed and run, and its main languages and tools. For example: \
         \"A streaming data API for a stock exchange, serving delayed prices to websites. \
         Python, built with Make and Nix, deployed as a container on AWS.\"\n\n\
         Read only what a newcomer would read first: the README, agent guidance such as \
         AGENTS.md, manifests, and build, CI and deploy files. Do not read the source code, and \
         do not judge the project. Do not modify anything.\n\n\
         Reply with the sentences and nothing else.",
        project.dir.display()
    )
}

/// The cards in an agent's reply: the last `<crosscut-cards>` block.
pub fn parse(reply: &str) -> Result<Landscape, String> {
    let start = reply
        .rfind(CARDS_OPEN)
        .ok_or("the agent's reply had no <crosscut-cards> answer")?;
    let rest = &reply[start + CARDS_OPEN.len()..];
    let body = rest[..rest.find(CARDS_CLOSE).unwrap_or(rest.len())].trim();
    let body = body
        .strip_prefix("```json")
        .or_else(|| body.strip_prefix("```"))
        .map_or(body, |b| b.trim_end().trim_end_matches("```"));
    serde_json::from_str(body).map_err(|e| format!("the agent's cards are not valid JSON: {e}"))
}

/// Runs the agent in `dir`, showing what it looks at as it goes, and returns its final reply.
/// It may read the projects in `access`; with none, a Claude agent gets no tools at all.
pub fn run(
    progress: &(dyn Fn(&str) + Sync),
    harness: &Harness,
    prompt: &str,
    model: Option<&str>,
    prepare: &dyn Fn(&mut Command),
    access: &[Project],
    timeout: Duration,
) -> Result<String, String> {
    let (mut cmd, stdin) = harness.command(prompt, model)?;
    prepare(&mut cmd);
    let claude = matches!(harness, Harness::Claude);
    if claude {
        cmd.args(["--output-format", "stream-json", "--verbose"]);
        if access.is_empty() {
            cmd.args(["--tools", ""]);
        } else {
            cmd.arg("--add-dir").args(access.iter().map(|p| &p.dir));
        }
    }
    let projects = access;
    let started = Instant::now();
    let mut steps = 0;
    let mut result: Option<Result<String, String>> = None;
    // Seconds since the start at which something was last shown, so a long silent think
    // still shows it is alive.
    let shown = AtomicU64::new(0);
    let finished = AtomicBool::new(false);
    let out = std::thread::scope(|scope| {
        scope.spawn(|| {
            while !finished.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(200));
                let now = started.elapsed().as_secs();
                if now >= shown.load(Ordering::Relaxed) + 30 {
                    progress(&format!("{now:>4}s  still thinking"));
                    shown.store(now, Ordering::Relaxed);
                }
            }
        });
        let out = harness::run_streaming(cmd, stdin.then_some(prompt), timeout, |line| {
            if !claude {
                return;
            }
            let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
                return;
            };
            match event["type"].as_str() {
                Some("assistant") => {
                    for block in event["message"]["content"].as_array().into_iter().flatten() {
                        if block["type"] == "tool_use" {
                            steps += 1;
                            let now = started.elapsed().as_secs();
                            progress(&format!("{now:>4}s  {}", describe_tool(block, projects)));
                            shown.store(now, Ordering::Relaxed);
                        }
                    }
                }
                Some("result") => {
                    let text = event["result"].as_str().unwrap_or_default().to_string();
                    result = Some(if event["is_error"] == true {
                        Err(format!("the agent failed: {text}"))
                    } else {
                        Ok(text)
                    });
                }
                _ => {}
            }
        });
        finished.store(true, Ordering::Relaxed);
        out
    })?;
    if claude {
        let secs = started.elapsed().as_secs();
        match steps {
            0 => progress(&format!("{secs:>4}s  done")),
            n => progress(&format!("{secs:>4}s  done, after {n} steps")),
        }
        return result.unwrap_or_else(|| Err("the agent finished without a result".into()));
    }
    Ok(out)
}

/// One line for one tool call, with project paths shortened to project names.
fn describe_tool(block: &serde_json::Value, projects: &[Project]) -> String {
    let input = &block["input"];
    let name = block["name"].as_str().unwrap_or("?");
    let detail = ["file_path", "pattern", "command", "path", "url", "query"]
        .iter()
        .filter_map(|key| input[key].as_str())
        .collect::<Vec<_>>()
        .join("  ");
    let mut detail = detail.replace('\n', " ");
    for p in projects {
        detail = detail.replace(&*p.dir.to_string_lossy(), &p.name);
    }
    let detail: String = detail.chars().take(110).collect();
    format!("{name:<5} {detail}")
}

/// The concern directory for an accepted card: its definition, and a prompt check that
/// reviews a project for it.
pub fn accept(root: &Path, card: &Card, today: &str) -> Result<PathBuf, String> {
    let dir = root.join("crosscut/concerns").join(&card.slug);
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let stories: String = card.stories.iter().map(|s| format!("- {s}\n")).collect();
    let here: String = card
        .here
        .iter()
        .map(|h| format!("- **{}**: {}\n", h.project, h.story))
        .collect();
    let definition = format!(
        "# {}\n\n{}\n\n## Stakeholder stories\n\n{stories}\n## Here\n\n{here}\n\
         Accepted {today}.\n\n## Decisions\n",
        card.title, card.justification
    );
    let check = format!(
        "Review this project against the rule in the concern, as a reviewer who wants every \
         real instance and no false ones. Where the concern says the rule does not apply, \
         respect that.\n\n{}\n\n\
         Status: `yes` when the rule holds everywhere it applies, `partly` when it holds in \
         some places and not others, `missing` when it does not hold where it matters most, \
         `n/a` when the rule has nothing to apply to in this project.\n",
        card.how_to_review
    );
    write(&dir.join("concern.md"), &definition)?;
    write(&dir.join("check.md"), &check)?;
    Ok(dir)
}

/// Appends a declined card, with the reason, to `crosscut/declined.md`.
pub fn decline(root: &Path, card: &Card, reason: &str, today: &str) -> Result<(), String> {
    let path = root.join("crosscut/declined.md");
    let mut text = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        "# Declined\n\nConcerns proposed and declined. Unpacking reads this, and does not \
         propose them again.\n\n"
            .into()
    });
    let reason = if reason.is_empty() {
        "no reason given"
    } else {
        reason
    };
    text += &format!("- {}: {reason} ({today})\n", card.title);
    write(&path, &text)
}

/// Adds directories to `crosscut/projects`, relative to the root where possible.
pub fn add_projects(root: &Path, dirs: &[PathBuf]) -> Result<(), String> {
    let path = root.join("crosscut/projects");
    let mut text = std::fs::read_to_string(&path).unwrap_or_default();
    let listed: Vec<PathBuf> = model::projects(root)
        .map(|ps| ps.into_iter().map(|p| p.dir).collect())
        .unwrap_or_default();
    for dir in dirs {
        let dir = dir
            .canonicalize()
            .map_err(|e| format!("cannot find project {}: {e}", dir.display()))?;
        if !dir.is_dir() {
            return Err(format!("{} is not a directory", dir.display()));
        }
        if listed.contains(&dir) {
            continue;
        }
        let line = dir.strip_prefix(root).unwrap_or(&dir).display().to_string();
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text += &line;
        text.push('\n');
    }
    write(&path, &text)
}

/// A project's description, if it has one: `crosscut/about/<project>.md`.
pub fn about(root: &Path, project: &str) -> Option<String> {
    std::fs::read_to_string(about_path(root, project))
        .ok()
        .filter(|t| !t.trim().is_empty())
}

pub fn about_path(root: &Path, project: &str) -> PathBuf {
    root.join("crosscut/about").join(format!("{project}.md"))
}

pub fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cards(slugs: &[&str]) -> Vec<Card> {
        slugs
            .iter()
            .map(|slug| Card {
                slug: slug.to_string(),
                title: String::new(),
                justification: String::new(),
                stories: vec![],
                here: vec![],
                how_to_review: String::new(),
            })
            .collect()
    }

    #[test]
    fn an_order_must_be_exactly_a_reordering_of_the_cards() {
        let cards = cards(&["a", "b", "c"]);
        let order = |body: &str| {
            parse_order(
                &format!("prose\n{ORDER_OPEN}\n{body}\n{ORDER_CLOSE}"),
                &cards,
            )
        };
        assert_eq!(order("c\n- `a`\nb"), Ok(vec![2, 0, 1]));
        assert!(order("c\na").unwrap_err().contains("2 of the 3"));
        assert!(order("c\na\nb\na").unwrap_err().contains("twice"));
        assert!(order("c\na\nb\nd: a new concern")
            .unwrap_err()
            .contains("not a card"));
        assert!(parse_order("no tags", &cards).is_err());
    }
}
