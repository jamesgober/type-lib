//! Generating named validated newtypes with `#[derive(Validated)]`.
//!
//! Requires the `derive` feature:
//! `cargo run --example derive_newtype --features derive`

use type_lib::combinator::And;
use type_lib::rules::{Ascii, InRange, LenRange, Trimmed};
use type_lib::Validated;

/// A username: trimmed, 3–16 characters.
#[derive(Validated, Debug, Clone, PartialEq)]
#[valid(And<Trimmed, LenRange<3, 16>>)]
struct Username(String);

/// An API key: 16–64 ASCII characters.
#[derive(Validated, Debug)]
#[valid(And<Ascii, LenRange<16, 64>>)]
struct ApiKey(String);

/// A percentage: 0–100.
#[derive(Validated, Debug)]
#[valid(InRange<0, 100>)]
struct Percent(i32);

fn main() {
    match Username::new("alice".to_owned()) {
        Ok(user) => println!("username ok: {user:?} (len {})", user.get().len()),
        Err(err) => println!("username rejected: {err}"),
    }

    for candidate in ["ab", "  alice  "] {
        if let Err(err) = Username::new(candidate.to_owned()) {
            println!("username {candidate:?} rejected: {err}");
        }
    }

    match ApiKey::new("sk_live_0123456789abcdef".to_owned()) {
        Ok(key) => println!("api key ok ({} chars)", key.get().len()),
        Err(err) => println!("api key rejected: {err}"),
    }

    for value in [50, 150] {
        match Percent::new(value) {
            Ok(p) => println!("percent ok: {}", p.get()),
            Err(err) => println!("percent {value} rejected: {err}"),
        }
    }
}
