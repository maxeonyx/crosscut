//! Running things: executable checks, and prompts through an installed coding harness.

use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub enum Harness {
    Claude,
    Codex,
    OpenCode,
    Shell(String),
}

impl Harness {
    pub fn resolve(choice: Option<String>) -> Result<Harness, String> {
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
                "no coding harness found for prompt checks: install and log in to claude, codex \
                     or opencode, or pass --harness '<command that reads a prompt on stdin>'"
                    .into()
            }),
        }
    }

    /// The command to run, and whether the prompt goes on stdin (otherwise it is the last argument).
    pub fn command(&self, prompt: &str, model: Option<&str>) -> Result<(Command, bool), String> {
        let (mut cmd, stdin) = match self {
            // A prompt check only reads and runs commands; it answers in text. Bash is allowed so
            // it can look; edit tools, subagents, MCP servers and the target's project settings
            // (hooks, allow rules) are not, because the target may not be trusted. Only Codex's
            // read-only sandbox actually prevents writes.
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
                    return Err(
                        "--model only applies to claude, codex and opencode; put the model in your command".into(),
                    );
                }
                let mut cmd = shell(command);
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

fn shell(command: &str) -> Command {
    if cfg!(windows) {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", command]);
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", command]);
        cmd
    }
}

/// How a command will be run, for --dry-run, with a prompt argument elided.
pub fn describe(cmd: &Command, stdin: bool, prompt: &str) -> String {
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

/// Runs `cmd` to completion or until `timeout`, feeding `input` on stdin when given. Returns
/// stdout, or an explanation that includes the last lines of stderr.
pub fn run(mut cmd: Command, input: Option<&str>, timeout: Duration) -> Result<String, String> {
    cmd.stdin(if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    })
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("cannot start {:?}: {e}", cmd.get_program()))?;
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    if let Some(input) = input {
        let mut pipe = child.stdin.take().expect("stdin is piped");
        // A check may exit without reading its input; that is not an error.
        let _ = pipe.write_all(input.as_bytes());
    }
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(50)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "still running after {}s and was stopped",
                    timeout.as_secs()
                ));
            }
            Err(e) => return Err(format!("did not finish: {e}")),
        }
    };
    let stdout = stdout.join().unwrap_or_default();
    let stderr = strip_ansi(&stderr.join().unwrap_or_default());
    if !status.success() {
        let lines: Vec<&str> = stderr.lines().filter(|l| !l.trim().is_empty()).collect();
        let tail = lines[lines.len().saturating_sub(3)..].join(" | ");
        return Err(format!("exited with {status}: {tail}"));
    }
    Ok(stdout)
}

/// Reads a pipe to the end on its own thread, so a chatty child cannot fill one pipe and
/// block while we wait on the other.
fn drain(pipe: Option<impl Read + Send + 'static>) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut bytes);
        }
        String::from_utf8_lossy(&bytes).into_owned()
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
