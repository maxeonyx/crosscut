//! What a person sees and answers while CrossCut sets up a workspace and unpacks cross-cutting
//! concerns. The flow is written once against `Ui`; a terminal and a browser page each
//! implement it.

use std::io::{BufRead, IsTerminal, Write};
use std::path::Path;
use std::time::Duration;

use crate::build::{Recommendation, Tier};
use crate::style;
use crate::unpack::{Answer, Card};
use crate::workspace::Repo;

/// Something to show, with no answer expected.
pub enum Say<'a> {
    /// The name and tagline.
    Title,
    /// A quiet aside.
    Note(&'a str),
    /// A step starting: a verb in bold, then what it applies to.
    Step(&'a str, &'a str),
    /// What happens next, with the coding agent that will do it.
    Plan(&'a str),
    /// A long wait is starting.
    Wait(&'a str),
    /// What an agent is doing right now.
    Progress(&'a str),
    /// A repository being cloned, or found already there.
    Item(&'a str, &'a str),
    /// The outcome of one card, and its title.
    Answered(&'a Answer, &'a str),
    /// A review of one project for one concern: what its agent is doing, or its status once
    /// it is done.
    Cell {
        concern: &'a str,
        project: &'a str,
        doing: &'a str,
        status: Option<&'a str>,
    },
    /// How the choosing went.
    Summary {
        accepted: usize,
        declined: usize,
        skipped: usize,
        of: usize,
    },
}

pub trait Ui: Sync {
    fn say(&self, say: Say);
    fn yes_no(&self, question: &str) -> bool;
    /// Repositories to pull in, as `(name, source)`. Empty means stop.
    fn choose(&self, repos: &[Repo]) -> Vec<(String, String)>;
    /// The workspace directory, as typed; empty means the suggestion.
    fn workspace(&self, suggested: &str) -> String;
    /// A project's description, after the person has kept or replaced the draft.
    fn about(&self, project: &str, draft: &str) -> String;
    /// Shows every card before going through them.
    fn overview(&self, cards: &[&Card]);
    fn card(&self, card: &Card, n: usize, of: usize) -> Answer;
    /// The reviews are done: the map and report are in `root`.
    fn finished(&self, root: &Path) -> Result<(), String>;
    /// Whether to make the accepted cross-cutting concerns' checks permanent.
    fn permanent(&self) -> bool;
    /// The check to build for each, starting from the recommendation; `None` builds nothing.
    fn tiers(&self, recommended: &[Recommendation]) -> Vec<Option<Tier>>;
}

pub const PERMANENT: &str = "Make these checks permanent?";
pub const PERMANENT_LEAD: &str = "Each cross-cutting concern you accepted gets whichever of these suits it best, so checking it again costs little or nothing:";

/// The terminal: questions on stderr, answers from stdin, one line each.
pub struct Terminal;

fn read_line() -> Option<String> {
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    match std::io::stdin().lock().read_line(&mut line) {
        Ok(0) | Err(_) => None,
        Ok(_) => Some(line),
    }
}

/// One paragraph on one line after `prefix`, for the terminal to wrap at its own width.
fn line(prefix: &str, text: &str) -> String {
    format!(
        "{prefix}{}\n",
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    )
}

/// Which cards an overview lists: all of a short list, or the first three and the last two
/// of a long one, with `None` where the rest are left out.
pub fn overview_rows(n: usize) -> Vec<Option<usize>> {
    if n <= 7 {
        (0..n).map(Some).collect()
    } else {
        [Some(0), Some(1), Some(2), None, Some(n - 2), Some(n - 1)].to_vec()
    }
}

pub fn plan_steps(agent: &str) -> [(&'static str, String, &'static str); 6] {
    [
        (
            "Describe",
            "each repository, from its README".into(),
            "seconds; you can correct it",
        ),
        (
            "Unpack",
            format!("cross-cutting concerns for software like it, with {agent}"),
            "a few minutes; never sees your code",
        ),
        (
            "Sharpen",
            "each one: worth having, general, named, plain".into(),
            "or drop it",
        ),
        (
            "Prioritise",
            "by leverage across your repositories".into(),
            "it can only reorder them",
        ),
        ("Choose", "accept or decline each one".into(), ""),
        ("Review", "your code, for the ones you accept".into(), ""),
    ]
}

/// How long a person needs to read a line before the next one arrives. The page keeps the
/// same pace.
fn dwell(say: &Say) -> Duration {
    Duration::from_millis(match say {
        Say::Plan(_) => 2500,
        Say::Note(_) | Say::Wait(_) | Say::Summary { .. } => 1200,
        Say::Step(..) => 900,
        Say::Item(..) => 500,
        Say::Answered(..) => 300,
        Say::Title | Say::Progress(_) | Say::Cell { .. } => 0,
    })
}

impl Ui for Terminal {
    fn say(&self, say: Say) {
        let pause = dwell(&say);
        match say {
            Say::Title => eprintln!(
                "{}  {}\n",
                style::bold("CrossCut"),
                style::dim("cross-cutting concerns, kept visible across your repositories")
            ),
            Say::Note(text) => eprintln!("\n{}", style::dim(text)),
            Say::Step(verb, rest) => eprintln!("\n{} {rest}", style::bold(verb)),
            Say::Plan(agent) => {
                eprintln!("{}\n", style::bold("What happens next"));
                for (i, (verb, what, note)) in plan_steps(agent).iter().enumerate() {
                    eprintln!(
                        "  {}  {}  {}  {}",
                        style::dim(&(i + 1).to_string()),
                        style::bold(&format!("{verb:<10}")),
                        what,
                        style::dim(note)
                    );
                }
            }
            Say::Wait(text) => eprintln!("{}", style::italic(text)),
            Say::Progress(text) => eprintln!("{}", style::dim(&format!("  {text}"))),
            Say::Item(name, detail) => eprintln!("  {name}  {}", style::dim(detail)),
            // The terminal prints each review's outcome as it is recorded.
            Say::Cell { .. } => {}
            Say::Answered(answer, _) => match answer {
                Answer::Accept => eprintln!("  {}", style::good("✓ accepted")),
                Answer::Decline => eprintln!("  {}", style::dim("✗ declined")),
                Answer::Skip | Answer::Quit => {}
            },
            Say::Summary {
                accepted,
                declined,
                skipped,
                of,
            } => eprintln!(
                "\n\n{} {}",
                style::bold(&format!("{accepted} accepted")),
                style::dim(&format!("{declined} declined, {skipped} skipped, of {of}"))
            ),
        }
        if std::io::stderr().is_terminal() {
            std::thread::sleep(pause);
        }
    }

    fn yes_no(&self, question: &str) -> bool {
        eprint!("{} {} > ", style::bold(question), style::dim("[Y/n]"));
        read_line().is_some_and(|l| !l.trim().to_lowercase().starts_with('n'))
    }

    fn choose(&self, repos: &[Repo]) -> Vec<(String, String)> {
        // The list may scroll off the top; number 1, the most recently used, sits nearest the
        // prompt.
        if !repos.is_empty() {
            eprintln!();
        }
        for (i, repo) in repos.iter().enumerate().rev() {
            eprintln!(
                "  {}  {}  {}",
                style::dim(&format!("{:>3}", i + 1)),
                repo.name,
                style::dim(&repo.where_())
            );
        }
        loop {
            eprint!(
                "\n{} {} > ",
                style::bold("Pull in which?"),
                style::dim(if repos.is_empty() {
                    "owner/name"
                } else {
                    "numbers (1 is the most recent), or owner/name"
                })
            );
            let Some(line) = read_line() else {
                return Vec::new();
            };
            match crate::workspace::parse_choice(&line, repos) {
                Some(picked) => return picked,
                None => eprintln!(
                    "  Numbers from 1 to {}, or owner/name, please.",
                    repos.len()
                ),
            }
        }
    }

    fn workspace(&self, suggested: &str) -> String {
        eprint!(
            "\n{} {}\n{} {} > ",
            style::dim("Fresh clones go in a workspace."),
            style::dim("Your own checkouts are never touched."),
            style::bold("Workspace"),
            style::dim(&format!("[{suggested}]"))
        );
        read_line().unwrap_or_default().trim().to_string()
    }

    fn about(&self, project: &str, draft: &str) -> String {
        eprint!(
            "\n{}",
            line(
                &format!("  {}  ", style::bold(project)),
                &style::italic(draft.trim())
            )
        );
        eprint!(
            "  {} > ",
            style::dim("Enter to keep it, or type a better description")
        );
        match read_line().unwrap_or_default().trim() {
            "" => draft.trim().to_string(),
            better => better.to_string(),
        }
    }

    fn overview(&self, cards: &[&Card]) {
        eprintln!(
            "\n\n{} {}\n",
            style::bold(&match cards.len() {
                1 => "1 cross-cutting concern".to_string(),
                n => format!("{n} cross-cutting concerns"),
            }),
            style::dim("highest leverage first")
        );
        for row in overview_rows(cards.len()) {
            match row {
                Some(i) => eprintln!(
                    "  {}  {}  {}",
                    style::dim(&format!("{:>2}", i + 1)),
                    cards[i].title,
                    style::dim(&cards[i].projects().join(" · "))
                ),
                None => eprintln!(
                    "  {}",
                    style::dim(&format!("    … {} more …", cards.len() - 5))
                ),
            }
        }
        eprint!(
            "\n{} > ",
            style::dim("Enter to go through them one at a time")
        );
        read_line();
    }

    fn card(&self, card: &Card, n: usize, of: usize) -> Answer {
        let mut out = format!(
            "\n\n{}  {}\n{}\n\n",
            style::dim(&format!("{n}/{of}")),
            style::bold(&card.title),
            style::dim(&format!("      {}", card.projects().join(" · ")))
        );
        out += &line("  ", &card.justification);
        out += &format!("\n  {}\n", style::bold("Stakeholder stories"));
        for story in &card.stories {
            out += &line(&format!("  {} ", style::dim("•")), story);
        }
        out += &format!("\n  {}\n", style::bold("Here"));
        for here in &card.here {
            out += &line(
                &format!("  {}  ", style::italic(&here.project)),
                &here.story,
            );
        }
        eprint!("{out}");
        let key = |k: &str, rest: &str| format!("{}{}", style::accent(&style::bold(k)), rest);
        loop {
            eprint!(
                "\n  {}   {}   {}   {} > ",
                key("a", "ccept"),
                key("d", "ecline"),
                key("s", "kip"),
                key("q", "uit")
            );
            let Some(line) = read_line() else {
                return Answer::Quit;
            };
            match line.trim().to_lowercase().as_str() {
                "a" | "accept" => return Answer::Accept,
                "s" | "skip" => return Answer::Skip,
                "q" | "quit" => return Answer::Quit,
                "d" | "decline" => return Answer::Decline,
                _ => {}
            }
        }
    }

    fn permanent(&self) -> bool {
        eprintln!(
            "\n\n{}\n{}\n",
            style::bold(PERMANENT),
            style::dim(PERMANENT_LEAD)
        );
        for tier in Tier::ALL {
            eprintln!(
                "  {}  {}",
                style::bold(tier.label()),
                style::dim(tier.meaning())
            );
        }
        eprint!("\n{} > ", style::dim("[Y/n]"));
        read_line().is_some_and(|l| !l.trim().to_lowercase().starts_with('n'))
    }

    fn tiers(&self, recommended: &[Recommendation]) -> Vec<Option<Tier>> {
        let mut chosen: Vec<Option<Tier>> = recommended.iter().map(|r| Some(r.tier)).collect();
        loop {
            eprintln!();
            for (i, (r, tier)) in recommended.iter().zip(&chosen).enumerate() {
                eprintln!(
                    "  {}  {}",
                    style::dim(&(i + 1).to_string()),
                    style::bold(&r.title)
                );
                match tier {
                    Some(t) if *t == r.tier => {
                        eprintln!("     {}  {}", t.label(), style::dim(&r.why))
                    }
                    Some(t) => eprintln!("     {}", t.label()),
                    None => eprintln!("     {}", style::dim("nothing")),
                }
            }
            eprint!(
                "\n{} > ",
                style::dim(
                    "Enter to build these, or change one: 2 tool, 2 custom, 2 prompt, 2 skip"
                )
            );
            let Some(line) = read_line() else {
                return vec![None; recommended.len()];
            };
            let words: Vec<&str> = line.split_whitespace().collect();
            match words[..] {
                [] => return chosen,
                [n, key] => match (n.parse::<usize>(), Tier::parse(key), key) {
                    (Ok(n), tier, _)
                        if (1..=chosen.len()).contains(&n) && (tier.is_some() || key == "skip") =>
                    {
                        chosen[n - 1] = tier
                    }
                    _ => eprintln!(
                        "  A number from 1 to {}, then tool, custom, prompt or skip.",
                        chosen.len()
                    ),
                },
                _ => eprintln!(
                    "  A number from 1 to {}, then tool, custom, prompt or skip.",
                    chosen.len()
                ),
            }
        }
    }

    fn finished(&self, root: &Path) -> Result<(), String> {
        println!();
        crate::map(None, Some(root.to_path_buf()))?;
        println!(
            "\n{}  file://{}",
            style::bold("Report"),
            crate::report::write(root)?.display()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::overview_rows;

    #[test]
    fn a_long_overview_shows_the_first_three_and_the_last_two() {
        assert_eq!(overview_rows(3), [Some(0), Some(1), Some(2)]);
        assert_eq!(overview_rows(7).len(), 7);
        assert_eq!(
            overview_rows(23),
            [Some(0), Some(1), Some(2), None, Some(21), Some(22)]
        );
    }
}
