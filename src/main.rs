//! `crosscut`: runs concern checks cell by cell (concern × project), shows the map, tests
//! mechanisms against their fixtures, and delivers the CrossCut skill.
//!
//! The model is in docs/design.md: observations (written here) are kept apart from
//! decisions (written by people, in each concern.md), and the map is computed from both.

mod harness;
mod model;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use clap::{Parser, Subcommand};

use harness::Harness;
use model::{Concern, Mechanism, Observation, Project};

/// Every file of the skill, by its path relative to the skill directory.
const SKILL_FILES: &[(&str, &str)] = &[
    ("SKILL.md", include_str!("../skill/SKILL.md")),
    (
        "concern-files.md",
        include_str!("../skill/concern-files.md"),
    ),
    ("catalogue.md", include_str!("../skill/catalogue.md")),
    ("reservoirs.md", include_str!("../skill/reservoirs.md")),
    ("modes/setup.md", include_str!("../skill/modes/setup.md")),
    (
        "modes/discover.md",
        include_str!("../skill/modes/discover.md"),
    ),
    (
        "modes/establish.md",
        include_str!("../skill/modes/establish.md"),
    ),
    (
        "modes/refresh.md",
        include_str!("../skill/modes/refresh.md"),
    ),
    (
        "modes/reconsider.md",
        include_str!("../skill/modes/reconsider.md"),
    ),
    (
        "modes/generalize.md",
        include_str!("../skill/modes/generalize.md"),
    ),
];

const MODES: &[&str] = &[
    "setup",
    "discover",
    "establish",
    "refresh",
    "reconsider",
    "generalize",
];

const CELL_OPEN: &str = "<crosscut-cell>";
const CELL_CLOSE: &str = "</crosscut-cell>";

#[derive(Parser)]
#[command(
    name = "crosscut",
    version,
    about = "Map cross-cutting concerns across your projects, and keep the map current.",
    long_about = "Map cross-cutting concerns across your projects, and keep the map current.\n\n\
        Run `crosscut` with no arguments to read what CrossCut is and how an agent should use it.",
    after_help = "Examples:\n  \
        $ crosscut check\n  \
        $ crosscut map\n  \
        $ crosscut map --project larder\n  \
        $ crosscut test staying-current\n  \
        $ crosscut prompt setup | claude -p"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run concern checks for every project and record what they observe.
    ///
    /// Each cell (concern × project) runs the concern's mechanism: an executable `check`
    /// (tiers 1-2, no model) or, with --agentic, a `check.md` prompt (tier 3, through a
    /// coding harness).
    /// Observations go to each concern's observed.tsv. A row changes only when what was
    /// observed changes, so rerunning an unchanged world produces no diff. Exit status says
    /// whether every cell ran, never what the cells found: 0 all ran, 1 some mechanisms
    /// failed (their previous rows are kept), 2 usage error.
    #[command(
        after_help = "Examples:\n  $ crosscut check\n  $ crosscut check staying-current --project larder\n  \
        $ crosscut check --agentic --harness codex --model gpt-5-mini\n  $ crosscut check --agentic --dry-run"
    )]
    Check {
        /// Concern slugs to check (default: all).
        slugs: Vec<String>,
        /// Also run prompt checks (tier 3), which cost model calls. Without this, their
        /// previous observations stay as they are.
        #[arg(long)]
        agentic: bool,
        #[command(flatten)]
        run: RunArgs,
        /// Show which cells would run, and each prompt, without running anything.
        #[arg(long)]
        dry_run: bool,
    },

    /// Show every concern against every project, as one grid.
    #[command(
        after_help = "A cell shows the human decision if there is one, otherwise the latest\n\
        observation. `*` marks a decision the latest observation contradicts.\n\n\
        Examples:\n  $ crosscut map\n  $ crosscut map --project larder"
    )]
    Map {
        /// Show one project's column, with evidence, instead of the grid.
        #[arg(long)]
        project: Option<String>,
        /// Directory containing crosscut/ (default: search upwards from here).
        #[arg(long)]
        root: Option<PathBuf>,
    },

    /// Run each concern's mechanism against its fixtures and compare with what they expect.
    ///
    /// Fixtures are invented projects in concerns/<slug>/fixtures/<case>/, with an `expect`
    /// file of `<project> <status>` lines. Prompt checks cost model calls, so they are
    /// tested only with --agentic. Exit 1 if any mechanism disagrees with its fixtures.
    #[command(
        after_help = "Examples:\n  $ crosscut test\n  $ crosscut test recovery --agentic --model haiku"
    )]
    Test {
        /// Concern slugs to test (default: all).
        slugs: Vec<String>,
        /// Also run prompt checks against their fixtures.
        #[arg(long)]
        agentic: bool,
        #[command(flatten)]
        run: RunArgs,
    },

    /// Print the full prompt for one mode of work, with the doctrine and references inlined.
    #[command(
        after_help = "Modes: setup, discover, establish, refresh, reconsider, generalize\n\n\
        Examples:\n  $ crosscut prompt setup | claude -p\n  $ crosscut prompt establish > /tmp/prompt.md"
    )]
    Prompt { mode: String },

    /// Install the skill where Claude Code, Codex and OpenCode look for it.
    ///
    /// Defaults to ~/.claude/skills/crosscut and ~/.agents/skills/crosscut. Overwrites only
    /// the files CrossCut ships.
    #[command(
        after_help = "Examples:\n  $ crosscut install-skill\n  $ crosscut install-skill --dir .claude/skills/crosscut"
    )]
    InstallSkill {
        /// Install into this directory instead (repeatable).
        #[arg(long = "dir")]
        dirs: Vec<PathBuf>,
    },
}

#[derive(clap::Args)]
struct RunArgs {
    /// Only these projects (repeatable; default: all in crosscut/projects).
    #[arg(long = "project")]
    projects: Vec<String>,
    /// How many cells to run at once.
    #[arg(long, default_value_t = 6)]
    jobs: usize,
    /// Harness for prompt checks: claude, codex, opencode, or a shell command reading the
    /// prompt on stdin. Default: $CROSSCUT_HARNESS, then the first of those on PATH.
    #[arg(long)]
    harness: Option<String>,
    /// Model for prompt checks (claude, codex and opencode only).
    #[arg(long)]
    model: Option<String>,
    /// Seconds to allow each cell before stopping it.
    #[arg(long, default_value_t = 1800)]
    timeout: u64,
    /// Directory containing crosscut/ (default: search upwards from here).
    #[arg(long)]
    root: Option<PathBuf>,
}

fn main() -> ExitCode {
    let result = match Cli::parse().command {
        None => {
            print!("{}", orientation());
            Ok(ExitCode::SUCCESS)
        }
        Some(Cmd::Check {
            slugs,
            agentic,
            run,
            dry_run,
        }) => check(slugs, agentic, run, dry_run),
        Some(Cmd::Map { project, root }) => map(project, root),
        Some(Cmd::Test {
            slugs,
            agentic,
            run,
        }) => test(slugs, agentic, run),
        Some(Cmd::Prompt { mode }) => prompt(&mode).map(|text| {
            print!("{text}");
            ExitCode::SUCCESS
        }),
        Some(Cmd::InstallSkill { dirs }) => install_skill(dirs),
    };
    result.unwrap_or_else(|message| {
        eprintln!("crosscut: {message}");
        ExitCode::from(2)
    })
}

// ---------------------------------------------------------------------------------------
// The skill

fn skill_file(path: &str) -> &'static str {
    SKILL_FILES
        .iter()
        .find(|(name, _)| *name == path)
        .map(|(_, text)| *text)
        .expect("embedded")
}

/// SKILL.md without its YAML frontmatter.
fn skill_body() -> &'static str {
    let text = skill_file("SKILL.md");
    text.strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .map_or(text, |(_, body)| body.trim_start())
}

/// The doctrine section of SKILL.md, from its heading up to the next `## ` heading.
fn doctrine() -> &'static str {
    let body = skill_body();
    let rest = &body[body.find("## Doctrine").expect("SKILL.md has a doctrine")..];
    let end = rest[3..].find("\n## ").map_or(rest.len(), |i| i + 4);
    rest[..end].trim_end()
}

fn orientation() -> String {
    format!(
        "CrossCut maps cross-cutting concerns, such as version visibility, staying current, or\n\
         recoverable state, across your projects, and keeps the map current.\n\n\
         {}\n\n\
         ## If you are an agent asked to use CrossCut\n\n\
         Load the mode that fits, and follow it:\n\n  \
         crosscut prompt setup        \"use CrossCut here\"\n  \
         crosscut prompt discover     \"what are we not thinking about?\"\n  \
         crosscut prompt establish    add one concern, with its check and fixtures\n  \
         crosscut prompt refresh      update the map\n  \
         crosscut prompt reconsider   should a concern or its mechanism still exist?\n  \
         crosscut prompt generalize   what does the work just done reveal?\n\n\
         Or run `crosscut install-skill` once, and your harness will load the same\n\
         material as the `crosscut` skill.\n\n\
         Commands: `crosscut check`, `crosscut map`, `crosscut test`. See `crosscut --help`.\n",
        doctrine()
    )
}

fn prompt(mode: &str) -> Result<String, String> {
    if !MODES.contains(&mode) {
        return Err(format!(
            "unknown mode '{mode}'; expected one of: {}",
            MODES.join(", ")
        ));
    }
    let mut refs = vec![format!("modes/{mode}.md")];
    if matches!(mode, "setup" | "discover") {
        refs.push("catalogue.md".into());
        refs.push("reservoirs.md".into());
    }
    if matches!(mode, "setup" | "establish" | "refresh" | "reconsider") {
        refs.push("concern-files.md".into());
    }
    let mut out = String::from(skill_body());
    out.push_str(
        "\n---\n\nThe files this prompt refers to are included below, so ignore their links. \
         Line numbers in this prompt are not line numbers in any file.\n",
    );
    for path in refs {
        out.push_str(&format!(
            "\n---\n\n<!-- {path} -->\n\n{}",
            skill_file(&path)
        ));
    }
    Ok(out)
}

fn install_skill(dirs: Vec<PathBuf>) -> Result<ExitCode, String> {
    let dirs = if dirs.is_empty() {
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .ok_or("cannot find your home directory; pass --dir")?;
        let home = PathBuf::from(home);
        vec![
            home.join(".claude/skills/crosscut"),
            home.join(".agents/skills/crosscut"),
        ]
    } else {
        dirs
    };
    for dir in &dirs {
        for (name, text) in SKILL_FILES {
            let path = dir.join(name);
            path.parent()
                .map_or(Ok(()), std::fs::create_dir_all)
                .and_then(|()| std::fs::write(&path, text))
                .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        }
        println!("installed skill to {}", dir.display());
    }
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------------------
// Running cells

struct Runner {
    harness: Result<Harness, String>,
    model: Option<String>,
    timeout: Duration,
    today: String,
}

impl Runner {
    fn new(run: &RunArgs) -> Runner {
        Runner {
            harness: Harness::resolve(run.harness.clone()),
            model: run.model.clone(),
            timeout: Duration::from_secs(run.timeout),
            today: jiff::Zoned::now().date().to_string(),
        }
    }

    /// What one mechanism observes for one project: `(status, evidence)`, or why it could not
    /// say. `siblings` are all the projects the cell can see, itself included.
    fn cell(
        &self,
        concern: &Concern,
        mechanism: &Mechanism,
        project: &Project,
        siblings: &[Project],
    ) -> Result<(String, String), String> {
        match mechanism {
            Mechanism::Exec(path) => {
                let mut cmd = Command::new(path);
                cmd.current_dir(&project.dir);
                cell_env(&mut cmd, concern, project, siblings);
                let out = harness::run(cmd, None, self.timeout)?;
                let line = out
                    .lines()
                    .map(str::trim)
                    .find(|l| !l.is_empty())
                    .unwrap_or("");
                parse_cell(line)
            }
            Mechanism::Prompt(check) => {
                let harness = self.harness.as_ref().map_err(Clone::clone)?;
                let prompt = self.cell_prompt(concern, check, project, siblings);
                let (mut cmd, stdin) = harness.command(&prompt, self.model.as_deref())?;
                cmd.current_dir(&project.dir);
                cell_env(&mut cmd, concern, project, siblings);
                let out = harness::run(cmd, stdin.then_some(prompt.as_str()), self.timeout)?;
                let start = out
                    .rfind(CELL_OPEN)
                    .ok_or("the agent's reply had no <crosscut-cell> answer")?;
                let rest = &out[start + CELL_OPEN.len()..];
                parse_cell(rest[..rest.find(CELL_CLOSE).unwrap_or(rest.len())].trim())
            }
        }
    }

    fn cell_prompt(
        &self,
        concern: &Concern,
        check: &str,
        project: &Project,
        siblings: &[Project],
    ) -> String {
        let others: Vec<String> = siblings
            .iter()
            .filter(|p| p.name != project.name)
            .map(|p| format!("`{}` at `{}`", p.name, p.dir.display()))
            .collect();
        format!(
            "{}\n\n---\n\n# Your task\n\n\
             You are a CrossCut prompt check: answer one concern for one project. No human is \
             available.\n\n\
             - Concern: {} (its definition and your instructions are below).\n\
             - Project: `{}`, at `{}`, which is your working directory.\n\
             - Its sibling projects, if you need to compare: {}.\n\
             - Today: {}.\n\n\
             Do not modify any files. Read, and run only commands that do not change state. \
             This is not a bug hunt.\n\n\
             Answer with exactly one line between `{CELL_OPEN}` and `{CELL_CLOSE}`: a status \
             (yes, partly, missing, n/a or unknown), a colon, and one sentence of evidence. \
             Use `unknown` with the reason when you cannot tell.\n\n\
             <concern>\n{}\n</concern>\n\n<check>\n{}\n</check>\n",
            doctrine(),
            concern.name(),
            project.name,
            project.dir.display(),
            if others.is_empty() {
                "none".into()
            } else {
                others.join(", ")
            },
            self.today,
            concern.definition.trim(),
            check.trim()
        )
    }
}

fn cell_env(cmd: &mut Command, concern: &Concern, project: &Project, siblings: &[Project]) {
    let list: Vec<String> = siblings
        .iter()
        .map(|p| format!("{}\t{}", p.name, p.dir.display()))
        .collect();
    cmd.env("CROSSCUT_PROJECT", &project.name)
        .env("CROSSCUT_PROJECT_DIR", &project.dir)
        .env("CROSSCUT_PROJECTS", list.join("\n"))
        .env("CROSSCUT_CONCERN_DIR", &concern.dir);
}

/// `status: evidence` → the pair, if the status is one a mechanism may observe.
fn parse_cell(line: &str) -> Result<(String, String), String> {
    let (status, evidence) = model::split_status(line);
    if model::OBSERVED.contains(&status.as_str()) {
        Ok((status, evidence))
    } else if status == "deferred" {
        Err("printed `deferred`, which only a person's decision can record".into())
    } else {
        Err(format!(
            "printed `{}`, which does not start with {}",
            line.chars().take(80).collect::<String>(),
            model::OBSERVED.join(", ")
        ))
    }
}

/// Runs `work` on each item with up to `jobs` workers, each taking the next unstarted item
/// as soon as it is free. Results come back in item order.
fn pool<T: Sync, R: Send>(jobs: usize, items: &[T], work: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<Option<R>>> = Mutex::new((0..items.len()).map(|_| None).collect());
    std::thread::scope(|scope| {
        for _ in 0..jobs.clamp(1, items.len().max(1)) {
            scope.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some(item) = items.get(i) else { break };
                let result = work(item);
                results.lock().expect("no worker panics holding the lock")[i] = Some(result);
            });
        }
    });
    results
        .into_inner()
        .expect("workers finished")
        .into_iter()
        .map(|r| r.expect("every item ran"))
        .collect()
}

fn select_concerns(root: &Path, slugs: &[String]) -> Result<Vec<Concern>, String> {
    let mut concerns = model::concerns(root)?;
    for slug in slugs {
        if !concerns.iter().any(|c| &c.slug == slug) {
            return Err(format!(
                "no concern `{slug}` in {}",
                root.join("crosscut/concerns").display()
            ));
        }
    }
    if !slugs.is_empty() {
        concerns.retain(|c| slugs.contains(&c.slug));
    }
    Ok(concerns)
}

fn select_projects(all: &[Project], names: &[String]) -> Result<Vec<Project>, String> {
    for name in names {
        if !all.iter().any(|p| &p.name == name) {
            let known: Vec<&str> = all.iter().map(|p| p.name.as_str()).collect();
            return Err(format!(
                "no project `{name}` in crosscut/projects (known: {})",
                known.join(", ")
            ));
        }
    }
    Ok(all
        .iter()
        .filter(|p| names.is_empty() || names.contains(&p.name))
        .cloned()
        .collect())
}

fn check(
    slugs: Vec<String>,
    agentic: bool,
    run: RunArgs,
    dry_run: bool,
) -> Result<ExitCode, String> {
    let root = model::find_root(run.root.clone())?;
    let concerns = select_concerns(&root, &slugs)?;
    let all = model::projects(&root)?;
    let projects = select_projects(&all, &run.projects)?;
    let runner = Runner::new(&run);
    if let Ok(harness) = &runner.harness {
        harness.command("", runner.model.as_deref())?;
    }

    let mut cells = Vec::new();
    for concern in &concerns {
        match concern.mechanism() {
            Some(Mechanism::Prompt(_)) if !agentic => {
                eprintln!(
                    "crosscut: {}: prompt check not run (add --agentic to spend model calls on it)",
                    concern.slug
                )
            }
            Some(_) => cells.extend(projects.iter().map(|project| (concern, project))),
            None => eprintln!(
                "crosscut: {}: no check or check.md yet, so nothing to run",
                concern.slug
            ),
        }
    }

    if dry_run {
        for (concern, project) in &cells {
            match concern.mechanism().expect("selected above") {
                Mechanism::Exec(path) => {
                    println!("{}/{}: run {}", concern.slug, project.name, path.display())
                }
                Mechanism::Prompt(check) => {
                    let prompt = runner.cell_prompt(concern, &check, project, &all);
                    let how = match &runner.harness {
                        Ok(h) => h
                            .command(&prompt, runner.model.as_deref())
                            .map(|(cmd, stdin)| harness::describe(&cmd, stdin, &prompt))
                            .unwrap_or_else(|e| e),
                        Err(e) => e.clone(),
                    };
                    println!("{}/{}: ask {how}\n{prompt}", concern.slug, project.name);
                }
            }
        }
        return Ok(ExitCode::SUCCESS);
    }

    let results = pool(run.jobs, &cells, |(concern, project)| {
        let mechanism = concern.mechanism().expect("selected above");
        let result = runner.cell(concern, &mechanism, project, &all);
        (matches!(mechanism, Mechanism::Prompt(_)), result)
    });

    let (mut changed, mut failed) = (0, 0);
    for concern in &concerns {
        let old = concern.observations();
        let mut rows: Vec<Observation> = old
            .iter()
            .filter(|o| all.iter().any(|p| p.name == o.project))
            .cloned()
            .collect();
        for ((cell_concern, project), (prompted, result)) in cells.iter().zip(&results) {
            if cell_concern.slug != concern.slug {
                continue;
            }
            let previous = old.iter().find(|o| o.project == project.name);
            match result {
                Err(message) => {
                    failed += 1;
                    eprintln!(
                        "crosscut: {}/{}: check failed, previous row kept: {message}",
                        concern.slug, project.name
                    );
                }
                Ok((status, evidence)) => {
                    let row = match previous {
                        // An unchanged status keeps its date, and for prompt checks its wording
                        // too, so a rerun on an unchanged world changes nothing.
                        Some(p) if &p.status == status && *prompted => p.clone(),
                        Some(p) if &p.status == status => Observation {
                            evidence: evidence.clone(),
                            ..p.clone()
                        },
                        _ => Observation {
                            project: project.name.clone(),
                            status: status.clone(),
                            since: runner.today.clone(),
                            evidence: evidence.clone(),
                        },
                    };
                    if previous.map(|p| &p.status) != Some(status) {
                        changed += 1;
                        let from = previous.map_or("(none)", |p| p.status.as_str());
                        println!(
                            "{}/{}: {from} → {status}: {evidence}",
                            concern.slug, project.name
                        );
                    }
                    rows.retain(|r| r.project != project.name);
                    rows.push(row);
                }
            }
        }
        let mut sorted_old = old.clone();
        sorted_old.sort_by(|a, b| a.project.cmp(&b.project));
        rows.sort_by(|a, b| a.project.cmp(&b.project));
        if rows != sorted_old || (!rows.is_empty() && !concern.observed_path().exists()) {
            model::write_observations(&concern.observed_path(), &mut rows)?;
        }
    }
    println!(
        "{} cells checked: {changed} changed, {failed} could not run",
        cells.len()
    );
    Ok(if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

// ---------------------------------------------------------------------------------------
// The map

/// What the map shows for one cell: the decision if there is one, else the observation.
/// The flag marks a decision that the latest observation contradicts.
fn cell_view(concern: &Concern, project: &str) -> Option<(String, bool)> {
    let observed = concern
        .observations()
        .into_iter()
        .find(|o| o.project == project);
    let decided = concern
        .decisions()
        .into_iter()
        .find(|d| d.project == project);
    match (decided, observed) {
        (Some(d), Some(o)) => {
            let found_after_all = matches!(d.status.as_str(), "n/a" | "deferred" | "missing")
                && matches!(o.status.as_str(), "yes" | "partly");
            let decided_met_but_not =
                matches!(d.status.as_str(), "yes" | "partly") && o.status == "missing";
            Some((d.status, found_after_all || decided_met_but_not))
        }
        (Some(d), None) => Some((d.status, false)),
        (None, Some(o)) => Some((o.status, false)),
        (None, None) => None,
    }
}

fn map(project: Option<String>, root: Option<PathBuf>) -> Result<ExitCode, String> {
    let root = model::find_root(root)?;
    let concerns = model::concerns(&root)?;
    let projects = model::projects(&root)?;
    if let Some(name) = project {
        select_projects(&projects, std::slice::from_ref(&name))?;
        return map_column(&concerns, &name);
    }
    let first = concerns
        .iter()
        .map(|c| c.slug.len())
        .max()
        .unwrap_or(0)
        .max(7);
    let widths: Vec<usize> = projects.iter().map(|p| p.name.len().max(8)).collect();
    let mut header = format!("{:first$}", "");
    for (p, w) in projects.iter().zip(&widths) {
        header.push_str(&format!("  {:w$}", p.name));
    }
    println!("{}", header.trim_end());
    let mut notes = Vec::new();
    for concern in &concerns {
        let mut line = format!("{:first$}", concern.slug);
        let cells: Vec<Option<(String, bool)>> = projects
            .iter()
            .map(|p| cell_view(concern, &p.name))
            .collect();
        if cells.iter().all(Option::is_none) {
            line.push_str(if concern.mechanism().is_some() {
                "  (not checked yet)"
            } else {
                "  (no check yet)"
            });
        }
        for ((p, w), cell) in projects.iter().zip(&widths).zip(&cells) {
            let text = match cell {
                Some((status, true)) => {
                    notes.push(format!("{}/{}", concern.slug, p.name));
                    format!("{status}*")
                }
                Some((status, false)) => status.clone(),
                None if cells.iter().all(Option::is_none) => continue,
                None => "·".into(),
            };
            line.push_str(&format!("  {text:w$}"));
        }
        println!("{}", line.trim_end());
    }
    if !notes.is_empty() {
        println!(
            "\n* a decision the latest observation contradicts: {}",
            notes.join(", ")
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn map_column(concerns: &[Concern], project: &str) -> Result<ExitCode, String> {
    let width = concerns.iter().map(|c| c.slug.len()).max().unwrap_or(0);
    for concern in concerns {
        let observed = concern
            .observations()
            .into_iter()
            .find(|o| o.project == project);
        let decided = concern
            .decisions()
            .into_iter()
            .find(|d| d.project == project);
        let status = cell_view(concern, project).map_or("·".to_string(), |(s, flag)| {
            if flag {
                format!("{s}*")
            } else {
                s
            }
        });
        let mut detail = match &observed {
            Some(o) => format!("{} (since {})", o.evidence, o.since),
            None => "not checked".into(),
        };
        if let Some(d) = decided {
            detail = format!("decided: {}; observed: {detail}", d.reason);
        }
        println!("{:width$}  {status:9} {detail}", concern.slug);
    }
    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------------------
// Testing mechanisms

fn test(slugs: Vec<String>, agentic: bool, run: RunArgs) -> Result<ExitCode, String> {
    let root = model::find_root(run.root.clone())?;
    let concerns = select_concerns(&root, &slugs)?;
    let runner = Runner::new(&run);
    let mut jobs = Vec::new();
    for concern in &concerns {
        let Some(mechanism) = concern.mechanism() else {
            println!("{}: no check yet", concern.slug);
            continue;
        };
        let prompted = matches!(mechanism, Mechanism::Prompt(_));
        let cases = fixture_cases(&concern.dir.join("fixtures"));
        if cases.is_empty() {
            println!("{}: no fixtures", concern.slug);
        } else if prompted && !agentic {
            println!(
                "{}: prompt check, {} fixture cases not run (use --agentic)",
                concern.slug,
                cases.len()
            );
        } else {
            for (case, projects, expected) in cases {
                for (project, status) in expected {
                    jobs.push((concern, case.clone(), projects.clone(), project, status));
                }
            }
        }
    }
    let results = pool(run.jobs, &jobs, |(concern, _, projects, project, _)| {
        let mechanism = concern.mechanism().expect("selected above");
        match projects.iter().find(|p| &p.name == project) {
            Some(p) => runner.cell(concern, &mechanism, p, projects),
            None => Err(format!(
                "`expect` names `{project}`, which is not a directory in this case"
            )),
        }
    });
    let mut wrong = 0;
    for ((concern, case, _, project, expected), result) in jobs.iter().zip(&results) {
        match result {
            Ok((status, _)) if status == expected => {}
            Ok((status, evidence)) => {
                wrong += 1;
                println!(
                    "{}/{case}/{project}: expected {expected}, observed {status}: {evidence}",
                    concern.slug
                );
            }
            Err(message) => {
                wrong += 1;
                println!(
                    "{}/{case}/{project}: expected {expected}, but the check failed: {message}",
                    concern.slug
                );
            }
        }
    }
    println!(
        "{} fixture cells: {} as expected, {wrong} not",
        jobs.len(),
        jobs.len() - wrong
    );
    Ok(if wrong == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

type Case = (String, Vec<Project>, Vec<(String, String)>);

/// Each fixture case: its name, its project directories, and its `expect` lines.
fn fixture_cases(dir: &Path) -> Vec<Case> {
    let mut cases: Vec<Case> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|case| case.join("expect").is_file())
        .map(|case| {
            let mut projects: Vec<Project> = std::fs::read_dir(&case)
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .map(|dir| Project {
                    name: dir
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                    dir,
                })
                .collect();
            projects.sort_by(|a, b| a.name.cmp(&b.name));
            let expected = std::fs::read_to_string(case.join("expect"))
                .unwrap_or_default()
                .lines()
                .filter_map(|l| {
                    let (project, status) = l.trim().split_once(char::is_whitespace)?;
                    Some((project.to_string(), status.trim().to_lowercase()))
                })
                .collect();
            (
                case.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                projects,
                expected,
            )
        })
        .collect();
    cases.sort_by(|a, b| a.0.cmp(&b.0));
    cases
}
