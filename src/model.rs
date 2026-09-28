//! The files CrossCut reads and writes: the project inventory, concern definitions with
//! their decisions, and the observations checks produce. See docs/design.md section 6.

use std::path::{Path, PathBuf};

/// Statuses a mechanism may observe. `deferred` is deliberately absent: it only exists as a
/// human decision.
pub const OBSERVED: &[&str] = &["yes", "partly", "missing", "n/a", "unknown"];

/// Statuses a decision may record.
pub const DECIDED: &[&str] = &["yes", "partly", "missing", "n/a", "deferred", "unknown"];

pub const OBSERVED_HEADER: &str = "project\tstatus\tsince\tevidence";

/// The directory containing `crosscut/concerns/`, given or found by searching upwards.
pub fn find_root(root: Option<PathBuf>) -> Result<PathBuf, String> {
    let has = |dir: &Path| dir.join("crosscut/concerns").is_dir();
    let root =
        match root {
            Some(root) if has(&root) => root,
            Some(root) => return Err(format!("no crosscut/concerns/ in {}", root.display())),
            None => {
                let here = std::env::current_dir()
                    .map_err(|e| format!("cannot read current directory: {e}"))?;
                here.ancestors().find(|dir| has(dir)).map(Path::to_path_buf).ok_or(
                "no crosscut/concerns/ here or above; to set one up, ask your agent to use \
                 CrossCut (see `crosscut prompt setup`)",
            )?
            }
        };
    Ok(root.canonicalize().unwrap_or(root))
}

#[derive(Clone, Debug)]
pub struct Project {
    pub name: String,
    pub dir: PathBuf,
}

/// Projects listed in `crosscut/projects`: one path or glob per line, relative to the root.
/// A project's name is its directory name.
pub fn projects(root: &Path) -> Result<Vec<Project>, String> {
    let path = root.join("crosscut/projects");
    let text = std::fs::read_to_string(&path).map_err(|e| {
        format!(
            "cannot read {}: {e} (list one project path or glob per line)",
            path.display()
        )
    })?;
    let mut found: Vec<Project> = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let dirs = expand(root, line);
        if dirs.is_empty() {
            eprintln!(
                "crosscut: {}: `{line}` matches no directory",
                path.display()
            );
        }
        for dir in dirs {
            let dir = dir.canonicalize().unwrap_or(dir);
            let name = dir
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if let Some(other) = found.iter().find(|p| p.name == name) {
                if other.dir != dir {
                    return Err(format!(
                        "two projects are named `{name}`: {} and {}",
                        other.dir.display(),
                        dir.display()
                    ));
                }
                continue;
            }
            found.push(Project { name, dir });
        }
    }
    Ok(found)
}

/// Directories matching a relative path whose components may contain `*`.
fn expand(root: &Path, pattern: &str) -> Vec<PathBuf> {
    let mut current = vec![root.to_path_buf()];
    for part in pattern.split('/').filter(|p| !p.is_empty() && *p != ".") {
        let mut next = Vec::new();
        for dir in &current {
            if part.contains('*') {
                let mut matches: Vec<PathBuf> = std::fs::read_dir(dir)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .map(|entry| entry.path())
                    .filter(|path| path.is_dir())
                    .filter(|path| {
                        let name = path.file_name().unwrap_or_default().to_string_lossy();
                        !name.starts_with('.') && wildcard(part, &name)
                    })
                    .collect();
                matches.sort();
                next.extend(matches);
            } else {
                next.push(dir.join(part));
            }
        }
        current = next;
    }
    current.into_iter().filter(|dir| dir.is_dir()).collect()
}

fn wildcard(pattern: &str, name: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == name,
        Some((head, rest)) => {
            let Some(tail) = name.strip_prefix(head) else {
                return false;
            };
            (0..=tail.len())
                .filter(|i| tail.is_char_boundary(*i))
                .any(|i| wildcard(rest, &tail[i..]))
        }
    }
}

pub enum Mechanism {
    /// Tiers 1 and 2: an executable that prints `<status>: <evidence>` for one project.
    Exec(PathBuf),
    /// Tier 3: a prompt a coding agent answers for one project.
    Prompt(String),
}

pub struct Concern {
    pub slug: String,
    pub dir: PathBuf,
    pub definition: String,
}

impl Concern {
    pub fn name(&self) -> &str {
        self.definition
            .lines()
            .find_map(|l| l.strip_prefix("# "))
            .unwrap_or(&self.slug)
            .trim()
    }

    pub fn mechanism(&self) -> Option<Mechanism> {
        let exec = self.dir.join("check");
        if exec.is_file() {
            return Some(Mechanism::Exec(exec));
        }
        std::fs::read_to_string(self.dir.join("check.md"))
            .ok()
            .map(Mechanism::Prompt)
    }

    /// Decisions: list items under `## Decisions`, written `- **project**: status: reason`.
    pub fn decisions(&self) -> Vec<Decision> {
        let mut lines = self.definition.lines();
        if !lines
            .by_ref()
            .any(|l| l.trim().eq_ignore_ascii_case("## decisions"))
        {
            return Vec::new();
        }
        lines
            .take_while(|l| !l.starts_with("## "))
            .filter_map(|line| {
                let (project, rest) = line.trim().strip_prefix("- **")?.split_once("**")?;
                let rest = rest.trim_start_matches([':', ' ']);
                let (status, reason) = split_status(rest);
                DECIDED.contains(&status.as_str()).then(|| Decision {
                    project: project.trim().to_string(),
                    status,
                    reason,
                })
            })
            .collect()
    }

    pub fn observed_path(&self) -> PathBuf {
        self.dir.join("observed.tsv")
    }

    pub fn observations(&self) -> Vec<Observation> {
        read_observations(&self.observed_path())
    }
}

pub struct Decision {
    pub project: String,
    pub status: String,
    pub reason: String,
}

/// Splits `status: rest` or `status — rest` into a lowercase status word and the rest.
pub fn split_status(text: &str) -> (String, String) {
    let text = text.trim();
    let end = text
        .find(|c: char| c == ':' || c.is_whitespace())
        .unwrap_or(text.len());
    let status = text[..end].to_lowercase();
    let rest = text[end..]
        .trim_start_matches([':', ' ', '—', '-'])
        .trim()
        .to_string();
    (status, rest)
}

pub fn concerns(root: &Path) -> Result<Vec<Concern>, String> {
    let dir = root.join("crosscut/concerns");
    let entries =
        std::fs::read_dir(&dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let definition = path.join("concern.md");
        if definition.is_file() {
            let text = std::fs::read_to_string(&definition)
                .map_err(|e| format!("cannot read {}: {e}", definition.display()))?;
            let slug = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            found.push(Concern {
                slug,
                dir: path,
                definition: text,
            });
        }
    }
    found.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(found)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Observation {
    pub project: String,
    pub status: String,
    pub since: String,
    pub evidence: String,
}

fn read_observations(path: &Path) -> Vec<Observation> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.is_empty() && *line != OBSERVED_HEADER)
        .filter_map(|line| {
            let mut cells = line.splitn(4, '\t');
            Some(Observation {
                project: cells.next()?.to_string(),
                status: cells.next()?.to_string(),
                since: cells.next()?.to_string(),
                evidence: cells.next().unwrap_or("").to_string(),
            })
        })
        .collect()
}

/// Writes observations sorted by project, so parallel edits to different rows merge cleanly.
pub fn write_observations(path: &Path, rows: &mut [Observation]) -> Result<(), String> {
    rows.sort_by(|a, b| a.project.cmp(&b.project));
    let mut text = String::from(OBSERVED_HEADER);
    text.push('\n');
    for row in rows.iter() {
        let evidence = row.evidence.replace(['\t', '\n', '\r'], " ");
        text.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            row.project,
            row.status,
            row.since,
            evidence.trim()
        ));
    }
    std::fs::write(path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))
}
