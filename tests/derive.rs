//! End-to-end tests for the `#[derive(Validated)]` macro.
//!
//! Compiled only when the `derive` feature is enabled.

#![cfg(feature = "derive")]

use type_lib::combinator::And;
use type_lib::rules::{InRange, LenRange, Trimmed};
use type_lib::{Validated, ValidationError};

#[derive(Validated, Debug, Clone, PartialEq)]
#[valid(And<Trimmed, LenRange<3, 16>>)]
struct Username(String);

#[derive(Validated, Debug, PartialEq)]
#[valid(InRange<0, 100>)]
struct Percent(i32);

#[test]
fn accepts_valid_value() {
    let user = Username::new("alice".to_owned()).expect("valid username");
    assert_eq!(user.get(), "alice");
    assert_eq!(&*user, "alice"); // Deref
    assert_eq!(user.into_inner(), "alice");
}

#[test]
fn rejects_invalid_value_with_validator_error() {
    // Too short, and not trimmed.
    let err: ValidationError = Username::new("  x  ".to_owned()).unwrap_err();
    assert!(err.code() == "trimmed" || err.code() == "len_range");

    let err = Percent::new(150).unwrap_err();
    assert_eq!(err.code(), "in_range");
}

#[test]
fn numeric_newtype_round_trips() {
    let p = Percent::new(42).expect("in range");
    assert_eq!(*p, 42);
    assert_eq!(p, Percent::new(42).expect("in range"));
}

#[test]
fn standard_derives_coexist() {
    // `Clone`/`PartialEq`/`Debug` derived alongside `Validated` work as usual.
    let a = Username::new("bob".to_owned()).expect("valid");
    let b = a.clone();
    assert_eq!(a, b);
    assert_eq!(format!("{b:?}"), "Username(\"bob\")");
}
