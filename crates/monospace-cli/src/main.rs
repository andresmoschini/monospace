//! Minimal non-interactive command-line front end for Monospace.
//!
//! It holds no domain logic of its own: everything it draws comes from `monospace-core`.

mod description;

use std::process::ExitCode;

use description::Description;

/// The shipped demonstration description, embedded at compile time so the no-argument run works
/// from any working directory and from a binary copied outside a checkout (FR-022, FR-023).
const DEMO: &str = include_str!("../assets/demo.json");

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let text = match args.as_slice() {
        [] => DEMO.to_owned(),
        [path] => match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                eprintln!("{path}: {error}");
                return ExitCode::FAILURE;
            }
        },
        _ => {
            eprintln!("usage: monospace-cli [path]");
            return ExitCode::FAILURE;
        }
    };

    let description = match serde_json::from_str::<Description>(&text) {
        Ok(description) => description,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    print!("{}", description.render());
    ExitCode::SUCCESS
}
