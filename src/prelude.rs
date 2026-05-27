//! Common imports for working with `type-lib`.
//!
//! Glob-import this module to bring the foundation types into scope in one line.
//!
//! # Examples
//!
//! ```rust
//! use type_lib::prelude::*;
//!
//! struct NonEmpty;
//!
//! impl<S: AsRef<str> + ?Sized> Validator<S> for NonEmpty {
//!     type Error = ValidationError;
//!
//!     fn validate(value: &S) -> Result<(), Self::Error> {
//!         if value.as_ref().is_empty() {
//!             Err(ValidationError::new("non_empty", "value must not be empty"))
//!         } else {
//!             Ok(())
//!         }
//!     }
//! }
//!
//! let ok: Result<Refined<&str, NonEmpty>, _> = Refined::new("hello");
//! assert!(ok.is_ok());
//! ```

pub use crate::error::ValidationError;
pub use crate::refined::Refined;
pub use crate::validator::Validator;
