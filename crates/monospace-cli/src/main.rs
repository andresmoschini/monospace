//! Minimal non-interactive command-line front end for Monospace.
//!
//! It holds no domain logic of its own: everything it draws comes from `monospace-core`.

mod description;

use description::Description;

/// The shipped demonstration description, embedded at compile time so the no-argument run works
/// from any working directory and from a binary copied outside a checkout (FR-022, FR-023).
const DEMO: &str = include_str!("../assets/demo.json");

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let text = match args.as_slice() {
        [path] => std::fs::read_to_string(path).expect("path should be readable"),
        _ => DEMO.to_owned(),
    };

    let description: Description =
        serde_json::from_str(&text).expect("file should hold a well-formed description");
    print!("{}", description.render());
}
