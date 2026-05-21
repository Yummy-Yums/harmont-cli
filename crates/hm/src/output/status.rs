#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "this module is the centralised print sink for the CLI"
)]

use owo_colors::OwoColorize;

/// Print a success message.
pub fn print_success(msg: &str) {
    let check = format!("{}", "\u{2714}".green().bold());
    println!("{check} {msg}");
}

/// Print a warning message.
pub fn print_warning(msg: &str) {
    let bang = format!("{}", "!".yellow().bold());
    eprintln!("{bang} {msg}");
}

/// Print an error message.
pub fn print_error(msg: &str) {
    let cross = format!("{}", "\u{2718}".red().bold());
    eprintln!("{cross} {msg}");
}

/// Print an info message.
pub fn print_info(msg: &str) {
    let arrow = format!("{}", "\u{25b6}".cyan());
    println!("{arrow} {msg}");
}
