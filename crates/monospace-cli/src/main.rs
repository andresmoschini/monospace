//! Minimal non-interactive command-line front end for Monospace.
//!
//! It holds no domain logic of its own: everything it prints comes from `monospace-core`.

fn main() {
    println!("{}", monospace_core::greeting());
}
