//! Composing rules with the `And`, `Or`, and `Not` combinators.
//!
//! Run with: `cargo run --example composing_rules`

use type_lib::combinator::{And, Not, Or};
use type_lib::rules::{Alphanumeric, Ascii, LenRange, NonEmpty, Trimmed};
use type_lib::Refined;

// A slug: 1–32 chars, alphanumeric, no surrounding whitespace.
type Slug = Refined<String, And<And<NonEmpty, Alphanumeric>, And<Trimmed, LenRange<1, 32>>>>;

// A field that must be non-empty and either alphanumeric or ASCII.
type FlexibleId = Refined<String, And<NonEmpty, Or<Alphanumeric, Ascii>>>;

// A value required to contain at least one non-ASCII character.
type HasUnicode = Refined<String, Not<Ascii>>;

fn main() {
    for candidate in ["release2026", "with space", ""] {
        let verdict = Slug::new(candidate.to_owned())
            .map(|_| "ok")
            .unwrap_or("rejected");
        println!("Slug {candidate:?} -> {verdict}");
    }

    println!();
    for candidate in ["abc123", "a-b_c.d", ""] {
        let verdict = FlexibleId::new(candidate.to_owned())
            .map(|_| "ok")
            .unwrap_or("rejected");
        println!("FlexibleId {candidate:?} -> {verdict}");
    }

    println!();
    for candidate in ["café", "plain"] {
        let verdict = HasUnicode::new(candidate.to_owned())
            .map(|_| "ok")
            .unwrap_or("rejected");
        println!("HasUnicode {candidate:?} -> {verdict}");
    }
}
