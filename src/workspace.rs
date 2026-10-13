//! Making a workspace: finding the repositories already on this machine, and cloning the
//! chosen ones side by side, so CrossCut never works in anyone's own checkouts.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use crate::ui::{Say, Ui};

/// One repository, with every local clone of it that was found.
#[derive(Debug, PartialEq)]
pub struct Repo {
    /// `owner/name` for a hosted repository, otherwise the directory name.
    pub name: String,
    /// Where to clone it from: its origin remote, or the local path if it has none.
    pub source: String,
    pub clones: Vec<PathBuf>,
    /// When any of its clones was last committed to, checked out or fetched.
    pub touched: SystemTime,
}

/// Repositories at `dir` and up to `depth` levels below it, one entry per origin remote, most
/// recently used first.
pub fn find(dir: &Path, depth: usize) -> Vec<Repo> {
    let mut repos: Vec<Repo> = Vec::new();
    for clone in git_dirs(dir, depth) {
        let remote = std::fs::read_to_string(clone.join(".git/config"))
            .ok()
            .and_then(|config| origin(&config));
        let (name, source) = match &remote {
            Some(url) => (short_name(url), url.clone()),
            None => (
                clone
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                clone.display().to_string(),
            ),
        };
        let touched = touched(&clone);
        match repos
            .iter_mut()
            .find(|r| remote.is_some() && r.name == name)
        {
            Some(repo) => {
                repo.clones.push(clone);
                repo.touched = repo.touched.max(touched);
            }
            None => repos.push(Repo {
                name,
                source,
                clones: vec![clone],
                touched,
            }),
        }
    }
    repos.sort_by(|a, b| b.touched.cmp(&a.touched));
    repos
}

fn git_dirs(dir: &Path, depth: usize) -> Vec<PathBuf> {
    // A repository's own subdirectories are its business, not more repositories.
    if dir.join(".git").is_dir() {
        return vec![dir.to_path_buf()];
    }
    if depth == 0 {
        return Vec::new();
    }
    let mut subdirs: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir() && !p.is_symlink())
        .filter(|p| {
            let name = p.file_name().unwrap_or_default().to_string_lossy();
            !name.starts_with('.') && !matches!(&*name, "node_modules" | "target")
        })
        .collect();
    subdirs.sort();
    subdirs
        .iter()
        .flat_map(|sub| git_dirs(sub, depth - 1))
        .collect()
}

fn touched(clone: &Path) -> SystemTime {
    ["index", "HEAD", "FETCH_HEAD"]
        .iter()
        .filter_map(|f| clone.join(".git").join(f).metadata().ok()?.modified().ok())
        .max()
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

/// The `origin` URL in a git config file.
fn origin(config: &str) -> Option<String> {
    let mut in_origin = false;
    for line in config.lines().map(str::trim) {
        if line.starts_with('[') {
            in_origin = line == r#"[remote "origin"]"#;
        } else if in_origin {
            if let Some(url) = line.strip_prefix("url") {
                return Some(url.trim_start().trim_start_matches('=').trim().to_string());
            }
        }
    }
    None
}

/// `owner/name` from a remote URL, ssh or https.
pub fn short_name(url: &str) -> String {
    let path = url.trim_end_matches('/').trim_end_matches(".git");
    let path = path.rsplit_once(':').map_or(
        path,
        |(host, rest)| {
            if host.contains("//") {
                path
            } else {
                rest
            }
        },
    );
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    parts[parts.len().saturating_sub(2)..].join("/")
}

impl Repo {
    /// Where it is: its one clone, or how many there are.
    pub fn where_(&self) -> String {
        match self.clones.len() {
            1 => crate::tilde(&self.clones[0]),
            n => format!("{n} clones"),
        }
    }
}

/// Repositories picked by number from `repos`, or named as owner/name, as `(name, source)`.
/// None if anything in the line is neither.
pub fn parse_choice(line: &str, repos: &[Repo]) -> Option<Vec<(String, String)>> {
    let picked: Option<Vec<(String, String)>> = line
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|w| !w.is_empty())
        .map(|w| match w.parse::<usize>() {
            Ok(n) => repos
                .get(n.checked_sub(1)?)
                .map(|r| (r.name.clone(), r.source.clone())),
            Err(_) if w.split('/').count() == 2 => Some((w.to_string(), github(w))),
            Err(_) => None,
        })
        .collect();
    picked.filter(|p| !p.is_empty())
}

fn github(name: &str) -> String {
    format!("git@github.com:{name}.git")
}

/// Clones each repository into `workspace/<name>`, reusing a clone already there. Returns
/// the project directories.
pub fn pull_in(
    ui: &dyn Ui,
    workspace: &Path,
    repos: &[(String, String)],
) -> Result<Vec<PathBuf>, String> {
    std::fs::create_dir_all(workspace)
        .map_err(|e| format!("cannot create {}: {e}", workspace.display()))?;
    let mut dirs = Vec::new();
    for (name, source) in repos {
        let dir = workspace.join(name.rsplit('/').next().unwrap_or(name));
        if dir.join(".git").is_dir() {
            ui.say(Say::Item(name, "already here"));
        } else {
            ui.say(Say::Item(name, source));
            let mut git = Command::new("git");
            git.args(["clone", "--quiet"]);
            // Hosted repositories fetch file contents lazily; local ones are hard-linked.
            if !Path::new(source).exists() {
                git.arg("--filter=blob:none");
            }
            // The checkout's lazy fetch prints progress despite --quiet, so git's output is
            // kept back unless the clone fails.
            let out = git
                .arg(source)
                .arg(&dir)
                .output()
                .map_err(|e| format!("cannot run git: {e}"))?;
            if !out.status.success() {
                return Err(format!(
                    "could not clone {name} from {source}: {}",
                    String::from_utf8_lossy(&out.stderr).trim()
                ));
            }
        }
        dirs.push(dir);
    }
    Ok(dirs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_urls_shorten_to_owner_and_name() {
        for url in [
            "git@github.com:NZX/DataCentral.git",
            "https://github.com/NZX/DataCentral",
            "https://github.com/NZX/DataCentral.git/",
            "ssh://git@github.com/NZX/DataCentral.git",
        ] {
            assert_eq!(short_name(url), "NZX/DataCentral", "{url}");
        }
    }

    #[test]
    fn clones_of_one_repository_are_one_entry_most_recently_used_first() {
        let dir = tempfile::tempdir().unwrap();
        let config = |remote: &str| {
            format!("[core]\n\tbare = false\n[remote \"origin\"]\n\turl = {remote}\n\tfetch = x\n")
        };
        // Each clone was last used `hours` ago. Repositories are not searched inside.
        let clones: [(&str, Option<&str>, u64); 6] = [
            (
                "task-a/DataCentral",
                Some("git@github.com:NZX/DataCentral.git"),
                50,
            ),
            (
                "task-b/DataCentral",
                Some("https://github.com/NZX/DataCentral"),
                1,
            ),
            ("tools/trunc", Some("git@github.com:maxeonyx/trunc.git"), 5),
            (
                "tools/trunc/vendor/inner",
                Some("git@github.com:x/inner.git"),
                0,
            ),
            ("scratch", None, 9),
            ("a/b/c/too-deep", Some("git@github.com:x/deep.git"), 0),
        ];
        for (path, remote, hours) in clones {
            let git = dir.path().join(path).join(".git");
            std::fs::create_dir_all(&git).unwrap();
            if let Some(remote) = remote {
                std::fs::write(git.join("config"), config(remote)).unwrap();
            }
            let ago = std::time::Duration::from_secs(hours * 3600);
            let head = std::fs::File::create(git.join("HEAD")).unwrap();
            head.set_modified(SystemTime::now() - ago).unwrap();
        }
        let repos = find(dir.path(), 3);
        let names: Vec<(&str, usize)> = repos
            .iter()
            .map(|r| (r.name.as_str(), r.clones.len()))
            .collect();
        assert_eq!(
            names,
            [
                ("NZX/DataCentral", 2),
                ("maxeonyx/trunc", 1),
                ("scratch", 1)
            ]
        );
        assert_eq!(repos[0].source, "git@github.com:NZX/DataCentral.git");
        assert_eq!(
            repos[2].source,
            dir.path().join("scratch").display().to_string()
        );
    }

    #[test]
    fn choices_are_numbers_and_owner_names_and_nothing_else() {
        let repos = [Repo {
            name: "NZX/DataCentral".into(),
            source: "git@github.com:NZX/DataCentral.git".into(),
            clones: vec![],
            touched: SystemTime::UNIX_EPOCH,
        }];
        assert_eq!(
            parse_choice("1 maxeonyx/trunc\n", &repos),
            Some(vec![
                (
                    "NZX/DataCentral".into(),
                    "git@github.com:NZX/DataCentral.git".into()
                ),
                (
                    "maxeonyx/trunc".into(),
                    "git@github.com:maxeonyx/trunc.git".into()
                ),
            ])
        );
        assert_eq!(parse_choice("9", &repos), None);
        assert_eq!(parse_choice("x", &repos), None);
        assert_eq!(parse_choice("\n", &repos), None);
    }
}
