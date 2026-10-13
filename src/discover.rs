//! `crosscut discover`: an agent proposes concerns for a set of projects, the person accepts
//! or declines each one, and accepted concerns become concern directories that the prompt
//! check then reviews.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::harness::{self, Harness};
use crate::model::{self, Project};

const CARDS_OPEN: &str = "<crosscut-cards>";
const CARDS_CLOSE: &str = "</crosscut-cards>";

/// What one discovery run proposes. It is written to `crosscut/landscape.json`, so the cards
/// can be floated again without another run.
#[derive(Serialize, Deserialize)]
pub struct Landscape {
    pub cards: Vec<Card>,
    #[serde(default)]
    pub also_noticed: Vec<String>,
    #[serde(default)]
    pub could_not_see: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Card {
    pub slug: String,
    pub title: String,
    pub justification: String,
    pub stories: Vec<String>,
    pub instances: Vec<Instance>,
    pub effort: String,
    pub how_to_review: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Instance {
    pub project: String,
    pub location: String,
    pub what: String,
}

pub enum Answer {
    Accept,
    Decline(String),
    Skip,
    Quit,
}

/// The prompt for one discovery run: the discover mode with the doctrine and seeds, then this
/// run's projects, what is already decided, and the answer format.
pub fn prompt(
    mode_prompt: &str,
    projects: &[Project],
    known: &[String],
    declined: &str,
    today: &str,
) -> String {
    let projects: Vec<String> = projects
        .iter()
        .map(|p| format!("- `{}` at `{}`", p.name, p.dir.display()))
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
         Discover concerns for these projects, as the discover mode above describes. No human \
         is available until you finish: the person will see your cards one at a time, and \
         accept or decline each.\n\n\
         {}\n\n\
         Today: {today}.\n\n\
         Do not modify any files. Read, and run only commands that do not change state.\n\n\
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
             \"stories\": [\"A concrete story\", \"...including where it does not apply\"],\n      \
             \"instances\": [{{\"project\": \"project name\", \"location\": \"path/to/file:line\", \"what\": \"what is there, concretely\"}}],\n      \
             \"effort\": \"What acting on it would take here, in one sentence\",\n      \
             \"how_to_review\": \"How a reviewer finds every instance of this in a project: what to search for, and what to read\"\n    \
           }}\n  \
         ],\n  \
           \"also_noticed\": [\"One line each: findings that matter to one project only, and defects you tripped over\"],\n  \
           \"could_not_see\": [\"What you could not see, and what would let you\"]\n\
         }}\n\
         ```\n\n\
         Order the cards by leverage, highest first.\n",
        projects.join("\n"),
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

/// Runs the agent, showing what it looks at as it goes, and returns its final reply.
pub fn run(
    harness: &Harness,
    prompt: &str,
    model: Option<&str>,
    root: &Path,
    projects: &[Project],
    timeout: Duration,
) -> Result<String, String> {
    let (mut cmd, stdin) = harness.command(prompt, model)?;
    cmd.current_dir(root);
    let claude = matches!(harness, Harness::Claude);
    if claude {
        cmd.args(["--output-format", "stream-json", "--verbose", "--add-dir"]);
        cmd.args(projects.iter().map(|p| &p.dir));
    }
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

/// One card, laid out for a terminal `width` columns wide.
pub fn render(card: &Card, n: usize, of: usize, width: usize) -> String {
    let width = width.clamp(60, 110);
    let mut out = format!("\n[{n}/{of}]  {}\n\n", card.title);
    out += &wrap(&card.justification, "  ", "  ", width);
    out += "\n";
    for story in &card.stories {
        out += &wrap(story, "  • ", "    ", width);
    }
    out += "\n  Here:\n";
    for i in &card.instances {
        out += &wrap(
            &format!("{} · {}: {}", i.project, i.location, i.what),
            "  → ",
            "    ",
            width,
        );
    }
    out += &wrap(&format!("Effort: {}", card.effort), "\n  ", "  ", width);
    out
}

pub fn wrap(text: &str, first: &str, rest: &str, width: usize) -> String {
    let mut out = String::new();
    let mut line = first.to_string();
    let mut empty = true;
    for word in text.split_whitespace() {
        if !empty && line.chars().count() + 1 + word.chars().count() > width {
            out += &line;
            out.push('\n');
            line = rest.to_string();
            empty = true;
        }
        if !empty {
            line.push(' ');
        }
        line += word;
        empty = false;
    }
    out += &line;
    out.push('\n');
    out
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

/// Asks which candidates to look at: all by default, or the numbers given.
pub fn choose(input: &mut impl BufRead, candidates: &[PathBuf]) -> Vec<PathBuf> {
    loop {
        eprint!("\nLook at which? Enter for all, or numbers such as `1 3` > ");
        let _ = std::io::stderr().flush();
        let mut line = String::new();
        if input.read_line(&mut line).unwrap_or(0) == 0 || line.trim().is_empty() {
            return candidates.to_vec();
        }
        let picked: Option<Vec<PathBuf>> = line
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|w| !w.is_empty())
            .map(|w| {
                let n: usize = w.parse().ok()?;
                candidates.get(n.checked_sub(1)?).cloned()
            })
            .collect();
        match picked {
            Some(picked) if !picked.is_empty() => return picked,
            _ => eprintln!("  Numbers from 1 to {}, please.", candidates.len()),
        }
    }
}

/// Git repositories at `dir` and up to two levels below it.
pub fn candidates(dir: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, depth: usize, found: &mut Vec<PathBuf>) {
        if dir.join(".git").exists() {
            found.push(dir.to_path_buf());
        }
        if depth == 0 {
            return;
        }
        let mut subdirs: Vec<PathBuf> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .filter(|p| {
                let name = p.file_name().unwrap_or_default().to_string_lossy();
                !name.starts_with('.') && !matches!(&*name, "node_modules" | "target")
            })
            .collect();
        subdirs.sort();
        for sub in subdirs {
            walk(&sub, depth - 1, found);
        }
    }
    let mut found = Vec::new();
    walk(dir, 2, &mut found);
    found
}

/// The concern directory for an accepted card: its definition, with where it was seen, and
/// a prompt check that reviews a project for it.
pub fn accept(root: &Path, card: &Card, today: &str) -> Result<PathBuf, String> {
    let dir = root.join("crosscut/concerns").join(&card.slug);
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let stories: String = card.stories.iter().map(|s| format!("- {s}\n")).collect();
    let seen: String = card
        .instances
        .iter()
        .map(|i| format!("- **{}** `{}`: {}\n", i.project, i.location, i.what))
        .collect();
    let definition = format!(
        "# {}\n\n{}\n\n## User stories\n\n{stories}\n## Seen when it was accepted ({today})\n\n{seen}\n## Decisions\n",
        card.title, card.justification
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
        "# Declined\n\nConcerns proposed and declined. Discovery reads this, and does not \
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

pub fn write(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_are_git_repositories_up_to_two_levels_down() {
        let dir = tempfile::tempdir().unwrap();
        for repo in ["api", "apps/web", "a/b/too-deep", ".hidden/x"] {
            std::fs::create_dir_all(dir.path().join(repo).join(".git")).unwrap();
        }
        std::fs::create_dir_all(dir.path().join("notes")).unwrap();
        let found: Vec<PathBuf> = candidates(dir.path())
            .into_iter()
            .map(|p| p.strip_prefix(dir.path()).unwrap().to_path_buf())
            .collect();
        assert_eq!(found, [PathBuf::from("api"), PathBuf::from("apps/web")]);
    }

    #[test]
    fn choosing_takes_all_by_default_numbers_otherwise_and_asks_again_on_nonsense() {
        let all = [PathBuf::from("a"), PathBuf::from("b"), PathBuf::from("c")];
        assert_eq!(choose(&mut "\n".as_bytes(), &all), all);
        assert_eq!(
            choose(&mut "3, 1\n".as_bytes(), &all),
            [all[2].clone(), all[0].clone()]
        );
        assert_eq!(choose(&mut "4\nx\n2\n".as_bytes(), &all), [all[1].clone()]);
    }
}
