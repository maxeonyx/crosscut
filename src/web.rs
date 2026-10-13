//! The browser: a page served on localhost that shows what the flow says and asks what it
//! asks. The flow runs on this thread and blocks on each question; the server thread hands
//! the page the current state and passes its answers back.
//!
//! With CROSSCUT_RECORD set to a file, every state the page could show is appended to it as
//! one JSON line, so the session can be replayed, as the slides do.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};

use serde_json::{json, Value};

use crate::build::{Recommendation, Tier};
use crate::ui::{plan_steps, Say, Ui};
use crate::unpack::{Answer, Card};
use crate::workspace::{parse_choice, Repo};

const PAGE: &str = include_str!("web.html");

#[derive(Default)]
struct State {
    items: Vec<Value>,
    ask: Option<Value>,
    asked: u64,
    answer: Option<Value>,
    done: Option<Value>,
    report: Option<PathBuf>,
}

impl State {
    fn view(&self) -> Value {
        json!({ "items": self.items, "ask": self.ask, "done": self.done })
    }
}

struct Shared {
    state: Mutex<State>,
    answered: Condvar,
    record: Option<Mutex<std::fs::File>>,
}

impl Shared {
    fn change(&self, f: impl FnOnce(&mut State)) {
        let mut state = self.state.lock().expect("no panics holding the lock");
        f(&mut state);
        if let Some(file) = &self.record {
            let line = format!("{}\n", state.view());
            let _ = file.lock().expect("unpoisoned").write_all(line.as_bytes());
        }
    }
}

pub struct Browser {
    shared: Arc<Shared>,
    pub url: String,
}

impl Browser {
    /// Starts serving on a free localhost port.
    pub fn start() -> Result<Browser, String> {
        let server = tiny_http::Server::http("127.0.0.1:0")
            .map_err(|e| format!("cannot serve the page: {e}"))?;
        let url = format!("http://{}/", server.server_addr());
        let record = std::env::var_os("CROSSCUT_RECORD")
            .map(|path| {
                std::fs::File::create(&path).map(Mutex::new).map_err(|e| {
                    format!("cannot record to {}: {e}", PathBuf::from(&path).display())
                })
            })
            .transpose()?;
        let shared = Arc::new(Shared {
            state: Mutex::new(State::default()),
            answered: Condvar::new(),
            record,
        });
        let serving = Arc::clone(&shared);
        std::thread::spawn(move || {
            for request in server.incoming_requests() {
                serve(&serving, request);
            }
        });
        Ok(Browser { shared, url })
    }

    /// Opens the page in the person's browser, if one can be found. Failing to is fine: the
    /// URL is printed too.
    pub fn open(&self) {
        let browser = std::env::var("BROWSER").ok();
        let candidates = browser
            .iter()
            .map(String::as_str)
            .chain(["wslview", "xdg-open", "open"]);
        for program in candidates {
            let spawned = std::process::Command::new(program)
                .arg(&self.url)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
            if spawned.is_ok() {
                return;
            }
        }
    }

    fn push(&self, item: Value) {
        self.shared.change(|s| s.items.push(item));
    }

    fn ask(&self, mut question: Value) -> Value {
        let mut id = 0;
        self.shared.change(|s| {
            s.asked += 1;
            id = s.asked;
            question["id"] = json!(id);
            s.ask = Some(question);
            s.answer = None;
        });
        let mut state = self.shared.state.lock().expect("unpoisoned");
        while state.answer.is_none() {
            state = self.shared.answered.wait(state).expect("unpoisoned");
        }
        let answer = state.answer.take().expect("checked above");
        drop(state);
        self.shared.change(|s| s.ask = None);
        answer
    }

    /// Shows that the flow ended, whether or not it went well.
    pub fn end(&self, error: Option<&str>) {
        self.shared.change(|s| {
            if let Some(e) = error {
                s.done = Some(json!({ "error": e }));
            } else if s.done.is_none() {
                s.done = Some(json!({}));
            }
        });
    }
}

fn serve(shared: &Shared, mut request: tiny_http::Request) {
    let html = "Content-Type: text/html; charset=utf-8";
    let respond = |request: tiny_http::Request, body: String, kind: &str| {
        let header = kind.parse::<tiny_http::Header>().expect("a valid header");
        let _ = request.respond(tiny_http::Response::from_string(body).with_header(header));
    };
    match (request.method(), request.url()) {
        (tiny_http::Method::Get, "/") => respond(request, PAGE.into(), html),
        (tiny_http::Method::Get, "/state") => {
            let view = shared.state.lock().expect("unpoisoned").view();
            respond(request, view.to_string(), "Content-Type: application/json")
        }
        (tiny_http::Method::Get, "/report") => {
            let path = shared.state.lock().expect("unpoisoned").report.clone();
            let body = path
                .and_then(|p| std::fs::read_to_string(p).ok())
                .unwrap_or_else(|| "No report yet.".into());
            respond(request, body, html)
        }
        (tiny_http::Method::Post, "/answer") => {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            if let Ok(answer) = serde_json::from_str::<Value>(&body) {
                let mut state = shared.state.lock().expect("unpoisoned");
                let current = state.ask.as_ref().map(|a| a["id"].clone());
                if current.is_some() && current == Some(answer["id"].clone()) {
                    state.answer = Some(answer["value"].clone());
                    shared.answered.notify_all();
                }
            }
            respond(request, "{}".into(), "Content-Type: application/json")
        }
        _ => {
            let _ = request.respond(tiny_http::Response::empty(404));
        }
    }
}

fn card_json(card: &Card) -> Value {
    json!({
        "title": card.title,
        "projects": card.projects(),
        "justification": card.justification,
        "stories": card.stories,
        "here": card.here,
    })
}

impl Ui for Browser {
    fn say(&self, say: Say) {
        let item = match say {
            Say::Title => return,
            Say::Note(t) => json!({ "k": "note", "t": t }),
            Say::Step(verb, t) => json!({ "k": "step", "verb": verb, "t": t }),
            Say::Plan(agent) => json!({ "k": "plan", "steps": plan_steps(agent) }),
            Say::Wait(t) => json!({ "k": "wait", "t": t }),
            Say::Progress(t) => json!({ "k": "progress", "t": t.trim() }),
            Say::Item(name, detail) => json!({ "k": "item", "name": name, "t": detail }),
            Say::Cell {
                concern,
                project,
                doing,
                status,
            } => json!({ "k": "cell", "concern": concern, "project": project,
                         "t": doing.trim(), "status": status }),
            Say::Answered(answer, title) => {
                json!({ "k": "answered", "title": title, "a": match answer {
                    Answer::Accept => "accepted",
                    Answer::Decline(_) => "declined",
                    Answer::Skip => "skipped",
                    Answer::Quit => return,
                }})
            }
            Say::Summary {
                accepted,
                declined,
                skipped,
                of,
            } => json!({ "k": "summary", "accepted": accepted, "declined": declined,
                         "skipped": skipped, "of": of }),
        };
        self.push(item);
    }

    fn yes_no(&self, question: &str) -> bool {
        self.ask(json!({ "kind": "yesno", "q": question })) == json!(true)
    }

    fn choose(&self, repos: &[Repo]) -> Vec<(String, String)> {
        let list: Vec<Value> = repos
            .iter()
            .map(|r| json!({ "name": r.name, "where": r.where_() }))
            .collect();
        loop {
            let answer = self.ask(json!({ "kind": "repos", "repos": list }));
            let line = answer.as_str().unwrap_or_default();
            if line.trim().is_empty() {
                return Vec::new();
            }
            if let Some(picked) = parse_choice(line, repos) {
                return picked;
            }
        }
    }

    fn workspace(&self, suggested: &str) -> String {
        let answer = self.ask(json!({ "kind": "workspace", "suggested": suggested }));
        let path = answer.as_str().unwrap_or_default().trim();
        if path == suggested {
            String::new()
        } else {
            path.to_string()
        }
    }

    fn about(&self, project: &str, draft: &str) -> String {
        let answer =
            self.ask(json!({ "kind": "about", "project": project, "draft": draft.trim() }));
        match answer.as_str().map(str::trim) {
            Some(text) if !text.is_empty() => text.to_string(),
            _ => draft.trim().to_string(),
        }
    }

    fn overview(&self, cards: &[&Card]) {
        let list: Vec<Value> = crate::ui::overview_rows(cards.len())
            .into_iter()
            .map(|row| match row {
                Some(i) => {
                    json!({ "n": i + 1, "title": cards[i].title, "projects": cards[i].projects() })
                }
                None => json!({ "more": cards.len() - 5 }),
            })
            .collect();
        self.ask(json!({ "kind": "overview", "count": cards.len(), "cards": list }));
    }

    fn card(&self, card: &Card, n: usize, of: usize) -> Answer {
        let answer = self.ask(json!({ "kind": "card", "n": n, "of": of, "card": card_json(card) }));
        match answer["a"].as_str() {
            Some("accept") => Answer::Accept,
            Some("decline") => {
                Answer::Decline(answer["reason"].as_str().unwrap_or_default().trim().into())
            }
            Some("skip") => Answer::Skip,
            _ => Answer::Quit,
        }
    }

    fn finished(&self, root: &Path) -> Result<(), String> {
        let report = crate::report::write(root)?;
        let data = crate::report::data(root)?;
        self.shared.change(|s| {
            s.items.push(json!({ "k": "map", "map": data }));
            s.report = Some(report);
        });
        Ok(())
    }

    fn permanent(&self) -> bool {
        let options: Vec<Value> = Tier::ALL
            .iter()
            .map(|t| json!([t.label(), t.meaning()]))
            .collect();
        self.ask(json!({ "kind": "permanent", "q": crate::ui::PERMANENT,
                         "lead": crate::ui::PERMANENT_LEAD, "options": options }))
            == json!(true)
    }

    fn tiers(&self, recommended: &[Recommendation]) -> Vec<Option<Tier>> {
        let recs: Vec<Value> = recommended
            .iter()
            .map(|r| json!({ "title": r.title, "tier": r.tier.key(), "why": r.why }))
            .collect();
        let options: Vec<Value> = Tier::ALL
            .iter()
            .map(|t| json!([t.key(), t.label()]))
            .collect();
        let answer = self.ask(json!({ "kind": "tiers", "recs": recs, "options": options }));
        recommended
            .iter()
            .enumerate()
            .map(|(i, r)| match answer.get(i).and_then(Value::as_str) {
                Some(key) => Tier::parse(key),
                None => Some(r.tier),
            })
            .collect()
    }
}
