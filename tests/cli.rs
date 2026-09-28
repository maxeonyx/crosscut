//! Black-box tests: spawn the binary against temporary concern directories.
//! Harness runs use small shell commands in place of a real coding agent.

use std::fs;
use std::path::Path;

use assert_cmd::Command;

const CARRY_FORWARD: &str =
    "must carry both the doctrine and this requirement to carry both forward again";

/// Collapse whitespace so prose checks do not depend on line wrapping.
fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn crosscut() -> Command {
    let mut cmd = Command::cargo_bin("crosscut").unwrap();
    cmd.env_remove("CROSSCUT_HARNESS");
    cmd
}

fn stdout(cmd: &mut Command) -> String {
    let out = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(out).unwrap()
}

const RECOVERY: &str = "# Can we get back the photos after losing the server?\n\n\
## Why this matters here\n\nFamily photos exist only on the home server.\n\n\
## How to look\n\n- `restic snapshots`\n\n\
## Current view — 2026-01-01\n\nUnknown: backups were never checked.\n";

const VERSION: &str =
    "# Can we tell what is deployed?\n\n## Why this matters here\n\nManual deploys.\n";

fn ecosystem() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let concerns = dir.path().join("crosscut/concerns");
    fs::create_dir_all(&concerns).unwrap();
    fs::write(concerns.join("recovery.md"), RECOVERY).unwrap();
    fs::write(concerns.join("version.md"), VERSION).unwrap();
    dir
}

fn read(root: &Path, slug: &str) -> String {
    fs::read_to_string(root.join(format!("crosscut/concerns/{slug}.md"))).unwrap()
}

#[test]
fn bare_invocation_orients_an_agent_with_the_doctrine() {
    let out = stdout(&mut crosscut());
    assert!(out.contains("## Doctrine"), "{out}");
    assert!(flat(&out).contains(CARRY_FORWARD), "{out}");
    assert!(out.contains("crosscut prompt setup"), "{out}");
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
        let out = stdout(crosscut().args(["prompt", mode]));
        assert!(
            flat(&out).contains(CARRY_FORWARD),
            "{mode} prompt lost the carry-forward requirement"
        );
        assert!(out.contains(&format!("<!-- modes/{mode}.md -->")), "{mode}");
        assert!(!out.starts_with("---"), "{mode} prompt leaked frontmatter");
    }
    assert!(stdout(crosscut().args(["prompt", "setup"])).contains("<!-- reservoirs.md -->"));
    assert!(stdout(crosscut().args(["prompt", "refresh"])).contains("<!-- concern-files.md -->"));
}

#[test]
fn unknown_mode_names_the_real_ones() {
    let out = crosscut()
        .args(["prompt", "audit"])
        .assert()
        .code(2)
        .get_output()
        .stderr
        .clone();
    let err = String::from_utf8(out).unwrap();
    assert!(
        err.contains("setup, discover, establish, refresh, reconsider, generalize"),
        "{err}"
    );
}

#[test]
fn readme_template_carries_the_framing_and_the_requirement_to_carry_it() {
    let out = stdout(crosscut().args(["prompt", "setup"]));
    assert!(out.contains("not obligations"), "{out}");
    assert!(flat(&out)
        .contains("must also pass on the requirement to keep both the thinking and framing and this requirement"));
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
        "reservoirs.md",
        "modes/setup.md",
        "modes/generalize.md",
    ] {
        assert!(target.join(file).is_file(), "{file} missing");
    }
    assert!(fs::read_to_string(target.join("SKILL.md"))
        .unwrap()
        .starts_with("---\nname: crosscut\n"));
}

#[test]
fn list_shows_questions_and_view_dates_from_a_subdirectory() {
    let eco = ecosystem();
    let sub = eco.path().join("crosscut/concerns");
    let out = stdout(crosscut().arg("list").current_dir(&sub));
    assert!(
        out.contains("recovery  Can we get back the photos after losing the server?"),
        "{out}"
    );
    assert!(
        out.contains("2026-01-01: Unknown: backups were never checked."),
        "{out}"
    );
    assert!(
        out.contains("version   Can we tell what is deployed?"),
        "{out}"
    );
    assert!(out.contains("no view yet"), "{out}");
}

#[test]
fn list_outside_any_crosscut_directory_says_how_to_start() {
    let dir = tempfile::tempdir().unwrap();
    let out = crosscut()
        .arg("list")
        .current_dir(dir.path())
        .assert()
        .code(2)
        .get_output()
        .stderr
        .clone();
    assert!(String::from_utf8(out)
        .unwrap()
        .contains("crosscut prompt setup"));
}

#[test]
fn refresh_replaces_only_the_view_and_keeps_the_definition() {
    let eco = ecosystem();
    let harness = "cat > prompt.txt; printf 'thinking...\\n<crosscut-view>\\n## Current view — 2026-09-28\\n\\nApplies strongly. Last snapshot 3 days old.\\n</crosscut-view>\\n'";
    let out = stdout(
        crosscut()
            .args(["refresh", "recovery", "--harness", harness])
            .current_dir(eco.path()),
    );
    assert!(out.contains("recovery: view updated"), "{out}");

    let text = read(eco.path(), "recovery");
    assert!(
        text.starts_with(RECOVERY.split("## Current view").next().unwrap().trim_end()),
        "{text}"
    );
    assert!(text.contains("Last snapshot 3 days old."), "{text}");
    assert!(!text.contains("never checked"), "old view kept: {text}");
    assert_eq!(text.matches("## Current view").count(), 1, "{text}");
    assert_eq!(
        read(eco.path(), "version"),
        VERSION,
        "unselected concern changed"
    );

    let prompt = fs::read_to_string(eco.path().join("prompt.txt")).unwrap();
    assert!(
        flat(&prompt).contains(CARRY_FORWARD),
        "headless prompt lost the doctrine"
    );
    assert!(
        prompt.contains("Family photos exist only on the home server."),
        "concern text missing"
    );
    assert!(prompt.contains("Do not modify any files."));
}

#[test]
fn refresh_adds_a_view_to_a_concern_that_has_none() {
    let eco = ecosystem();
    let harness = "cat >/dev/null; echo '<crosscut-view>'; echo 'Unknown: no deploy access.'; echo '</crosscut-view>'";
    crosscut()
        .args(["refresh", "version", "--harness", harness])
        .current_dir(eco.path())
        .assert()
        .success();
    let text = read(eco.path(), "version");
    assert!(text.starts_with(VERSION.trim_end()), "{text}");
    assert!(
        text.contains("\n\n## Current view — 20"),
        "heading not added: {text}"
    );
    assert!(text.ends_with("Unknown: no deploy access.\n"), "{text}");
}

#[test]
fn one_concern_failing_to_run_does_not_stop_the_others() {
    let eco = ecosystem();
    // Fails for the recovery prompt, succeeds for the version prompt.
    let harness = "if grep -q 'Family photos exist only'; then echo 'not logged in' >&2; exit 3; fi; \
                   echo '<crosscut-view>'; echo '## Current view — x'; echo; echo 'ok'; echo '</crosscut-view>'";
    let assert = crosscut()
        .args(["refresh", "--harness", harness])
        .current_dir(eco.path())
        .assert()
        .code(1);
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(err.contains("recovery: harness exited"), "{err}");
    assert!(err.contains("not logged in"), "{err}");
    assert_eq!(
        read(eco.path(), "recovery"),
        RECOVERY,
        "failed refresh touched the file"
    );
    assert!(read(eco.path(), "version").ends_with("## Current view — x\n\nok\n"));
}

#[test]
fn a_harness_that_exits_zero_without_a_view_leaves_the_file_alone_and_says_why() {
    let eco = ecosystem();
    let assert = crosscut()
        .args([
            "refresh",
            "recovery",
            "--harness",
            "cat >/dev/null; printf '\\033[91mError: \\033[0mModel not found\\n' >&2",
        ])
        .current_dir(eco.path())
        .assert()
        .code(2);
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(err.contains("left unchanged"), "{err}");
    assert!(
        err.contains("Error: Model not found"),
        "harness stderr hidden or garbled: {err}"
    );
    assert_eq!(read(eco.path(), "recovery"), RECOVERY);
}

#[test]
fn refresh_without_any_harness_explains_what_to_install() {
    let eco = ecosystem();
    let assert = crosscut()
        .arg("refresh")
        .env("PATH", "")
        .current_dir(eco.path())
        .assert()
        .code(2);
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(err.contains("no coding harness found"), "{err}");
}

#[test]
fn unknown_slug_is_rejected_before_anything_runs() {
    let eco = ecosystem();
    crosscut()
        .args(["refresh", "nope", "--harness", "touch ran"])
        .current_dir(eco.path())
        .assert()
        .code(2);
    assert!(!eco.path().join("ran").exists());
}

#[test]
fn dry_run_shows_the_command_and_prompt_without_running() {
    let eco = ecosystem();
    let out = stdout(
        crosscut()
            .args(["refresh", "--dry-run", "--harness", "codex"])
            .current_dir(eco.path()),
    );
    assert!(out.contains("codex exec --sandbox read-only"), "{out}");
    assert!(out.contains("===== prompt for recovery ====="), "{out}");
    assert!(out.contains("===== prompt for version ====="), "{out}");
    assert_eq!(read(eco.path(), "recovery"), RECOVERY);
}

#[test]
fn a_hung_harness_is_stopped_at_the_timeout_and_the_file_kept() {
    let eco = ecosystem();
    let assert = crosscut()
        .args([
            "refresh",
            "recovery",
            "--timeout",
            "0",
            "--harness",
            "sleep 30",
        ])
        .current_dir(eco.path())
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .code(2);
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(err.contains("was stopped"), "{err}");
    assert_eq!(read(eco.path(), "recovery"), RECOVERY);
}

#[test]
fn a_view_heading_inside_a_code_fence_is_not_the_view() {
    let eco = ecosystem();
    let fenced =
        format!("{VERSION}\n## How to look\n\n```markdown\n## Current view — example\n```\n");
    fs::write(eco.path().join("crosscut/concerns/version.md"), &fenced).unwrap();
    let out = stdout(crosscut().arg("list").current_dir(eco.path()));
    assert!(out.contains("no view yet"), "{out}");
    let harness = "cat >/dev/null; echo '<crosscut-view>'; echo real; echo '</crosscut-view>'";
    crosscut()
        .args(["refresh", "version", "--harness", harness])
        .current_dir(eco.path())
        .assert()
        .success();
    let text = read(eco.path(), "version");
    assert!(
        text.starts_with(fenced.trim_end()),
        "fenced example was cut: {text}"
    );
    assert!(text.ends_with("real\n"), "{text}");
}

#[test]
fn model_is_passed_to_known_harnesses_and_refused_for_custom_commands() {
    let eco = ecosystem();
    let out = stdout(
        crosscut()
            .args([
                "refresh",
                "--dry-run",
                "--harness",
                "claude",
                "--model",
                "haiku",
            ])
            .current_dir(eco.path()),
    );
    assert!(out.contains("--model haiku"), "{out}");
    assert!(
        out.contains("--setting-sources user"),
        "target project settings would load: {out}"
    );
    assert!(
        out.contains(
            "--disallowedTools Edit,Write,NotebookEdit,Agent,Workflow --strict-mcp-config"
        ),
        "{out}"
    );
    let out = stdout(
        crosscut()
            .args([
                "refresh",
                "--dry-run",
                "--harness",
                "opencode",
                "--model",
                "x/y",
            ])
            .current_dir(eco.path()),
    );
    assert!(out.contains("opencode -m x/y run <prompt>"), "{out}");
    crosscut()
        .args(["refresh", "--harness", "cat", "--model", "haiku"])
        .current_dir(eco.path())
        .assert()
        .code(2);
}

#[test]
fn concerns_are_refreshed_concurrently_up_to_jobs() {
    let eco = ecosystem();
    for slug in ["a", "b", "c", "d"] {
        fs::write(
            eco.path().join(format!("crosscut/concerns/{slug}.md")),
            format!("# {slug}?\n"),
        )
        .unwrap();
    }
    let harness =
        "cat >/dev/null; sleep 1; echo '<crosscut-view>'; echo ok; echo '</crosscut-view>'";
    let started = std::time::Instant::now();
    crosscut()
        .args(["refresh", "--jobs", "6", "--harness", harness])
        .current_dir(eco.path())
        .assert()
        .success();
    let elapsed = started.elapsed();
    assert!(
        elapsed.as_secs_f64() < 3.0,
        "six concerns with six workers took {elapsed:?}"
    );
    for slug in ["a", "b", "c", "d", "recovery", "version"] {
        assert!(
            read(eco.path(), slug).ends_with("ok\n"),
            "{slug} not refreshed"
        );
    }

    let started = std::time::Instant::now();
    crosscut()
        .args([
            "refresh",
            "a",
            "b",
            "c",
            "--jobs",
            "2",
            "--harness",
            harness,
        ])
        .current_dir(eco.path())
        .assert()
        .success();
    let elapsed = started.elapsed().as_secs_f64();
    assert!(
        (2.0..3.5).contains(&elapsed),
        "three concerns with two workers should take two rounds, took {elapsed}s"
    );
}

#[test]
fn readme_and_site_carry_the_requirement_to_carry_the_doctrine() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = flat(&fs::read_to_string(root.join("README.md")).unwrap());
    assert!(
        readme
            .contains("must pass on the requirement to keep both the framing and this requirement"),
        "README"
    );
    let site = flat(&fs::read_to_string(root.join("docs/index.html")).unwrap());
    assert!(
        site.contains("the requirement to pass on both the doctrine and this requirement"),
        "site"
    );
}
