//! Common imports for working with `type-lib`.
//!
//! Glob-import this module to bring the foundation types and the composition
//! combinators into scope in one line. The built-in [rules](crate::rules) are
//! intentionally left out to keep the namespace small; import the ones you need
//! from `type_lib::rules`.
//!
//! # Examples
//!
//! ```rust
//! use type_lib::prelude::*;
//! use type_lib::rules::{Ascii, NonEmpty};
//!
//! // `Refined`, `Validator`, `ValidationError`, and `And`/`Or`/`Not` are all in
//! // scope from the prelude.
//! type Token<'a> = Refined<&'a str, And<NonEmpty, Ascii>>;
//!
//! assert!(Token::new("abc123").is_ok());
//! assert!(Token::new("").is_err());
//! ```

pub use crate::combinator::{And, Not, Or};
pub use crate::error::ValidationError;
pub use crate::refined::Refined;
pub use crate::validator::Validator;
