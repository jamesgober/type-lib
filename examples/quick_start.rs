//! The shortest end-to-end use: a domain type that cannot hold an invalid value.
//!
//! Run with: `cargo run --example quick_start`

use type_lib::combinator::And;
use type_lib::rules::{LenRange, Trimmed};
use type_lib::Refined;

/// A username: 3–16 characters, no leading or trailing whitespace.
type Username = Refined<String, And<Trimmed, LenRange<3, 16>>>;

fn main() {
    match Username::new("alice".to_owned()) {
        Ok(user) => println!("accepted: {user}"),
        Err(err) => println!("rejected: {err}"),
    }

    // Each of these violates the rule and is rejected at construction.
    for candidate in ["ab", "  spaced  ", "this-name-is-far-too-long"] {
        if let Err(err) = Username::new(candidate.to_owned()) {
            println!("rejected {candidate:?}: {err}");
        }
    }
}
