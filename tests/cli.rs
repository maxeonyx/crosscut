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
        "unpack",
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
    for mode in ["setup", "unpack"] {
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
        err.contains("setup, unpack, establish, refresh, reconsider, generalize"),
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
        out.contains("no cross-cutting concerns yet") && out.contains("crosscut prompt establish"),
        "{out}"
    );
}

// --- unpack ----------------------------------------------------------------------------

/// A stand-in agent: describes a project, proposes two cards from descriptions, and reviews
/// to a cell prompt. It saves each prompt it is given, so tests can read them.
const UNPACKING_AGENT: &str = r#"p=$(cat); n=$(ls "$PROMPTS" 2>/dev/null | wc -l); mkdir -p "$PROMPTS"; printf '%s' "$p" > "$PROMPTS/$n.md"
case "$p" in
*'Describe the project in'*) echo 'A small Python API that serves records.';;
*'Read each title below once'*) printf 'Ignored prose.\n<crosscut-reword>\nlog-once: keep\nas-of: Derived data should say how old it is\n</crosscut-reword>\n';;
*'Put these concerns in order of leverage'*) printf 'Ignored prose.\n<crosscut-order>\nas-of\nlog-once\n</crosscut-order>\n';;
*'Unpack concerns for these projects'*) cat <<'JSON'
Thinking out loud first.
<crosscut-cards>
{"cards": [
 {"slug": "log-once", "title": "Each error should be logged once, where it is handled",
  "justification": "Only the handler knows how serious an error is.",
  "stories": ["As an operator paged at night, I want one line per failure, so I need each error logged once."],
  "here": [{"project": "api", "story": "As the api's on-call developer, I want to count real failures, so I need each logged once."}],
  "how_to_review": "Search for log calls inside except blocks that re-raise."},
 {"slug": "as-of", "title": "Derived data that can go stale should carry its as-of time",
  "justification": "A failed refresh looks like a quiet day.", "stories": ["As a reader of a dashboard, I want to trust its numbers, so I need to know how old they are."],
  "here": [{"project": "api", "story": "As a client of the api, I want current records, so I need to know their age."}], "how_to_review": "Find derived tables and reports."}
]}
</crosscut-cards>
JSON
;;
*) echo '<crosscut-findings>
- `src/repo.py:12` logs then re-raises; drop the log line.
</crosscut-findings>
<crosscut-cell>partly: one layer logs and re-raises</crosscut-cell>';;
esac"#;

/// An empty workspace holding one project, `api`.
fn workspace() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("api/src")).unwrap();
    fs::write(dir.path().join("api/src/repo.py"), "print('hi')\n").unwrap();
    dir
}

fn unpack(dir: &Path, args: &[&str], answers: &str) -> (String, String) {
    let out = crosscut()
        .arg("unpack")
        .args(args)
        .args(["--harness", UNPACKING_AGENT])
        .env("PROMPTS", dir.join("prompts"))
        .current_dir(dir)
        .write_stdin(answers)
        .assert()
        .success()
        .get_output()
        .clone();
    (text(&out.stdout), text(&out.stderr))
}

fn prompt_number(dir: &Path, n: usize) -> String {
    fs::read_to_string(dir.join("prompts").join(format!("{n}.md"))).unwrap()
}

#[test]
fn unpack_describes_floats_cards_keeps_answers_and_reviews_what_was_accepted() {
    let ws = workspace();
    let dir = ws.path();
    // Keep the description, see the overview, decline the first card, accept the second.
    let (out, err) = unpack(dir, &["api"], "\n\nd\nwe already show refresh times\na\n");

    assert!(
        err.contains("api  A small Python API that serves records."),
        "{err}"
    );
    assert_eq!(
        fs::read_to_string(dir.join("crosscut/about/api.md")).unwrap(),
        "A small Python API that serves records.\n"
    );
    assert!(err.contains("What happens next"), "{err}");
    let overview = err
        .find("2 cross-cutting concerns highest leverage first")
        .expect("an overview");
    assert!(
        err[overview..].starts_with(
            "2 cross-cutting concerns highest leverage first\n\n   1  Derived data should say how old it is  api\n   2  Each error"
        ),
        "the prioritising pass reordered the cards: {err}"
    );
    assert!(
        !err.contains("Ignored prose"),
        "only the order reaches the person"
    );
    assert!(
        err.contains("2/2  Each error should be logged once"),
        "{err}"
    );
    assert!(
        err.contains("Stakeholder stories\n  • As an operator paged at night")
            && err.contains("Here\n  api  As the api's on-call developer"),
        "{err}"
    );
    assert!(
        err.contains("1 accepted 1 declined, 0 skipped, of 2"),
        "{err}"
    );
    assert_eq!(
        fs::read_to_string(dir.join("crosscut/projects")).unwrap(),
        "api\n"
    );

    let concern = dir.join("crosscut/concerns/log-once");
    let definition = fs::read_to_string(concern.join("concern.md")).unwrap();
    assert!(definition.starts_with("# Each error should be logged once, where it is handled\n"));
    assert!(
        definition.contains("## Stakeholder stories\n\n- As an operator paged at night")
            && definition.contains("## Here\n\n- **api**: As the api's on-call developer"),
        "{definition}"
    );
    assert!(definition.contains("## Decisions"), "{definition}");
    let check = fs::read_to_string(concern.join("check.md")).unwrap();
    assert!(
        check.contains("Search for log calls inside except blocks"),
        "{check}"
    );

    let declined = fs::read_to_string(dir.join("crosscut/declined.md")).unwrap();
    assert!(
        declined.contains(
            "- Derived data should say how old it is: we already show refresh times"
        ),
        "{declined}"
    );
    assert!(!dir.join("crosscut/concerns/as-of").exists());

    // The accepted concern was reviewed in every project, and its findings kept.
    let observed = fs::read_to_string(concern.join("observed.tsv")).unwrap();
    assert!(observed.contains("api\tpartly\t"), "{observed}");
    let findings = fs::read_to_string(concern.join("findings/api.md")).unwrap();
    assert!(
        findings.contains("`src/repo.py:12` logs then re-raises"),
        "{findings}"
    );
    assert!(out.contains("partly"), "the map is shown at the end: {out}");
    let report = fs::read_to_string(dir.join("crosscut/report.html")).unwrap();
    assert!(
        report.contains("Each error should be logged once"),
        "report has the concern"
    );
}

#[test]
fn unpacking_sees_descriptions_not_code() {
    let ws = workspace();
    let dir = ws.path();
    unpack(dir, &["api"], "a much better description\n\nq\n");
    assert!(prompt_number(dir, 0).contains("Describe the project in"));
    let unpacking = prompt_number(dir, 1);
    assert!(
        unpacking.contains("### `api`\n\na much better description"),
        "{unpacking}"
    );
    assert!(
        !unpacking.contains(&*dir.join("api").to_string_lossy()),
        "no paths to the code"
    );
    assert!(
        unpacking.contains(CARRY_FORWARD),
        "the doctrine travels with the prompt"
    );
    assert!(
        unpacking.contains("never from its code"),
        "the unpack mode is included"
    );
    assert!(
        unpacking.contains("Elapsed time should be measured with a monotonic clock"),
        "and the seeds"
    );
}

#[test]
fn the_next_unpacking_reuses_descriptions_and_knows_what_was_kept_and_declined() {
    let ws = workspace();
    let dir = ws.path();
    unpack(dir, &["api"], "\n\nd\nwe already show refresh times\na\n");
    unpack(dir, &[], "\nq\n");
    // Prompts: describe, unpack, reword, order, review, then only the second unpacking,
    // rewording and order.
    assert_eq!(fs::read_dir(dir.join("prompts")).unwrap().count(), 8);
    let second = prompt_number(dir, 5);
    assert!(
        flat(&second).contains(
            "must not propose again: - Each error should be logged once, where it is handled"
        ),
        "{second}"
    );
    assert!(
        second.contains("we already show refresh times"),
        "declined reasons are passed on"
    );
}

#[test]
fn saved_cards_float_again_without_an_agent_minus_those_already_kept() {
    let ws = workspace();
    let dir = ws.path();
    // Decline the first card, accept the second, which is reviewed.
    unpack(dir, &["api"], "\n\nd\n\na\n");
    let (_, err) = unpack(dir, &["--cards", "crosscut/landscape.json"], "\ns\n");
    assert!(err.contains("1/1  Derived data should say how old it is"), "{err}");
    assert_eq!(
        fs::read_dir(dir.join("prompts")).unwrap().count(),
        5,
        "describe, unpack, reword, order, review: no new agent run"
    );
}

#[test]
fn an_unreadable_answer_is_kept_for_inspection() {
    let ws = workspace();
    fs::create_dir_all(ws.path().join("crosscut/about")).unwrap();
    fs::write(ws.path().join("crosscut/about/api.md"), "An API.\n").unwrap();
    let out = crosscut()
        .args([
            "unpack",
            "api",
            "--harness",
            "cat >/dev/null; echo 'no cards here'",
        ])
        .current_dir(ws.path())
        .assert()
        .failure()
        .get_output()
        .clone();
    assert!(
        text(&out.stderr).contains("no <crosscut-cards> answer"),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(
        fs::read_to_string(ws.path().join("crosscut/landscape.reply.md")).unwrap(),
        "no cards here\n"
    );
}

// --- the browser -----------------------------------------------------------------------

/// One HTTP exchange with the page's server, returning the response body. Reads up to the
/// declared Content-Length, so a server that resets the connection after answering is fine.
fn http(url: &str, method: &str, path: &str, body: &str) -> String {
    use std::io::{Read, Write};
    let addr = url.trim_start_matches("http://").trim_end_matches('/');
    let mut stream = std::net::TcpStream::connect(addr).unwrap();
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    let mut response = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let text = String::from_utf8_lossy(&response).into_owned();
        if let Some((head, rest)) = text.split_once("\r\n\r\n") {
            let length: usize = head
                .lines()
                .find_map(|l| {
                    l.to_lowercase()
                        .strip_prefix("content-length:")
                        .map(|v| v.trim().parse().unwrap())
                })
                .unwrap_or(0);
            if rest.len() >= length {
                return rest[..length].to_string();
            }
        }
        let n = stream.read(&mut chunk).unwrap();
        assert!(n > 0, "the server closed before answering: {text}");
        response.extend_from_slice(&chunk[..n]);
    }
}

#[test]
fn the_browser_asks_the_same_questions_and_ends_on_the_map() {
    use std::io::BufRead;
    let home = tempfile::tempdir().unwrap();
    let repo = home.path().join("src/api");
    fs::create_dir_all(&repo).unwrap();
    fs::write(repo.join("app.py"), "print('hi')\n").unwrap();
    for args in [
        &["init", "-q"][..],
        &["add", "app.py"],
        &[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-qm",
            "init",
        ],
    ] {
        assert!(std::process::Command::new("git")
            .args(args)
            .current_dir(&repo)
            .status()
            .unwrap()
            .success());
    }
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_crosscut"))
        .arg("--browser")
        .env("CROSSCUT_HARNESS", UNPACKING_AGENT)
        .env("PROMPTS", home.path().join("prompts"))
        .env("HOME", home.path())
        .current_dir(home.path())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut lines = std::io::BufReader::new(child.stderr.take().unwrap()).lines();
    let url = loop {
        let line = lines.next().expect("a URL before exiting").unwrap();
        if let Some(i) = line.find("http://127.0.0.1:") {
            break line[i..].trim().to_string();
        }
    };
    std::thread::spawn(move || for _ in lines {});

    let mut kinds = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(30);
    let done = loop {
        assert!(Instant::now() < deadline, "asked: {kinds:?}");
        let state: serde_json::Value =
            serde_json::from_str(&http(&url, "GET", "/state", "")).unwrap();
        if let Some(done) = state["done"].as_object().filter(|_| state["ask"].is_null()) {
            break done.clone();
        }
        let ask = &state["ask"];
        if ask.is_null() || kinds.last() == Some(&ask["id"].to_string()) {
            std::thread::sleep(Duration::from_millis(100));
            continue;
        }
        kinds.push(ask["id"].to_string());
        let value = match ask["kind"].as_str().unwrap() {
            "yesno" => serde_json::json!(true),
            // It has no remote, so it is cloned from its path.
            "repos" => serde_json::json!("1"),
            "workspace" => ask["suggested"].clone(),
            "about" => ask["draft"].clone(),
            "overview" => serde_json::Value::Null,
            "card" => serde_json::json!({"a": "accept"}),
            other => panic!("unexpected question {other}"),
        };
        http(
            &url,
            "POST",
            "/answer",
            &serde_json::json!({"id": ask["id"], "value": value}).to_string(),
        );
    };
    let report = http(&url, "GET", "/report", "");
    let _ = child.kill();
    let _ = child.wait();
    let map = &done["map"];
    assert_eq!(map["projects"], serde_json::json!(["api"]), "{done:?}");
    let slugs: Vec<&str> = map["concerns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["slug"].as_str().unwrap())
        .collect();
    assert_eq!(
        slugs,
        ["as-of", "log-once"],
        "both cards were accepted and reviewed"
    );
    assert_eq!(map["concerns"][0]["cells"][0]["observed"], "partly");
    assert!(report.contains("Each error should be logged once"));
    let workspace = home.path().join("crosscut-workspace");
    assert!(
        workspace.join("api/app.py").is_file(),
        "a fresh clone, beside the original"
    );
}
