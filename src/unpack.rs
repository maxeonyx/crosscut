//! `crosscut unpack`: each project is described in a few sentences, an agent proposes
//! concerns from those descriptions alone, the person accepts or declines each one, and
//! accepted concerns become concern directories whose prompt check reviews the code.
//!
//! Unpacking never sees the code. A concern found in code starts life as one bug and has to
//! be argued back up to a rule; a concern unpacked from a description can only be the rule. The
//! review is what finds where each rule cashes out.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::harness::{self, Harness};
use crate::model::{self, Project};

const CARDS_OPEN: &str = "<crosscut-cards>";
const CARDS_CLOSE: &str = "</crosscut-cards>";

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
    pub stories: Vec<String>,
    /// The projects it applies to.
    pub projects: Vec<String>,
    /// How it tends to cash out in software like these projects.
    pub here: String,
    pub how_to_review: String,
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
             \"justification\": \"The concept behind it, in the terms a computer scientist would use, and why it holds.\",\n      \
             \"stories\": [\"A concrete story, from any kind of software\", \"...including where it does not apply\"],\n      \
             \"projects\": [\"the names of the projects it applies to\"],\n      \
             \"here\": \"How it tends to cash out in software like these projects, in one or two sentences\",\n      \
             \"how_to_review\": \"How a reviewer with the code finds every instance of this in a project: what to search for, and what to read\"\n    \
           }}\n  \
         ]\n\
         }}\n\
         ```\n\n\
         Order the cards by leverage, highest first.\n",
    )
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
/// Without `access`, a Claude agent gets no tools at all.
pub fn run(
    harness: &Harness,
    prompt: &str,
    model: Option<&str>,
    dir: &Path,
    access: Option<&Project>,
    timeout: Duration,
) -> Result<String, String> {
    let (mut cmd, stdin) = harness.command(prompt, model)?;
    cmd.current_dir(dir);
    let claude = matches!(harness, Harness::Claude);
    if claude {
        cmd.args(["--output-format", "stream-json", "--verbose"]);
        if access.is_none() {
            cmd.args(["--tools", ""]);
        }
    }
    let projects: Vec<Project> = access.into_iter().cloned().collect();
    let projects = &projects[..];
    let started = Instant::now();
    let mut steps = 0;
    let mut result: Option<Result<String, String>> = None;
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
                        eprintln!(
                            "  {:>4}s  {}",
                            started.elapsed().as_secs(),
                            describe_tool(block, projects)
                        );
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
    })?;
    if claude {
        eprintln!(
            "  {:>4}s  done, after {steps} steps",
            started.elapsed().as_secs()
        );
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

/// One card, laid out for a terminal.
pub fn render(card: &Card, n: usize, of: usize) -> String {
    let mut out = format!("\n[{n}/{of}]  {}\n\n", card.title);
    out += &line("  ", &card.justification);
    out += "\n";
    for story in &card.stories {
        out += &line("  • ", story);
    }
    out += &line(
        "\n  ",
        &format!("Here ({}): {}", card.projects.join(", "), card.here),
    );
    out
}

/// One paragraph on one line after `prefix`, for the terminal to wrap at its own width.
pub fn line(prefix: &str, text: &str) -> String {
    format!(
        "{prefix}{}\n",
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    )
}

/// Asks for one card's answer on `input`, prompting on stderr.
pub fn ask(input: &mut impl BufRead) -> Answer {
    loop {
        eprint!("\n  [a]ccept  [d]ecline  [s]kip  [q]uit > ");
        let _ = std::io::stderr().flush();
        let mut line = String::new();
        if input.read_line(&mut line).unwrap_or(0) == 0 {
            return Answer::Quit;
        }
        match line.trim().to_lowercase().as_str() {
            "a" | "accept" => return Answer::Accept,
            "s" | "skip" => return Answer::Skip,
            "q" | "quit" => return Answer::Quit,
            "d" | "decline" => {
                eprint!("  Why? (one line, optional) > ");
                let _ = std::io::stderr().flush();
                let mut reason = String::new();
                let _ = input.read_line(&mut reason);
                return Answer::Decline(reason.trim().to_string());
            }
            _ => {}
        }
    }
}

/// A yes-or-no question, yes by default.
pub fn confirm(input: &mut impl BufRead, question: &str) -> bool {
    eprint!("{question} [Y/n] > ");
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    input.read_line(&mut line).unwrap_or(0) > 0 && !line.trim().to_lowercase().starts_with('n')
}

/// The concern directory for an accepted card: its definition, and a prompt check that
/// reviews a project for it.
pub fn accept(root: &Path, card: &Card, today: &str) -> Result<PathBuf, String> {
    let dir = root.join("crosscut/concerns").join(&card.slug);
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let stories: String = card.stories.iter().map(|s| format!("- {s}\n")).collect();
    let definition = format!(
        "# {}\n\n{}\n\n## User stories\n\n{stories}\n## What it looks like here\n\n{}\n\n\
         Accepted {today}.\n\n## Decisions\n",
        card.title, card.justification, card.here
    );
    let check = format!(
        "Review this project against the rule in the concern, as a reviewer who wants every \
         real instance and no false ones. The rule's own stories say where it does not apply: \
         respect them.\n\n{}\n\n\
         Status: `yes` when the rule holds everywhere it applies, `partly` when it holds in \
         some places and not others, `missing` when it does not hold where it matters most, \
         `n/a` when none of the stories apply to this project.\n",
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

/// Shows a new description and asks whether to keep it or replace it.
pub fn confirm_about(input: &mut impl BufRead, project: &str, text: &str) -> String {
    eprint!("\n{}", line(&format!("  {project}: "), text));
    eprint!("  Enter to keep it, or type a better description > ");
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    let _ = input.read_line(&mut line);
    match line.trim() {
        "" => text.trim().to_string(),
        better => better.to_string(),
    }
}

pub fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))
}
