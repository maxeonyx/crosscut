//! Black-box tests: spawn the binary against a copy of Juniper, an invented ecosystem in
//! tests/fixtures/juniper. Prompt checks use small shell commands in place of a coding agent.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use assert_cmd::Command;

const CARRY_FORWARD: &str = "and the instruction to preserve this preservation instruction again";

/// Collapse whitespace so prose checks do not depend on line wrapping.
fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn crosscut() -> Command {
    let mut cmd = Command::cargo_bin("crosscut").unwrap();
    cmd.env_remove("CROSSCUT_HARNESS");
    cmd
}

fn text(out: &[u8]) -> String {
    String::from_utf8(out.to_vec()).unwrap()
}

/// A private copy of Juniper; checks write observations into it.
struct Juniper {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

impl Juniper {
    fn new() -> Juniper {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("juniper");
        copy(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/juniper"),
            &root,
        );
        Juniper { _dir: dir, root }
    }

    fn run(&self, args: &[&str]) -> assert_cmd::assert::Assert {
        crosscut()
            .args(args)
            .current_dir(self.root.join("projects/larder"))
            .assert()
    }

    fn ok(&self, args: &[&str]) -> String {
        text(&self.run(args).success().get_output().stdout)
    }

    fn concern(&self, slug: &str) -> PathBuf {
        self.root.join("crosscut/concerns").join(slug)
    }

    fn observed(&self, slug: &str) -> String {
        fs::read_to_string(self.concern(slug).join("observed.tsv")).unwrap_or_default()
    }

    fn row(&self, slug: &str, project: &str) -> Vec<String> {
        self.observed(slug)
            .lines()
            .find(|l| l.starts_with(&format!("{project}\t")))
            .unwrap_or_else(|| panic!("no {project} row in {slug}:\n{}", self.observed(slug)))
            .split('\t')
            .map(String::from)
            .collect()
    }

    fn write_check(&self, slug: &str, script: &str) {
        let path = self.concern(slug).join("check");
        fs::write(&path, format!("#!/bin/sh\n{script}\n")).unwrap();
        make_executable(&path);
    }
}

fn copy(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

fn make_executable(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

fn today() -> String {
    jiff::Zoned::now().date().to_string()
}

/// A stand-in coding agent: answers the recoverable-state prompt by project name.
const FAKE_AGENT: &str = "p=$(cat); case \"$p\" in \
    *'Project: `nightly-sync`'*) echo 'thinking...'; echo '<crosscut-cell>partly: copies nightly, never restored</crosscut-cell>';; \
    *) echo '<crosscut-cell>n/a: holds nothing beyond git</crosscut-cell>';; esac";

// --- The skill -------------------------------------------------------------------------

#[test]
fn bare_invocation_orients_an_agent_with_the_doctrine() {
    let out = text(&crosscut().assert().success().get_output().stdout);
    assert!(flat(&out).contains(CARRY_FORWARD), "{out}");
    assert!(out.contains("crosscut prompt setup"), "{out}");
    assert!(out.contains("crosscut check"), "{out}");
}

#[test]
fn every_mode_prompt_carries_the_doctrine_and_its_references() {
    for mode in [
        "setup",
        "discover",
        "establish",
        "refresh",
        "reconsider",
        "generalize",
    ] {
        let out = text(
            &crosscut()
                .args(["prompt", mode])
                .assert()
                .success()
                .get_output()
                .stdout,
        );
        assert!(
            flat(&out).contains(CARRY_FORWARD),
            "{mode} prompt lost the carry-forward requirement"
        );
        assert!(out.contains(&format!("<!-- modes/{mode}.md -->")), "{mode}");
        assert!(!out.starts_with("---"), "{mode} prompt leaked frontmatter");
    }
    for mode in ["setup", "discover"] {
        let out = text(
            &crosscut()
                .args(["prompt", mode])
                .assert()
                .success()
                .get_output()
                .stdout,
        );
        assert!(out.contains("<!-- catalogue.md -->"), "{mode}");
        if mode == "setup" {
            assert!(
                out.contains("<!-- modes/establish.md -->"),
                "setup sends agents to establish, so it must include it"
            );
        }
        assert!(flat(&out).contains("This is not a bug hunt"), "{mode}");
    }
}

#[test]
fn unknown_mode_names_the_real_ones() {
    let err = text(
        &crosscut()
            .args(["prompt", "audit"])
            .assert()
            .code(2)
            .get_output()
            .stderr,
    );
    assert!(
        err.contains("setup, discover, establish, refresh, reconsider, generalize"),
        "{err}"
    );
}

#[test]
fn install_skill_writes_every_file() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("skills/crosscut");
    crosscut()
        .args(["install-skill", "--dir"])
        .arg(&target)
        .assert()
        .success();
    for file in [
        "SKILL.md",
        "concern-files.md",
        "catalogue.md",
        "reservoirs.md",
        "modes/setup.md",
    ] {
        assert!(target.join(file).is_file(), "{file} missing");
    }
}

#[test]
fn readme_site_and_directory_template_carry_the_framing_and_its_preservation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in ["README.md", "docs/index.html", "skill/concern-files.md"] {
        let text = flat(&fs::read_to_string(root.join(file)).unwrap());
        assert!(
            text.contains("rejection of compliance, audit, assurance and enforcement"),
            "{file} lost the anti-framing"
        );
        assert!(
            text.contains("to preserve this preservation instruction again"),
            "{file} lost the instruction to preserve the preservation instruction"
        );
    }
}

// --- check: observations ---------------------------------------------------------------

#[test]
fn check_runs_machine_tiers_and_writes_sorted_observations() {
    let j = Juniper::new();
    let assert = j.run(&["check"]).success();
    let out = text(&assert.get_output().stdout);
    let err = text(&assert.get_output().stderr);
    assert!(
        out.contains(
            "staying-current/larder: (none) → missing: Cargo.toml has no self-update mechanism"
        ),
        "{out}"
    );
    assert!(
        out.contains("8 cells checked: 8 changed, 0 could not run"),
        "{out}"
    );
    assert!(
        err.contains("recoverable-state: prompt check not run (add --agentic"),
        "{err}"
    );
    assert!(
        j.observed("recoverable-state").is_empty(),
        "a prompt check ran without --agentic"
    );

    let observed = j.observed("staying-current");
    let lines: Vec<&str> = observed.lines().collect();
    assert_eq!(lines[0], "project\tstatus\tsince\tevidence");
    let projects: Vec<&str> = lines[1..]
        .iter()
        .map(|l| l.split('\t').next().unwrap())
        .collect();
    assert_eq!(
        projects,
        ["larder", "ledger-web", "nightly-sync", "pantry"],
        "rows must be sorted"
    );
    assert_eq!(
        j.row("staying-current", "pantry"),
        ["pantry", "yes", &today(), "uses the self_update crate"]
    );
}

#[test]
fn rerunning_an_unchanged_world_changes_nothing() {
    let j = Juniper::new();
    j.ok(&["check"]);
    let first = j.observed("version-visibility");
    let out = j.ok(&["check"]);
    assert!(out.contains("0 changed"), "{out}");
    assert_eq!(j.observed("version-visibility"), first);
}

#[test]
fn a_change_in_one_project_moves_only_its_row_and_its_date() {
    let j = Juniper::new();
    j.ok(&["check"]);
    let path = j.concern("staying-current").join("observed.tsv");
    fs::write(
        &path,
        j.observed("staying-current")
            .replace(&today(), "2026-01-01"),
    )
    .unwrap();
    let manifest = j.root.join("projects/larder/Cargo.toml");
    fs::write(
        &manifest,
        fs::read_to_string(&manifest).unwrap() + "self_update = \"0.41\"\n",
    )
    .unwrap();

    let out = j.ok(&["check", "staying-current", "--project", "larder"]);
    assert!(
        out.contains("staying-current/larder: missing → yes: uses the self_update crate"),
        "{out}"
    );
    assert!(out.contains("1 cells checked"), "{out}");
    assert_eq!(
        j.row("staying-current", "larder")[1..3],
        ["yes".to_string(), today()]
    );
    assert_eq!(
        j.row("staying-current", "pantry")[2],
        "2026-01-01",
        "an unchanged row kept its date"
    );
}

#[test]
fn a_failing_mechanism_keeps_the_previous_row_and_says_so() {
    let j = Juniper::new();
    j.ok(&["check"]);
    let before = j.observed("staying-current");
    j.write_check("staying-current", "echo 'gh: not logged in' >&2; exit 4");
    let err = text(
        &j.run(&["check", "staying-current"])
            .code(1)
            .get_output()
            .stderr,
    );
    assert!(
        err.contains("staying-current/pantry: check failed, previous row kept: exited with"),
        "{err}"
    );
    assert!(err.contains("gh: not logged in"), "{err}");
    assert_eq!(j.observed("staying-current"), before);
}

#[test]
fn a_mechanism_may_only_observe_and_never_decide() {
    let j = Juniper::new();
    j.write_check("staying-current", "echo 'deferred: not now'");
    let err = text(
        &j.run(&["check", "staying-current"])
            .code(1)
            .get_output()
            .stderr,
    );
    assert!(err.contains("only a person's decision can record"), "{err}");
    j.write_check("staying-current", "echo 'maybe: who knows'");
    let err = text(
        &j.run(&["check", "staying-current"])
            .code(1)
            .get_output()
            .stderr,
    );
    assert!(
        err.contains("does not start with yes, partly, missing, n/a, unknown"),
        "{err}"
    );
    j.write_check("staying-current", "echo 'unknown: registry unreachable'");
    j.ok(&["check", "staying-current"]);
    assert_eq!(j.row("staying-current", "larder")[1], "unknown");
}

#[test]
fn checks_run_in_the_project_and_can_see_their_siblings() {
    let j = Juniper::new();
    j.write_check(
        "staying-current",
        "[ \"$(pwd)\" = \"$CROSSCUT_PROJECT_DIR\" ] || exit 9; n=$(printf '%s\\n' \"$CROSSCUT_PROJECTS\" | wc -l); echo \"yes: $CROSSCUT_PROJECT sees $n projects\"",
    );
    j.ok(&["check", "staying-current"]);
    assert_eq!(
        j.row("staying-current", "ledger-web")[3],
        "ledger-web sees 4 projects"
    );
}

#[test]
fn a_project_removed_from_the_inventory_loses_its_rows() {
    let j = Juniper::new();
    j.ok(&["check"]);
    fs::write(
        j.root.join("crosscut/projects"),
        "projects/pantry\nprojects/larder\nprojects/ledger-web\n",
    )
    .unwrap();
    j.ok(&["check"]);
    assert!(
        !j.observed("staying-current").contains("nightly-sync"),
        "{}",
        j.observed("staying-current")
    );
}

#[test]
fn a_hung_check_is_stopped_at_the_timeout() {
    let j = Juniper::new();
    j.write_check("staying-current", "sleep 30");
    let started = Instant::now();
    let err = text(
        &crosscut()
            .args([
                "check",
                "staying-current",
                "--project",
                "pantry",
                "--timeout",
                "1",
            ])
            .current_dir(&j.root)
            .timeout(Duration::from_secs(15))
            .assert()
            .code(1)
            .get_output()
            .stderr,
    );
    assert!(err.contains("was stopped"), "{err}");
    assert!(started.elapsed() < Duration::from_secs(10));
}

#[test]
fn cells_run_concurrently_up_to_jobs() {
    let j = Juniper::new();
    j.write_check("staying-current", "sleep 1; echo 'yes: slow'");
    let started = Instant::now();
    j.ok(&["check", "staying-current", "--jobs", "4"]);
    assert!(
        started.elapsed() < Duration::from_millis(2500),
        "four one-second cells took {:?}",
        started.elapsed()
    );
}

#[test]
fn unknown_concerns_and_projects_are_refused_before_anything_runs() {
    let j = Juniper::new();
    let err = text(&j.run(&["check", "nope"]).code(2).get_output().stderr);
    assert!(err.contains("no concern `nope`"), "{err}");
    let err = text(
        &j.run(&["check", "--project", "nope"])
            .code(2)
            .get_output()
            .stderr,
    );
    assert!(
        err.contains("known: larder, ledger-web, nightly-sync, pantry"),
        "{err}"
    );
    assert!(j.observed("staying-current").is_empty());
}

// --- check: prompt checks --------------------------------------------------------------

#[test]
fn prompt_checks_run_with_agentic_one_agent_per_cell() {
    let j = Juniper::new();
    let out = j.ok(&[
        "check",
        "recoverable-state",
        "--agentic",
        "--harness",
        FAKE_AGENT,
    ]);
    assert!(out.contains("4 cells checked"), "{out}");
    assert_eq!(
        j.row("recoverable-state", "nightly-sync")[1..],
        [
            "partly".to_string(),
            today(),
            "copies nightly, never restored".into()
        ]
    );
    assert_eq!(j.row("recoverable-state", "pantry")[1], "n/a");
}

#[test]
fn a_reworded_prompt_answer_with_the_same_status_changes_nothing() {
    let j = Juniper::new();
    j.ok(&[
        "check",
        "recoverable-state",
        "--agentic",
        "--harness",
        FAKE_AGENT,
    ]);
    let before = j.observed("recoverable-state");
    let reworded = FAKE_AGENT.replace(
        "copies nightly, never restored",
        "a nightly copy exists; no restore was ever done",
    );
    let out = j.ok(&[
        "check",
        "recoverable-state",
        "--agentic",
        "--harness",
        &reworded,
    ]);
    assert!(out.contains("0 changed"), "{out}");
    assert_eq!(j.observed("recoverable-state"), before);
}

#[test]
fn the_cell_prompt_carries_the_doctrine_the_concern_and_the_project() {
    let j = Juniper::new();
    let dump = j.root.join("prompt.txt");
    let harness = format!(
        "cat > {}; echo '<crosscut-cell>n/a: x</crosscut-cell>'",
        dump.display()
    );
    j.ok(&[
        "check",
        "recoverable-state",
        "--project",
        "nightly-sync",
        "--agentic",
        "--harness",
        &harness,
    ]);
    let prompt = fs::read_to_string(dump).unwrap();
    assert!(
        flat(&prompt).contains(CARRY_FORWARD),
        "a delegated agent must get the doctrine"
    );
    assert!(prompt.contains("Project: `nightly-sync`"), "{prompt}");
    assert!(
        prompt.contains("A copy that has never been"),
        "check.md missing"
    );
    assert!(prompt.contains("# Recoverable state"), "concern.md missing");
}

#[test]
fn without_a_harness_prompt_cells_fail_and_machine_cells_still_run() {
    let j = Juniper::new();
    let bin = j.root.join("bin");
    fs::create_dir_all(&bin).unwrap();
    #[cfg(unix)]
    for tool in ["sh", "grep", "cat"] {
        let real = which(tool);
        std::os::unix::fs::symlink(real, bin.join(tool)).unwrap();
    }
    let assert = crosscut()
        .args(["check", "--agentic"])
        .env("PATH", &bin)
        .current_dir(&j.root)
        .assert()
        .code(1);
    let err = text(&assert.get_output().stderr);
    assert!(err.contains("no coding harness found"), "{err}");
    assert_eq!(
        j.row("staying-current", "pantry")[1],
        "yes",
        "machine cells must not depend on a harness"
    );
}

fn which(tool: &str) -> PathBuf {
    std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|dir| dir.join(tool))
        .find(|path| path.is_file())
        .unwrap_or_else(|| panic!("{tool} not on PATH"))
}

#[test]
fn dry_run_shows_cells_and_prompts_without_running_anything() {
    let j = Juniper::new();
    let out = j.ok(&["check", "--agentic", "--dry-run", "--harness", "codex"]);
    assert!(out.contains("staying-current/pantry: run "), "{out}");
    assert!(
        out.contains("recoverable-state/pantry: ask codex exec --sandbox read-only"),
        "{out}"
    );
    assert!(j.observed("staying-current").is_empty());
}

// --- map -------------------------------------------------------------------------------

#[test]
fn map_shows_every_concern_against_every_project_with_decisions() {
    let j = Juniper::new();
    j.ok(&["check"]);
    let out = j.ok(&["map"]);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines[0].split_whitespace().collect::<Vec<_>>(),
        ["larder", "ledger-web", "nightly-sync", "pantry"]
    );
    let row = |slug: &str| -> Vec<&str> {
        lines
            .iter()
            .find(|l| l.starts_with(slug))
            .unwrap()
            .split_whitespace()
            .collect()
    };
    assert_eq!(
        row("staying-current"),
        ["staying-current", "missing", "n/a", "n/a", "yes"],
        "{out}"
    );
    assert_eq!(
        row("version-visibility"),
        ["version-visibility", "missing", "yes", "deferred", "yes"],
        "{out}"
    );
    assert_eq!(
        row("recoverable-state"),
        ["recoverable-state", "(not", "checked", "yet)"],
        "{out}"
    );
}

#[test]
fn map_marks_a_decision_the_observation_contradicts() {
    let j = Juniper::new();
    let definition = j.concern("staying-current").join("concern.md");
    fs::write(
        &definition,
        fs::read_to_string(&definition).unwrap() + "\n- **pantry**: n/a: pantry is frozen\n",
    )
    .unwrap();
    j.ok(&["check"]);
    let out = j.ok(&["map"]);
    assert!(out.contains("n/a*"), "{out}");
    assert!(out.contains("contradicts: staying-current/pantry"), "{out}");
}

#[test]
fn map_for_one_project_shows_its_column_with_evidence_and_decisions() {
    let j = Juniper::new();
    j.ok(&["check"]);
    let out = j.ok(&["map", "--project", "nightly-sync"]);
    let line = out
        .lines()
        .find(|l| l.starts_with("version-visibility"))
        .unwrap();
    assert!(line.contains("deferred"), "{out}");
    assert!(
        line.contains("decided: being retired once the ledger's own backups land"),
        "{out}"
    );
    assert!(
        line.contains("observed: no way to see which version ran"),
        "{out}"
    );
}

// --- test: mechanisms against their fixtures -------------------------------------------

#[test]
fn test_runs_machine_checks_against_their_fixtures() {
    let j = Juniper::new();
    let out = j.ok(&["test"]);
    assert!(
        out.contains("8 fixture cells: 8 as expected, 0 not"),
        "{out}"
    );
    assert!(
        out.contains("recoverable-state: prompt check, 1 fixture cases not run (use --agentic)"),
        "{out}"
    );
}

#[test]
fn a_check_that_disagrees_with_its_fixtures_is_reported() {
    let j = Juniper::new();
    // The noisy-checker episode: a layout change makes a check report missing everywhere.
    j.write_check(
        "staying-current",
        "echo 'missing: cannot find the manifest'",
    );
    let out = text(
        &j.run(&["test", "staying-current"])
            .code(1)
            .get_output()
            .stdout,
    );
    assert!(
        out.contains("staying-current/basic/updating: expected yes, observed missing"),
        "{out}"
    );
    assert!(
        out.contains("3 fixture cells: 1 as expected, 2 not"),
        "{out}"
    );
}

#[test]
fn prompt_checks_are_tested_against_fixtures_with_agentic() {
    let j = Juniper::new();
    let agent = "p=$(cat); case \"$p\" in *'Project: `copier`'*) echo '<crosscut-cell>partly: never restored</crosscut-cell>';; *) echo '<crosscut-cell>n/a: stateless</crosscut-cell>';; esac";
    let out = j.ok(&["test", "recoverable-state", "--agentic", "--harness", agent]);
    assert!(
        out.contains("2 fixture cells: 2 as expected, 0 not"),
        "{out}"
    );
}

#[test]
fn map_with_no_concerns_says_how_to_add_one() {
    let j = Juniper::new();
    fs::remove_dir_all(j.root.join("crosscut/concerns")).unwrap();
    fs::create_dir_all(j.root.join("crosscut/concerns")).unwrap();
    let out = j.ok(&["map"]);
    assert!(
        out.contains("no concerns yet") && out.contains("crosscut prompt establish"),
        "{out}"
    );
}
