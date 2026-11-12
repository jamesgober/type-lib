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

#[test]
fn try_from_runs_the_validator() {
    let ok = Percent::try_from(42).expect("in range");
    assert_eq!(*ok, 42);
    let err = Percent::try_from(150).unwrap_err();
    assert_eq!(err.code(), "in_range");

    let user: Result<Username, _> = "  padded  ".to_owned().try_into();
    assert!(user.is_err());
}

/// A wire type that validates on deserialize through the generated `TryFrom`.
#[derive(Validated, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
#[valid(And<Trimmed, LenRange<3, 16>>)]
struct WireName(String);

impl From<WireName> for String {
    fn from(name: WireName) -> String {
        name.into_inner()
    }
}

#[test]
fn serde_try_from_validates_on_deserialize() {
    let good: WireName = serde_json::from_str("\"alice\"").expect("valid name");
    assert_eq!(good.get(), "alice");
    assert_eq!(
        serde_json::to_string(&good).expect("serialize"),
        "\"alice\""
    );

    let err = serde_json::from_str::<WireName>("\"   padded and far too long   \"").unwrap_err();
    assert!(err.to_string().contains("trimmed"), "{err}");
    assert!(serde_json::from_str::<WireName>("\"ab\"").is_err());
}

mod renamed {
    // The same crate reached through another path, as a suite re-export would.
    pub mod tl {
        pub use type_lib::*;
    }
}

#[derive(Validated)]
#[valid(crate = renamed::tl, InRange<1, 65535>)]
struct Port(u16);

#[test]
fn crate_option_sets_the_type_lib_path() {
    assert!(Port::new(8080).is_ok());
    assert!(Port::new(0).is_err());
}
