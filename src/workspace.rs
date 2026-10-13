//! Making a workspace: finding the repositories already on this machine, and cloning the
//! chosen ones side by side, so CrossCut never works in anyone's own checkouts.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// One repository, with every local clone of it that was found.
#[derive(Debug, PartialEq)]
pub struct Repo {
    /// `owner/name` for a hosted repository, otherwise the directory name.
    pub name: String,
    /// Where to clone it from: its origin remote, or the local path if it has none.
    pub source: String,
    pub clones: Vec<PathBuf>,
}

/// Repositories at `dir` and up to `depth` levels below it, one entry per origin remote.
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
        match repos
            .iter_mut()
            .find(|r| remote.is_some() && r.name == name)
        {
            Some(repo) => repo.clones.push(clone),
            None => repos.push(Repo {
                name,
                source,
                clones: vec![clone],
            }),
        }
    }
    repos.sort_by_key(|r| r.name.to_lowercase());
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

/// Asks which repositories to pull in: numbers from `repos`, or `owner/name` for others.
pub fn choose(input: &mut impl BufRead, repos: &[Repo]) -> Vec<(String, String)> {
    loop {
        eprint!("\nPull in which? Numbers such as `1 3`, or owner/name for others > ");
        let _ = std::io::stderr().flush();
        let mut line = String::new();
        if input.read_line(&mut line).unwrap_or(0) == 0 {
            return Vec::new();
        }
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
        match picked {
            Some(picked) if !picked.is_empty() => return picked,
            _ => eprintln!(
                "  Numbers from 1 to {}, or owner/name, please.",
                repos.len()
            ),
        }
    }
}

fn github(name: &str) -> String {
    format!("git@github.com:{name}.git")
}

/// Clones each repository into `workspace/<name>`, reusing a clone already there. Returns
/// the project directories.
pub fn pull_in(workspace: &Path, repos: &[(String, String)]) -> Result<Vec<PathBuf>, String> {
    std::fs::create_dir_all(workspace)
        .map_err(|e| format!("cannot create {}: {e}", workspace.display()))?;
    let mut dirs = Vec::new();
    for (name, source) in repos {
        let dir = workspace.join(name.rsplit('/').next().unwrap_or(name));
        if dir.join(".git").is_dir() {
            eprintln!("  {name}: already here");
        } else {
            eprintln!("  {name}: cloning from {source}");
            let mut git = Command::new("git");
            git.args(["clone", "--quiet"]);
            // Hosted repositories fetch file contents lazily; local ones are hard-linked.
            if !Path::new(source).exists() {
                git.arg("--filter=blob:none");
            }
            let status = git
                .arg(source)
                .arg(&dir)
                .status()
                .map_err(|e| format!("cannot run git: {e}"))?;
            if !status.success() {
                return Err(format!("could not clone {name} from {source}"));
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
    fn clones_of_one_repository_are_one_entry_and_repositories_are_not_searched_inside() {
        let dir = tempfile::tempdir().unwrap();
        let config = |remote: &str| {
            format!("[core]\n\tbare = false\n[remote \"origin\"]\n\turl = {remote}\n\tfetch = x\n")
        };
        for (path, remote) in [
            (
                "task-a/DataCentral",
                Some("git@github.com:NZX/DataCentral.git"),
            ),
            (
                "task-b/DataCentral",
                Some("https://github.com/NZX/DataCentral"),
            ),
            ("tools/trunc", Some("git@github.com:maxeonyx/trunc.git")),
            (
                "tools/trunc/vendor/inner",
                Some("git@github.com:x/inner.git"),
            ),
            ("scratch", None),
            ("a/b/c/too-deep", Some("git@github.com:x/deep.git")),
        ] {
            let git = dir.path().join(path).join(".git");
            std::fs::create_dir_all(&git).unwrap();
            if let Some(remote) = remote {
                std::fs::write(git.join("config"), config(remote)).unwrap();
            }
        }
        let repos = find(dir.path(), 3);
        let names: Vec<(&str, usize)> = repos
            .iter()
            .map(|r| (r.name.as_str(), r.clones.len()))
            .collect();
        assert_eq!(
            names,
            [
                ("maxeonyx/trunc", 1),
                ("NZX/DataCentral", 2),
                ("scratch", 1)
            ]
        );
        assert_eq!(repos[1].source, "git@github.com:NZX/DataCentral.git");
        assert_eq!(
            repos[2].source,
            dir.path().join("scratch").display().to_string()
        );
    }

    #[test]
    fn choosing_takes_numbers_and_owner_names_and_asks_again_on_nonsense() {
        let repos = [Repo {
            name: "NZX/DataCentral".into(),
            source: "git@github.com:NZX/DataCentral.git".into(),
            clones: vec![],
        }];
        assert_eq!(
            choose(&mut "9\nx\n1 maxeonyx/trunc\n".as_bytes(), &repos),
            [
                (
                    "NZX/DataCentral".into(),
                    "git@github.com:NZX/DataCentral.git".into()
                ),
                (
                    "maxeonyx/trunc".into(),
                    "git@github.com:maxeonyx/trunc.git".into()
                ),
            ]
        );
    }
}
