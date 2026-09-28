# Version visibility

Anyone who needs to know what is live can find out, at the moment they need it.

## User stories

- **The on-call engineer, mid-incident:** "is the fix live?" answered in seconds.

## What it looks like here

- Rust CLIs: `--version`, from clap's `#[command(version)]`.
- Sites: a `version.json` next to `index.html`, naming the commit.
- Scheduled jobs: a version line in their log.

## Decisions

- **nightly-sync**: deferred: being retired once the ledger's own backups land (Juniper lead, 2026-09-01).
