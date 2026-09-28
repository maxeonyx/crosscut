//! `crosscut`: delivers the CrossCut skill, lists concern files, and refreshes
//! their current views headless through an installed coding harness.
//!
//! The substance of CrossCut is the skill in `skill/`. This binary only carries
//! it and does the few mechanical jobs that are fiddly to do by hand.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};

/// Every file of the skill, by its path relative to the skill directory.
const SKILL_FILES: &[(&str, &str)] = &[
    ("SKILL.md", include_str!("../skill/SKILL.md")),
    (
        "concern-files.md",
        include_str!("../skill/concern-files.md"),
    ),
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

const VIEW_HEADING: &str = "## Current view";
const VIEW_OPEN: &str = "<crosscut-view>";
const VIEW_CLOSE: &str = "</crosscut-view>";

#[derive(Parser)]
#[command(
    name = "crosscut",
    version,
    about = "Widen what coding agents notice about software, and keep it visible.",
    long_about = "Widen what coding agents notice about software, and keep it visible.\n\n\
        Run `crosscut` with no arguments to read what CrossCut is and how an agent should use it.",
    after_help = "Examples:\n  \
        $ crosscut install-skill\n  \
        $ crosscut prompt setup | claude -p\n  \
        $ crosscut list\n  \
        $ crosscut refresh recovery --harness codex"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Print the full prompt for one mode of work, with the doctrine and references inlined.
    ///
    /// For harnesses without skill support, or to paste into any agent.
    #[command(
        after_help = "Modes: setup, discover, establish, refresh, reconsider, generalize\n\n\
        Examples:\n  $ crosscut prompt discover | claude -p\n  $ crosscut prompt reconsider > /tmp/prompt.md"
    )]
    Prompt { mode: String },

    /// Install the skill where Claude Code, Codex and OpenCode look for it.
    ///
    /// Defaults to ~/.claude/skills/crosscut and ~/.agents/skills/crosscut. Overwrites
    /// only the files CrossCut ships.
    #[command(after_help = "Examples:\n  $ crosscut install-skill\n  \
        $ crosscut install-skill --dir .claude/skills/crosscut")]
    InstallSkill {
        /// Install into this directory instead (repeatable).
        #[arg(long = "dir")]
        dirs: Vec<PathBuf>,
    },

    /// List concerns and the date of each current view.
    #[command(
        after_help = "Finds the nearest crosscut/concerns/ at or above the current directory.\n\n\
        Examples:\n  $ crosscut list\n  $ crosscut list --root ~/home-ecosystem"
    )]
    List {
        /// Directory containing crosscut/ (default: search upwards from here).
        #[arg(long)]
        root: Option<PathBuf>,
    },

    /// Refresh current views headless, several concerns at once, through a coding harness.
    ///
    /// The agent reads the concern and the projects, runs what "How to look" says, and
    /// returns a new "Current view"; crosscut writes only that section back. Findings,
    /// good or bad, never affect the exit status: 0 means every refresh ran, 1 means
    /// some could not run (the rest still did), 2 means nothing could run.
    #[command(
        after_help = "Harness: --harness claude|codex|opencode, or any shell command that reads the\n\
        prompt on stdin and prints the reply. Defaults to $CROSSCUT_HARNESS, then the first of\n\
        claude, codex, opencode on PATH.\n\n\
        The agent gets a shell in the working directory so it can run \"How to look\" commands.\n\
        It is asked not to change anything, but only codex enforces that (read-only sandbox,\n\
        usually without network). Treat concern files like scripts: refresh only ones you trust.\n\n\
        Examples:\n  $ crosscut refresh\n  $ crosscut refresh recovery agent-guidance --harness codex\n  \
        $ crosscut refresh --dry-run"
    )]
    Refresh {
        /// Concern slugs to refresh (default: all).
        slugs: Vec<String>,
        #[arg(long)]
        harness: Option<String>,
        /// Model for claude, codex or opencode (for a custom command, put it in the command).
        #[arg(long)]
        model: Option<String>,
        /// How many concerns to refresh at once.
        #[arg(long, default_value_t = 6)]
        jobs: usize,
        /// Minutes to allow each concern before stopping its harness.
        #[arg(long, default_value_t = 30)]
        timeout: u64,
        /// Directory containing crosscut/ (default: search upwards from here).
        #[arg(long)]
        root: Option<PathBuf>,
        /// Print the harness command and prompt instead of running anything.
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        None => {
            print!("{}", orientation());
            Ok(ExitCode::SUCCESS)
        }
        Some(Cmd::Prompt { mode }) => prompt(&mode).map(|text| {
            print!("{text}");
            ExitCode::SUCCESS
        }),
        Some(Cmd::InstallSkill { dirs }) => install_skill(dirs),
        Some(Cmd::List { root }) => list(root),
        Some(Cmd::Refresh {
            slugs,
            harness,
            model,
            jobs,
            timeout,
            root,
            dry_run,
        }) => refresh(slugs, harness, model, jobs, timeout, root, dry_run),
    };
    result.unwrap_or_else(|message| {
        eprintln!("crosscut: {message}");
        ExitCode::from(2)
    })
}

fn skill_file(path: &str) -> &'static str {
    SKILL_FILES
        .iter()
        .find(|(name, _)| *name == path)
        .map(|(_, text)| *text)
        .expect("skill file is embedded")
}

/// SKILL.md without its YAML frontmatter.
fn skill_body() -> &'static str {
    let text = skill_file("SKILL.md");
    text.strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .map_or(text, |(_, body)| body.trim_start())
}

/// The doctrine section of SKILL.md, from its heading up to the next heading.
fn doctrine() -> &'static str {
    let body = skill_body();
    let start = body.find("## Doctrine").expect("SKILL.md has a doctrine");
    let rest = &body[start..];
    let end = rest[3..].find("\n## ").map_or(rest.len(), |i| i + 4);
    rest[..end].trim_end()
}

fn orientation() -> String {
    format!(
        "CrossCut widens what a coding agent notices about software, and keeps the\n\
         worthwhile parts visible as plain Markdown concern files in crosscut/concerns/.\n\n\
         {}\n\n\
         ## If you are an agent asked to use CrossCut\n\n\
         Load the mode that fits, and follow it:\n\n  \
         crosscut prompt setup        \"use CrossCut here\"\n  \
         crosscut prompt discover     \"what are we not thinking about?\"\n  \
         crosscut prompt establish    make one concern persistent\n  \
         crosscut prompt refresh      update current views\n  \
         crosscut prompt reconsider   should a concern or its mechanism still exist?\n  \
         crosscut prompt generalize   what does the work just done reveal?\n\n\
         Or run `crosscut install-skill` once, and your harness will load the same\n\
         material as the `crosscut` skill.\n\n\
         Other commands: `crosscut list`, `crosscut refresh`. See `crosscut --help`.\n",
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
            let written = path
                .parent()
                .map_or(Ok(()), std::fs::create_dir_all)
                .and_then(|()| std::fs::write(&path, text));
            written.map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        }
        println!("installed skill to {}", dir.display());
    }
    Ok(ExitCode::SUCCESS)
}

/// The directory that contains `crosscut/`, given explicitly or found upwards.
fn find_root(root: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(root) = root {
        return if root.join("crosscut/concerns").is_dir() {
            Ok(root)
        } else {
            Err(format!("no crosscut/concerns/ in {}", root.display()))
        };
    }
    let here =
        std::env::current_dir().map_err(|e| format!("cannot read current directory: {e}"))?;
    here.ancestors()
        .find(|dir| dir.join("crosscut/concerns").is_dir())
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            "no crosscut/concerns/ here or above; to set one up, ask your agent to \
             use CrossCut (see `crosscut prompt setup`)"
                .into()
        })
}

struct Concern {
    slug: String,
    path: PathBuf,
    text: String,
}

impl Concern {
    fn question(&self) -> &str {
        self.text
            .lines()
            .find_map(|line| line.strip_prefix("# "))
            .unwrap_or("(no question heading)")
            .trim()
    }

    /// Byte offset of the current view heading, if the concern has one.
    fn view_start(&self) -> Option<usize> {
        let mut offset = 0;
        let mut found = None;
        let mut fenced = false;
        for line in self.text.split_inclusive('\n') {
            if line.trim_start().starts_with("```") {
                fenced = !fenced;
            } else if !fenced && line.starts_with(VIEW_HEADING) {
                found = Some(offset);
            }
            offset += line.len();
        }
        found
    }

    /// The date on the view heading and the first sentence of the view.
    fn view_summary(&self) -> Option<(String, String)> {
        let view = &self.text[self.view_start()?..];
        let mut lines = view.lines();
        let heading = lines.next()?;
        let date = heading
            .trim_start_matches(VIEW_HEADING)
            .trim_start_matches([' ', '—', '-', ':'])
            .trim()
            .to_string();
        let first = lines
            .map(str::trim)
            .find(|line| !line.is_empty())
            .unwrap_or("");
        let headline = first
            .find(". ")
            .map_or(first, |end| &first[..=end])
            .to_string();
        Some((date, headline))
    }

    fn with_view(&self, view: &str) -> String {
        let definition = &self.text[..self.view_start().unwrap_or(self.text.len())];
        format!("{}\n\n{}\n", definition.trim_end(), view.trim())
    }
}

fn concerns(root: &Path) -> Result<Vec<Concern>, String> {
    let dir = root.join("crosscut/concerns");
    let entries =
        std::fs::read_dir(&dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "md") {
            let text = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
            let slug = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            found.push(Concern { slug, path, text });
        }
    }
    found.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(found)
}

fn list(root: Option<PathBuf>) -> Result<ExitCode, String> {
    let root = find_root(root)?;
    let concerns = concerns(&root)?;
    if concerns.is_empty() {
        println!(
            "no concerns yet in {}",
            root.join("crosscut/concerns").display()
        );
    }
    let width = concerns.iter().map(|c| c.slug.len()).max().unwrap_or(0);
    for concern in &concerns {
        println!("{:width$}  {}", concern.slug, concern.question());
        match concern.view_summary() {
            Some((date, headline)) => println!("{:width$}  {date}: {headline}", ""),
            None => println!("{:width$}  no view yet", ""),
        }
    }
    Ok(ExitCode::SUCCESS)
}

enum Harness {
    Claude,
    Codex,
    OpenCode,
    Shell(String),
}

impl Harness {
    fn resolve(choice: Option<String>) -> Result<Harness, String> {
        let choice = choice.or_else(|| std::env::var("CROSSCUT_HARNESS").ok());
        match choice.as_deref() {
            Some("claude") => Ok(Harness::Claude),
            Some("codex") => Ok(Harness::Codex),
            Some("opencode") => Ok(Harness::OpenCode),
            Some(command) => Ok(Harness::Shell(command.to_string())),
            None => [
                ("claude", Harness::Claude),
                ("codex", Harness::Codex),
                ("opencode", Harness::OpenCode),
            ]
            .into_iter()
            .find(|(name, _)| on_path(name))
            .map(|(_, harness)| harness)
            .ok_or_else(|| {
                "no coding harness found: install and log in to claude, codex or opencode, \
                     or pass --harness '<command that reads a prompt on stdin>'"
                    .into()
            }),
        }
    }

    /// The command to run, and whether the prompt goes on stdin (otherwise it is the last argument).
    fn command(&self, prompt: &str, model: Option<&str>) -> Result<(Command, bool), String> {
        let (mut cmd, stdin) = match self {
            // Bash is needed to run "How to look" commands. The edit tools are denied and the
            // target's project settings (hooks, MCP servers, allow rules) are not loaded,
            // because the target may not be trusted. MCP servers and subagents are off too,
            // so one refresh cannot reach external services or fan out. The view comes back as text.
            Harness::Claude => {
                let mut cmd = Command::new("claude");
                cmd.args([
                    "-p",
                    "--no-session-persistence",
                    "--setting-sources",
                    "user",
                ])
                .args(["--permission-mode", "dontAsk"])
                .args(["--allowedTools", "Read,Grep,Glob,Bash,WebFetch,WebSearch"])
                .args([
                    "--disallowedTools",
                    "Edit,Write,NotebookEdit,Agent,Workflow",
                ])
                .arg("--strict-mcp-config");
                (cmd, true)
            }
            Harness::Codex => {
                let mut cmd = Command::new("codex");
                cmd.args([
                    "exec",
                    "--sandbox",
                    "read-only",
                    "--skip-git-repo-check",
                    "--ephemeral",
                ]);
                (cmd, true)
            }
            Harness::OpenCode => (Command::new("opencode"), false),
            Harness::Shell(command) => {
                if model.is_some() {
                    return Err("--model only applies to claude, codex and opencode; put the model in your command".into());
                }
                let mut cmd = if cfg!(windows) {
                    let mut cmd = Command::new("cmd");
                    cmd.args(["/C", command]);
                    cmd
                } else {
                    let mut cmd = Command::new("sh");
                    cmd.args(["-c", command]);
                    cmd
                };
                cmd.env_remove("CROSSCUT_HARNESS");
                (cmd, true)
            }
        };
        if let Some(model) = model {
            cmd.args([
                if matches!(self, Harness::Claude) {
                    "--model"
                } else {
                    "-m"
                },
                model,
            ]);
        }
        match self {
            Harness::Codex => {
                cmd.arg("-");
            }
            Harness::OpenCode => {
                cmd.args(["run", prompt]);
            }
            _ => {}
        }
        Ok((cmd, stdin))
    }
}

/// How a command will be run, for --dry-run, with a prompt argument elided.
fn describe(cmd: &Command, stdin: bool, prompt: &str) -> String {
    let mut words = vec![cmd.get_program().to_string_lossy().into_owned()];
    for arg in cmd.get_args() {
        let arg = arg.to_string_lossy();
        words.push(if arg == prompt {
            "<prompt>".into()
        } else {
            arg.into_owned()
        });
    }
    if stdin {
        words.push(" (prompt on stdin)".into());
    }
    words.join(" ")
}

fn on_path(program: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths)
            .any(|dir| dir.join(program).is_file() || dir.join(format!("{program}.exe")).is_file())
    })
}

fn refresh_prompt(root: &Path, concern: &Concern, today: &str) -> String {
    let projects = std::fs::read_to_string(root.join("crosscut/projects.md"))
        .map(|text| format!("\n`crosscut/projects.md`:\n\n{text}\n"))
        .unwrap_or_default();
    format!(
        "{}\n---\n\n# Your task\n\n\
         Refresh one CrossCut concern, headless. No human is available, so anything you \
         cannot find out becomes a stated unknown in the view.\n\n\
         - Working directory: `{}` (the directory containing `crosscut/`).\n\
         - Concern: `crosscut/concerns/{}.md` (its full text is below).\n\
         - Today: {today}.\n{projects}\n\
         Do not modify any files. Read, and run only commands that do not change state.\n\n\
         When you are done, output the complete new section, starting with \
         `## Current view — {today}`, between `{VIEW_OPEN}` and `{VIEW_CLOSE}` on their own \
         lines. crosscut will replace the concern's current view with exactly that text, and \
         will keep everything above it unchanged.\n\n\
         <concern-file>\n{}\n</concern-file>\n",
        prompt("refresh").expect("refresh is a mode"),
        root.display(),
        concern.slug,
        concern.text
    )
}

/// The last tagged view in the harness output, with a heading guaranteed.
fn extract_view(output: &str, today: &str) -> Option<String> {
    let start = output.rfind(VIEW_OPEN)? + VIEW_OPEN.len();
    let end = start + output[start..].find(VIEW_CLOSE)?;
    let view = output[start..end].trim();
    if view.is_empty() {
        return None;
    }
    Some(if view.starts_with(VIEW_HEADING) {
        view.to_string()
    } else {
        format!("{VIEW_HEADING} — {today}\n\n{view}")
    })
}

/// Runs the harness and returns the view from its reply. Harnesses do not reliably
/// signal failure through their exit status (a misconfigured `opencode run` exits 0 with
/// no output), so a missing view is reported with the harness's own last words.
fn ask_for_view(
    harness: &Harness,
    model: Option<&str>,
    timeout: Duration,
    prompt: &str,
    root: &Path,
    today: &str,
) -> Result<String, String> {
    let (mut cmd, stdin) = harness.command(prompt, model)?;
    cmd.current_dir(root)
        .stdin(if stdin { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("cannot start harness: {e}"))?;
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    if stdin {
        let mut pipe = child.stdin.take().expect("stdin is piped");
        pipe.write_all(prompt.as_bytes())
            .map_err(|e| format!("cannot send prompt to harness: {e}"))?;
    }
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(200)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "harness still running after {} minutes and was stopped; raise --timeout if that is expected",
                    timeout.as_secs() / 60
                ));
            }
            Err(e) => return Err(format!("harness did not finish: {e}")),
        }
    };
    let stdout = stdout.join().unwrap_or_default();
    let stderr = strip_ansi(&stderr.join().unwrap_or_default());
    let tail: Vec<&str> = stderr.lines().filter(|l| !l.trim().is_empty()).collect();
    let tail = tail[tail.len().saturating_sub(5)..].join(" | ");
    if !status.success() {
        return Err(format!("harness exited with {status}: {tail}"));
    }
    extract_view(&stdout, today).ok_or_else(|| {
        format!("harness reply had no {VIEW_OPEN} section, so the view was left unchanged; harness stderr: {tail}")
    })
}

/// Reads a pipe to the end on its own thread, so a chatty harness cannot fill one pipe
/// and block while we wait on the other.
fn drain(pipe: Option<impl Read + Send + 'static>) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut bytes);
        }
        String::from_utf8_lossy(&bytes).into_owned()
    })
}

fn refresh(
    slugs: Vec<String>,
    harness: Option<String>,
    model: Option<String>,
    jobs: usize,
    timeout: u64,
    root: Option<PathBuf>,
    dry_run: bool,
) -> Result<ExitCode, String> {
    let model = model.as_deref();
    let timeout = Duration::from_secs(timeout * 60);
    let root = find_root(root)?;
    let root = root.canonicalize().unwrap_or(root);
    let mut concerns = concerns(&root)?;
    if !slugs.is_empty() {
        for slug in &slugs {
            if !concerns.iter().any(|c| &c.slug == slug) {
                return Err(format!(
                    "no concern '{slug}' in {}",
                    root.join("crosscut/concerns").display()
                ));
            }
        }
        concerns.retain(|c| slugs.contains(&c.slug));
    }
    if concerns.is_empty() {
        return Err("no concerns to refresh".into());
    }
    let harness = Harness::resolve(harness)?;
    harness.command("", model)?;
    let today = jiff::Zoned::now().date().to_string();

    if dry_run {
        let (cmd, stdin) = harness.command("<prompt>", model)?;
        println!(
            "harness: {}\nworking directory: {}\ntimeout: {} minutes per concern\n",
            describe(&cmd, stdin, "<prompt>"),
            root.display(),
            timeout.as_secs() / 60
        );
        for concern in &concerns {
            println!(
                "===== prompt for {} =====\n{}",
                concern.slug,
                refresh_prompt(&root, concern, &today)
            );
        }
        return Ok(ExitCode::SUCCESS);
    }

    // A pool of `jobs` workers, each taking the next unstarted concern as soon as it is
    // free. Concerns are separate files, so the workers never write to the same one.
    let next = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..jobs.clamp(1, concerns.len()) {
            scope.spawn(|| {
                while let Some(concern) = concerns.get(next.fetch_add(1, Ordering::Relaxed)) {
                    eprintln!("crosscut: refreshing {} ...", concern.slug);
                    let prompt = refresh_prompt(&root, concern, &today);
                    let outcome = ask_for_view(&harness, model, timeout, &prompt, &root, &today)
                        .and_then(|view| {
                            std::fs::write(&concern.path, concern.with_view(&view)).map_err(|e| {
                                format!("cannot write {}: {e}", concern.path.display())
                            })
                        });
                    match outcome {
                        Ok(()) => println!("{}: view updated", concern.slug),
                        Err(message) => {
                            failed.fetch_add(1, Ordering::Relaxed);
                            eprintln!("crosscut: {}: {message}", concern.slug);
                        }
                    }
                }
            });
        }
    });
    let failed = failed.into_inner();
    Ok(match failed {
        0 => ExitCode::SUCCESS,
        n if n == concerns.len() => ExitCode::from(2),
        _ => ExitCode::from(1),
    })
}

/// Removes terminal colour sequences, which harnesses print even into a pipe.
fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.next() == Some('[') {
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}
