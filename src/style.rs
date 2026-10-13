//! Bold, colour and italics for a person at a terminal. Anything else, such as a pipe, a file
//! or a test, gets plain text, and so does anyone who sets NO_COLOR.

use std::io::IsTerminal;
use std::sync::OnceLock;

fn on(stdout: bool) -> bool {
    static ERR: OnceLock<bool> = OnceLock::new();
    static OUT: OnceLock<bool> = OnceLock::new();
    let tty = |is: bool| is && std::env::var_os("NO_COLOR").is_none();
    if stdout {
        *OUT.get_or_init(|| tty(std::io::stdout().is_terminal()))
    } else {
        *ERR.get_or_init(|| tty(std::io::stderr().is_terminal()))
    }
}

fn paint(code: &str, text: &str, stdout: bool) -> String {
    if on(stdout) {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

pub fn bold(text: &str) -> String {
    paint("1", text, false)
}

pub fn dim(text: &str) -> String {
    paint("2", text, false)
}

pub fn italic(text: &str) -> String {
    paint("3", text, false)
}

pub fn accent(text: &str) -> String {
    paint("36", text, false)
}

pub fn good(text: &str) -> String {
    paint("32", text, false)
}

/// A status word on stdout, coloured by what it says. The text is padded by the caller.
pub fn status(text: &str) -> String {
    let code = match text.trim_end().trim_end_matches('*') {
        "yes" => "32",
        "partly" | "missing" => "33",
        _ => "2",
    };
    paint(code, text, true)
}
