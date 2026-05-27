//! Built-in validation rules.
//!
//! These are ready-to-use [`Validator`](crate::Validator) implementations for the
//! most common invariants, grouped by what they constrain:
//!
//! - [length] — `NonEmpty`, `MinLen`, `MaxLen`, `LenRange`
//! - [numeric] — `Positive`, `NonNegative`, `Negative`, `NonPositive`, `InRange`
//! - [string] — `Ascii`, `Alphanumeric`, `Trimmed`
//!
//! Every built-in rule reports failures as [`ValidationError`](crate::ValidationError),
//! so they share one error type and compose freely with the
//! [combinators](crate::combinator).
//!
//! # Examples
//!
//! ```rust
//! use type_lib::combinator::And;
//! use type_lib::rules::{LenRange, Trimmed};
//! use type_lib::Refined;
//!
//! // A display name: 1–32 characters, no surrounding whitespace.
//! type DisplayName<'a> = Refined<&'a str, And<Trimmed, LenRange<1, 32>>>;
//!
//! assert!(DisplayName::new("Ada Lovelace").is_ok());
//! assert!(DisplayName::new("  spaced  ").is_err());
//! ```

pub mod length;
pub mod numeric;
pub mod string;

pub use length::{HasLength, LenRange, MaxLen, MinLen, NonEmpty};
pub use numeric::{InRange, Negative, NonNegative, NonPositive, Positive};
pub use string::{Alphanumeric, Ascii, Trimmed};
